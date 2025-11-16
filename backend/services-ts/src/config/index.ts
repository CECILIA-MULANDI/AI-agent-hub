import dotenv from "dotenv";

// Load environment variables from .env file
dotenv.config();

interface Config {
  substrateRpcUrl: string;
  chainName: string;
  serviceRegistryAddress: string;
  paymentEscrowAddress: string;
  serviceAccount: string;
  port: number;
  nodeEnv: string;
}

// Validate and create config object
function loadConfig(): Config {
  const substrateRpcUrl = process.env.SUBSTRATE_RPC_URL;
  const chainName = process.env.CHAIN_NAME;
  const serviceRegistryAddress = process.env.SERVICE_REGISTRY_ADDRESS;
  const paymentEscrowAddress = process.env.PAYMENT_ESCROW_ADDRESS;
  const serviceAccount = process.env.SERVICE_ACCOUNT;
  const port = process.env.PORT;
  const nodeEnv = process.env.NODE_ENV;

  // Validate required environment variables
  const missing: string[] = [];

  if (!substrateRpcUrl) missing.push("SUBSTRATE_RPC_URL");
  if (!chainName) missing.push("CHAIN_NAME");
  if (!serviceRegistryAddress) missing.push("SERVICE_REGISTRY_ADDRESS");
  if (!paymentEscrowAddress) missing.push("PAYMENT_ESCROW_ADDRESS");
  if (!serviceAccount) missing.push("SERVICE_ACCOUNT");

  if (missing.length > 0) {
    throw new Error(
      `Missing required environment variables: ${missing.join(", ")}\n` +
      "Please check your .env file"
    );
  }

  return {
    substrateRpcUrl: substrateRpcUrl as string,
    chainName: chainName as string,
    serviceRegistryAddress: serviceRegistryAddress as string,
    paymentEscrowAddress: paymentEscrowAddress as string,
    serviceAccount: serviceAccount as string,
    port: port ? parseInt(port, 10) : 3000,
    nodeEnv: nodeEnv || "development",
  };
}


export const config = loadConfig();


export type { Config };