# Cross-Chain AI Agent Service Hub

## Project Overview

An autonomous marketplace infrastructure enabling AI agents to discover, transact, and deliver services across blockchain networks without human intervention. Built on Polkadot using ink! smart contracts, this project demonstrates how Web3 can power machine-to-machine economies where AI agents operate as independent economic actors.

---

## Solution Architecture

```
┌─────────────────────────────────────────────────┐
│           Frontend Dashboard (React)             │
│        Service Discovery & Transaction Monitor   │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│         Backend API (Node.js + Express)          │
│   Service Registry │ Payment Handler │ Discovery │
└─────────────────┬───────────────────────────────┘
                  │
        ┌─────────┴─────────┐
        │                   │
┌───────▼────────┐  ┌──────▼──────────┐
│ ServiceRegistry│  │ PaymentEscrow   │
│   Contract     │  │   Contract      │
│    (ink!)      │  │    (ink!)       │
└────────────────┘  └─────────────────┘
        │                   │
        └─────────┬─────────┘
                  │
        ┌─────────▼─────────┐
        │  Polkadot Testnet │
        │  (Rococo/Westend) │
        └───────────────────┘

┌──────────────┐              ┌──────────────┐
│Provider Agent│◄────────────►│Consumer Agent│
│ (Autonomous) │   Services   │ (Autonomous) │
└──────────────┘              └──────────────┘
```
