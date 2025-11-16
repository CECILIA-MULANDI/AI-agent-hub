import { createInkSdk } from "@polkadot-api/sdk-ink";
import { createClient } from "polkadot-api";
import { withPolkadotSdkCompat } from "polkadot-api/polkadot-sdk-compat";
import { getWsProvider } from "polkadot-api/ws-provider";
import { config } from "../config/index";
import WebSocket from 'ws';

// Make WebSocket available globally for PAPI
if (typeof global !== 'undefined' && !(global as any).WebSocket) {
    (global as any).WebSocket = WebSocket;
}

let descriptors: any = null;
let passet: any = null;

let client: any = null;
let inkSdk: any = null;
let serviceRegistry: any = null;
let paymentEscrow: any = null;

export async function initContracts() {
    if (client) {
        return { client, inkSdk, serviceRegistry, paymentEscrow };
    }

    try {
        // Load descriptors
        if (!descriptors) {
            try {
                // Try @polkadot-api/descriptors
                const desc = await import("@polkadot-api/descriptors");
                console.log("📦 Descriptors loaded from @polkadot-api/descriptors");
                descriptors = desc;
                passet = (desc as any).passet;
            } catch (e1) {
                console.error("❌ Failed to load from @polkadot-api/descriptors:", e1);
                try {
                    // Try local descriptors
                    const desc = await import("../../.papi/descriptors");
                    console.log("📦 Descriptors loaded from .papi/descriptors");
                    descriptors = desc;
                    passet = (desc as any).passet;
                } catch (e2) {
                    console.error("❌ Failed to load from .papi/descriptors:", e2);
                    throw new Error("Descriptors not found. Run 'npx papi' to generate them.");
                }
            }
        }

        // Check if contracts exist in descriptors
        if (!descriptors.contracts) {
            console.error("❌ descriptors.contracts is undefined");
            console.log("Available descriptor keys:", Object.keys(descriptors));
            throw new Error("Contracts not found in descriptors. Make sure you've run 'npx papi ink add' for both contracts.");
        }

        console.log("🔍 Available contracts:", Object.keys(descriptors.contracts));

        // Validate contracts exist
        if (!descriptors.contracts.serviceRegistry) {
            console.error("❌ serviceRegistry contract not found");
            console.log("Available contracts:", Object.keys(descriptors.contracts));
            throw new Error("ServiceRegistry contract not found. Make sure you've run 'npx papi ink add contracts/service_registry.json --key serviceRegistry'");
        }

        if (!descriptors.contracts.paymentEscrow) {
            console.error("❌ paymentEscrow contract not found");
            console.log("Available contracts:", Object.keys(descriptors.contracts));
            throw new Error("PaymentEscrow contract not found. Make sure you've run 'npx papi ink add contracts/payment_escrow.json --key paymentEscrow'");
        }

        // Create PAPI client connected to PassetHub
        // Try passing WebSocket explicitly, or let it use the global one
        console.log("🔌 Creating WebSocket connection...");
        console.log("🔍 WebSocket type:", typeof WebSocket);
        
        // Create provider - try without passing WebSocket if it's global
        const provider = getWsProvider(config.substrateRpcUrl, {
            websocketClass: WebSocket as any
        });
        
        client = createClient(
            withPolkadotSdkCompat(provider)
        );

        // Initialize Ink! SDK
        inkSdk = createInkSdk(client);

        // Get contract instances
        console.log("🔧 Creating contract instances...");
        serviceRegistry = inkSdk.getContract(
            descriptors.contracts.serviceRegistry,
            config.serviceRegistryAddress
        );

        paymentEscrow = inkSdk.getContract(
            descriptors.contracts.paymentEscrow,
            config.paymentEscrowAddress
        );

        console.log("✅ PAPI contracts initialized");
        console.log(`📝 Service Registry: ${config.serviceRegistryAddress}`);
        console.log(`💰 Payment Escrow: ${config.paymentEscrowAddress}`);

        return { client, inkSdk, serviceRegistry, paymentEscrow };
    } catch (error) {
        console.error("❌ Failed to initialize contracts:", error);
        throw error;
    }
}

export function getServiceRegistry() {
    if (!serviceRegistry) {
        throw new Error("Contracts not initialized. Call initContracts() first.");
    }
    return serviceRegistry;
}

export function getPaymentEscrow() {
    if (!paymentEscrow) {
        throw new Error("Contracts not initialized. Call initContracts() first.");
    }
    return paymentEscrow;
}