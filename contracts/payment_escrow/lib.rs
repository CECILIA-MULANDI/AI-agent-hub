#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod payment_escrow {
    
   
    use ink::primitives::H160;
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;
    use ink::prelude::string::String;
    /// Different statuses of an escrow
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]

    pub enum EscrowStatus {
        Pending,
        Completed,
        Refunded,
        Disputed,
    }
    /// Escrow details
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    pub struct EscrowDetails {
        pub id: u64,
        pub payer: H160,
        pub payee: H160,
        pub amount: Balance,
        pub service_id: u64,
        pub status: EscrowStatus,
        pub created_at: u64,
        pub completed_at: Option<u64>,
        pub payment_code: String,
    }

    
    /// Errors
       #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Emitted when the escrow is not found
        EscrowNotFound,
        /// Emitted when the caller is not authorized
        Unauthorized,
        /// Emitted when the amount is invalid
        InvalidAmount,
        /// Emitted when the status is invalid
        InvalidStatus,
        /// Emitted when the funds are insufficient
        InsufficientFunds,
        /// Emitted when the transfer fails
        TransferFailed,
        /// Emitted when the escrow is already completed
        AlreadyCompleted,
        /// Emitted when the escrow has expired
        EscrowExpired,
    }

    /// Result type
    pub type Result<T> = core::result::Result<T, Error>;
   /// Storage for our escrow contract
    #[ink(storage)]
    pub struct PaymentEscrow {
        escrows: Mapping<u64, EscrowDetails>,
        escrow_count: u64,
        user_escrows: Mapping<H160, Vec<u64>>,
        // Timeout period in milliseconds (e.g., 1 hour = 3600000)
        escrow_timeout: u64,

    }
    /// Events
    #[ink(event)]
    pub struct EscrowCreated {
        #[ink(topic)]
        escrow_id: u64,
        #[ink(topic)]
        payer: H160,
        #[ink(topic)]
        payee: H160,
        amount: Balance,
        service_id: u64,
    }

    #[ink(event)]
    pub struct EscrowCompleted {
        #[ink(topic)]
        escrow_id: u64,
        #[ink(topic)]
        payee: H160,
        amount: Balance,
    }

    #[ink(event)]
    pub struct EscrowRefunded {
        #[ink(topic)]
        escrow_id: u64,
        #[ink(topic)]
        payer: H160,
        amount: Balance,
    }

    #[ink(event)]
    pub struct EscrowDisputed {
        #[ink(topic)]
        escrow_id: u64,
        #[ink(topic)]
        disputer: H160,
    }

    impl PaymentEscrow {
        #[ink(constructor)]
        pub fn new(escrow_timeout: u64) -> Self {
            Self {
                escrows: Mapping::default(),
                escrow_count: 0,
                user_escrows: Mapping::default(),
                escrow_timeout
            }
        }
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(3600000) 
        }
        /// Creates an escrow
        #[ink(message, payable)]
        pub fn create_escrow(
            &mut self,
            payee: H160,
            service_id: u64,
            payment_code: String,
        ) -> Result<u64> {
            let payer = self.env().caller();
            let amount = self.env().transferred_value();

            // Validate amount
            if amount == Balance::from(0u128).into() {
                return Err(Error::InvalidAmount);
            }

            // Increment escrow count
            self.escrow_count += 1;
            let escrow_id = self.escrow_count;

            // Create escrow
            let escrow = EscrowDetails {
                id: escrow_id,
                payer,
                payee,
                amount: amount.try_into().unwrap_or_default(),
                service_id,
                status: EscrowStatus::Pending,
                created_at: self.env().block_timestamp(),
                completed_at: None,
                payment_code,
            };

            // Store escrow
            self.escrows.insert(escrow_id, &escrow);

            // Update user escrow lists
            let mut payer_escrows = self.user_escrows.get(payer).unwrap_or_default();
            payer_escrows.push(escrow_id);
            self.user_escrows.insert(payer, &payer_escrows);

            let mut payee_escrows = self.user_escrows.get(payee).unwrap_or_default();
            payee_escrows.push(escrow_id);
            self.user_escrows.insert(payee, &payee_escrows);

            // Emit event
            self.env().emit_event(EscrowCreated {
                escrow_id,
                payer,
                payee,
                amount: amount.try_into().unwrap_or_default(),
                service_id,
            });

            Ok(escrow_id)
        }

        /// Release payment to provider 
        #[ink(message)]
        pub fn release_payment(&mut self, escrow_id: u64) -> Result<()> {
            let caller = self.env().caller();
            let mut escrow = self.escrows.get(escrow_id).ok_or(Error::EscrowNotFound)?;

            // Check authorization (only payer can release)
            if escrow.payer != caller {
                return Err(Error::Unauthorized);
            }

            // Check status
            if escrow.status != EscrowStatus::Pending {
                return Err(Error::InvalidStatus);
            }

            // Check if expired
            if self.is_escrow_expired(escrow_id)? {
                return Err(Error::EscrowExpired);
            }

            // Transfer funds to payee
            if self.env().transfer(escrow.payee, escrow.amount.into()).is_err() {
                return Err(Error::TransferFailed);
            }

            // Update escrow status
            escrow.status = EscrowStatus::Completed;
            escrow.completed_at = Some(self.env().block_timestamp());
            self.escrows.insert(escrow_id, &escrow);

            // Emit event
            self.env().emit_event(EscrowCompleted {
                escrow_id,
                payee: escrow.payee,
                amount: escrow.amount,
            });

            Ok(())
        }

        /// Auto-release payment (can be called by provider after timeout)
        #[ink(message)]
        pub fn auto_release_payment(&mut self, escrow_id: u64) -> Result<()> {
            let caller = self.env().caller();
            let mut escrow = self.escrows.get(escrow_id).ok_or(Error::EscrowNotFound)?;

            // Check authorization (only payee can auto-release)
            if escrow.payee != caller {
                return Err(Error::Unauthorized);
            }

            // Check status
            if escrow.status != EscrowStatus::Pending {
                return Err(Error::InvalidStatus);
            }

            // Check if expired (must be expired for auto-release)
            if !self.is_escrow_expired(escrow_id)? {
                return Err(Error::InvalidStatus);
            }

            // Transfer funds to payee
            if self.env().transfer(escrow.payee, escrow.amount.into()).is_err() {
                return Err(Error::TransferFailed);
            }

            // Update escrow status
            escrow.status = EscrowStatus::Completed;
            escrow.completed_at = Some(self.env().block_timestamp());
            self.escrows.insert(escrow_id, &escrow);

            // Emit event
            self.env().emit_event(EscrowCompleted {
                escrow_id,
                payee: escrow.payee,
                amount: escrow.amount,
            });

            Ok(())
        }

        /// Refund payment to payer
        #[ink(message)]
        pub fn refund(&mut self, escrow_id: u64) -> Result<()> {
            let caller = self.env().caller();
            let mut escrow = self.escrows.get(escrow_id).ok_or(Error::EscrowNotFound)?;

            // Check authorization (both parties or expired timeout for payer)
            let is_authorized = escrow.payer == caller
                || (escrow.payee == caller && self.is_escrow_expired(escrow_id)?);

            if !is_authorized {
                return Err(Error::Unauthorized);
            }

            // Check status
            if escrow.status != EscrowStatus::Pending {
                return Err(Error::InvalidStatus);
            }

            // Transfer funds back to payer
            if self.env().transfer(escrow.payer, escrow.amount.into()).is_err() {
                return Err(Error::TransferFailed);
            }

            // Update escrow status
            escrow.status = EscrowStatus::Refunded;
            escrow.completed_at = Some(self.env().block_timestamp());
            self.escrows.insert(escrow_id, &escrow);

            // Emit event
            self.env().emit_event(EscrowRefunded {
                escrow_id,
                payer: escrow.payer,
                amount: escrow.amount,
            });

            Ok(())
        }

        /// Dispute an escrow
        #[ink(message)]
        pub fn dispute_escrow(&mut self, escrow_id: u64) -> Result<()> {
            let caller = self.env().caller();
            let mut escrow = self.escrows.get(escrow_id).ok_or(Error::EscrowNotFound)?;

            // Check authorization (payer or payee)
            if escrow.payer != caller && escrow.payee != caller {
                return Err(Error::Unauthorized);
            }

            // Check status
            if escrow.status != EscrowStatus::Pending {
                return Err(Error::InvalidStatus);
            }

            // Update status
            escrow.status = EscrowStatus::Disputed;
            self.escrows.insert(escrow_id, &escrow);

            // Emit event
            self.env().emit_event(EscrowDisputed {
                escrow_id,
                disputer: caller,
            });

            Ok(())
        }

        /// Get escrow details
        #[ink(message)]
        pub fn get_escrow(&self, escrow_id: u64) -> Result<EscrowDetails> {
            self.escrows.get(escrow_id).ok_or(Error::EscrowNotFound)
        }

        /// Get user escrows
        #[ink(message)]
        pub fn get_user_escrows(&self, user: H160) -> ink::prelude::vec::Vec<u64> {
            self.user_escrows.get(user).unwrap_or_default()
        }

        /// Get total escrow count
        #[ink(message)]
        pub fn get_escrow_count(&self) -> u64 {
            self.escrow_count
        }

        /// Check if escrow is expired
        #[ink(message)]
        pub fn is_escrow_expired(&self, escrow_id: u64) -> Result<bool> {
            let escrow = self.escrows.get(escrow_id).ok_or(Error::EscrowNotFound)?;
            let current_time = self.env().block_timestamp();
            let elapsed = current_time.saturating_sub(escrow.created_at);
            Ok(elapsed > self.escrow_timeout)
        }

        /// Get escrow timeout period
        #[ink(message)]
        pub fn get_escrow_timeout(&self) -> u64 {
            self.escrow_timeout
        }

    }

   
}
