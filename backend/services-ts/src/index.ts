import express from 'express';
import cors from 'cors';
import { config } from './config';
import { initContracts } from './contracts';

const app = express();

// Middleware
app.use(cors());
app.use(express.json());

// Health check endpoint
app.get('/health', (req, res) => {
    res.json({ 
        status: "ok", 
        message: "x402 Services Backend is running",
        config: {
            chain: config.chainName,
            nodeEnv: config.nodeEnv,
            ...(config.nodeEnv === 'development' && {
                serviceRegistryAddress: config.serviceRegistryAddress,
                paymentEscrowAddress: config.paymentEscrowAddress,
            })
        }
    });
});

// Initialize contracts and start server
async function start() {
    try {
        console.log("🔌 Connecting to PassetHub...");
        console.log(`📡 RPC: ${config.substrateRpcUrl}`);
        
        // Initialize PAPI contracts
        await initContracts();
        
        // Start the server
        app.listen(config.port, () => {
            console.log(`🚀 Server is running on port ${config.port}`);
            console.log(`⛓️  Chain: ${config.chainName}`);
            console.log(`✅ Health check: http://localhost:${config.port}/health`);
        });
    } catch (error) {
        console.error('❌ Failed to start server:', error);
        process.exit(1);
    }
}

start();