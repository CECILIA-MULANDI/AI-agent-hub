#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod service_registry {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::primitives::H160;
    use ink::storage::Mapping;
    use ink::H256;

    /// Options for type of services an AI agent can offer

    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]

    pub enum ServiceCategory {
        TextProcessing,
        ImageGeneration,
        DataAnalysis,
        Translation,
        Computation,
    }

    /// Service structure
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Service {
        pub id: u64,
        pub provider: H160,
        pub name: String,
        pub description: String,
        pub category: ServiceCategory,
        pub price: Balance,
        pub endpoint: String,
        pub is_active: bool,
        pub total_requests: u32,
        pub successful_requests: u32,
        pub created_at: u64,
        // I need some x402 integration
        pub supports_x402: bool,
        pub x402_payment_token: Option<H160>,
        pub x402_payment_amount: Option<Balance>,
        pub x402_gateway_address: Option<H160>,
        pub x402_chain_id: Option<u64>,
    }

    /// Events
    /// Emitted when a new service is registered
    #[ink(event)]
    pub struct ServiceRegistered {
        #[ink(topic)]
        service_id: u64,
        #[ink(topic)]
        provider: H160,
        name: String,
        price: Balance,
    }
    /// Emitted when x402 payment is recorded
    #[ink(event)]
    pub struct X402PaymentRecorded {
        #[ink(topic)]
        service_id: u64,
        #[ink(topic)]
        payment_hash: H256,
        success: bool,
    }
    /// Emitted when the service status is updated
    #[ink(event)]
    pub struct ServiceUpdated {
        #[ink(topic)]
        service_id: u64,
        is_active: bool,
    }
    /// Emitted when the reputation is updated
    #[ink(event)]
    pub struct ReputationUpdated {
        #[ink(topic)]
        provider: H160,
        score: u32,
    }

    /// Errors
    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Emitted when an input is invalid
        InvalidInput,
        /// Emitted when there is an arithmetic overflow
        Overflow,
        /// Emitted when the service is not found
        ServiceNotFound,

        /// Emitted when the caller is not authorized to update the service status
        Unauthorized,
    }

    #[ink(storage)]
    pub struct ServiceRegistry {
        services: Mapping<u64, Service>,
        provider_services: Mapping<H160, Vec<u64>>,
        service_count: u64,
        reputation_scores: Mapping<H160, u32>,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl ServiceRegistry {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                services: Mapping::default(),
                provider_services: Mapping::default(),
                service_count: 0,
                reputation_scores: Mapping::default(),
            }
        }

        /// Register a new service
        #[ink(message)]
        pub fn register_service(
            &mut self,
            name: String,
            description: String,
            category: ServiceCategory,
            price: Balance,
            endpoint: String,
            supports_x402: bool,
            x402_payment_token: Option<H160>,
            x402_payment_amount: Option<Balance>,
            x402_gateway_address: Option<H160>,
            x402_chain_id: Option<u64>,
        ) -> Result<u64> {
            let caller = self.env().caller();
            if name.is_empty() || description.is_empty() || endpoint.is_empty() || price == 0 {
                return Err(Error::InvalidInput);
            }
            // Validate x402 parameters if x402 is enabled
            if supports_x402 {
                if x402_payment_token.is_none() || x402_payment_amount.is_none() {
                    return Err(Error::InvalidInput);
                }
            }

            self.service_count = self.service_count.checked_add(1).ok_or(Error::Overflow)?;
            let service_id = self.service_count;

            let service = Service {
                id: service_id,
                provider: caller,
                name: name.clone(),
                description,
                category,
                price,
                endpoint,
                is_active: true,
                total_requests: 0,
                successful_requests: 0,
                created_at: self.env().block_timestamp(),
                supports_x402,
                x402_payment_token,
                x402_payment_amount,
                x402_gateway_address,
                x402_chain_id,
            };

            self.services.insert(service_id, &service);

            let mut provider_services = self.provider_services.get(caller).unwrap_or_default();
            provider_services.push(service_id);
            self.provider_services.insert(caller, &provider_services);

            self.env().emit_event(ServiceRegistered {
                service_id,
                provider: caller,
                name,
                price,
            });

            Ok(service_id)
        }
        #[ink(message)]
        pub fn get_service(&self, service_id: u64) -> Result<Service> {
            self.services.get(service_id).ok_or(Error::ServiceNotFound)
        }
        #[ink(message)]
        pub fn update_service_status(&mut self, service_id: u64, is_active: bool) -> Result<()> {
            let caller = self.env().caller();
            let mut service = self
                .services
                .get(service_id)
                .ok_or(Error::ServiceNotFound)?;

            // Check authorization
            if service.provider != caller {
                return Err(Error::Unauthorized);
            }

            service.is_active = is_active;
            self.services.insert(service_id, &service);

            self.env().emit_event(ServiceUpdated {
                service_id,
                is_active,
            });

            Ok(())
        }
        #[ink(message)]
        pub fn record_service_request(&mut self, service_id: u64, success: bool) -> Result<()> {
            let mut service = self
                .services
                .get(service_id)
                .ok_or(Error::ServiceNotFound)?;

            service.total_requests += 1;
            if success {
                service.successful_requests += 1;
            }

            self.services.insert(service_id, &service);
            Ok(())
        }

        /// Update provider reputation
        #[ink(message)]
        pub fn update_reputation(&mut self, provider: H160, score: u32) -> Result<()> {
            self.reputation_scores.insert(provider, &score);

            self.env().emit_event(ReputationUpdated { provider, score });

            Ok(())
        }

        /// Get provider reputation
        #[ink(message)]
        pub fn get_reputation(&self, provider: H160) -> u32 {
            self.reputation_scores.get(provider).unwrap_or(0)
        }

        /// Get all services by provider
        #[ink(message)]
        pub fn get_provider_services(&self, provider: H160) -> Vec<u64> {
            self.provider_services.get(provider).unwrap_or_default()
        }

        /// Get total service count
        #[ink(message)]
        pub fn get_service_count(&self) -> u64 {
            self.service_count
        }

        /// Get active services
        /// For this MVP I have simplified it to returns first N active services
        ///
        #[ink(message)]
        pub fn get_active_services(&self, limit: u64) -> Vec<Service> {
            let mut active_services = Vec::new();
            let max = if limit > self.service_count {
                self.service_count
            } else {
                limit
            };

            for i in 1..=max {
                if let Some(service) = self.services.get(i) {
                    if service.is_active {
                        active_services.push(service);
                    }
                }
            }

            active_services
        }

        /// Update service price
        #[ink(message)]
        pub fn update_service_price(&mut self, service_id: u64, new_price: Balance) -> Result<()> {
            let caller = self.env().caller();
            let mut service = self
                .services
                .get(service_id)
                .ok_or(Error::ServiceNotFound)?;

            if service.provider != caller {
                return Err(Error::Unauthorized);
            }

            service.price = new_price;
            self.services.insert(service_id, &service);

            Ok(())
        }
        /// Update x402 payment parameters for a service
        #[ink(message)]
        pub fn update_x402_params(
            &mut self,
            service_id: u64,
            supports_x402: bool,
            x402_payment_token: Option<H160>,
            x402_payment_amount: Option<Balance>,
            x402_gateway_address: Option<H160>,
            x402_chain_id: Option<u64>,
        ) -> Result<()> {
            let caller = self.env().caller();
            let mut service = self
                .services
                .get(service_id)
                .ok_or(Error::ServiceNotFound)?;

            if service.provider != caller {
                return Err(Error::Unauthorized);
            }

            // Validate x402 parameters if x402 is enabled
            if supports_x402 {
                if x402_payment_token.is_none() || x402_payment_amount.is_none() {
                    return Err(Error::InvalidInput);
                }
            }

            service.supports_x402 = supports_x402;
            service.x402_payment_token = x402_payment_token;
            service.x402_payment_amount = x402_payment_amount;
            service.x402_gateway_address = x402_gateway_address;
            service.x402_chain_id = x402_chain_id;

            self.services.insert(service_id, &service);

            Ok(())
        }

        /// Get services that support x402 payments
        #[ink(message)]
        pub fn get_x402_services(&self, limit: u64) -> Vec<Service> {
            let mut x402_services = Vec::new();
            let max = if limit > self.service_count {
                self.service_count
            } else {
                limit
            };

            for i in 1..=max {
                if let Some(service) = self.services.get(i) {
                    if service.is_active && service.supports_x402 {
                        x402_services.push(service);
                    }
                }
            }

            x402_services
        }

        /// Record x402 payment for a service request
        #[ink(message)]
        pub fn record_x402_payment(
            &mut self,
            service_id: u64,
            payment_hash: H256,
            success: bool,
        ) -> Result<()> {
            let mut service = self
                .services
                .get(service_id)
                .ok_or(Error::ServiceNotFound)?;

            if !service.supports_x402 {
                return Err(Error::InvalidInput);
            }

            service.total_requests += 1;
            if success {
                service.successful_requests += 1;
            }

            self.services.insert(service_id, &service);
            Ok(())
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn register_service_works() {
            let mut contract = ServiceRegistry::new();
            ink::env::test::default_accounts();

            let result = contract.register_service(
                String::from("Text Summarizer"),
                String::from("AI-powered text summarization"),
                ServiceCategory::TextProcessing,
                1000,
                String::from("https://api.example.com/summarize"),
            );

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 1);
            assert_eq!(contract.get_service_count(), 1);
        }

        #[ink::test]
        fn get_service_works() {
            let mut contract = ServiceRegistry::new();

            let service_id = contract
                .register_service(
                    String::from("Test Service"),
                    String::from("Description"),
                    ServiceCategory::Computation,
                    500,
                    String::from("https://test.com"),
                )
                .unwrap();

            let service = contract.get_service(service_id).unwrap();
            assert_eq!(service.name, String::from("Test Service"));
            assert_eq!(service.price, 500);
        }

        #[ink::test]
        fn update_status_works() {
            let mut contract = ServiceRegistry::new();

            let service_id = contract
                .register_service(
                    String::from("Test"),
                    String::from("Desc"),
                    ServiceCategory::DataAnalysis,
                    100,
                    String::from("https://test.com"),
                )
                .unwrap();

            assert!(contract.update_service_status(service_id, false).is_ok());

            let service = contract.get_service(service_id).unwrap();
            assert_eq!(service.is_active, false);
        }

        #[ink::test]
        fn unauthorized_update_fails() {
            let mut contract = ServiceRegistry::new();
            let accounts = ink::env::test::default_accounts();

            let service_id = contract
                .register_service(
                    String::from("Test"),
                    String::from("Desc"),
                    ServiceCategory::Translation,
                    200,
                    String::from("https://test.com"),
                )
                .unwrap();

            // Change caller
            ink::env::test::set_caller(accounts.bob);

            let result = contract.update_service_status(service_id, false);
            assert_eq!(result, Err(Error::Unauthorized));
        }

        #[ink::test]
        fn reputation_system_works() {
            let mut contract = ServiceRegistry::new();
            let accounts = ink::env::test::default_accounts();

            contract.update_reputation(accounts.alice, 95).unwrap();
            assert_eq!(contract.get_reputation(accounts.alice), 95);
        }
    }
}
