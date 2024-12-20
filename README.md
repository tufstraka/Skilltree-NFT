# 🎮 SkillTree NFT Marketplace on ICP

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Built on Internet Computer](https://img.shields.io/badge/Built%20on-Internet%20Computer-blue)](https://internetcomputer.org/)

A decentralized NFT marketplace for trading skill-based digital assets on the Internet Computer Protocol (ICP). Create, trade, and manage NFTs representing skills, achievements, or digital credentials.

## ✨ Features

- 🎨 Mint skill-based NFTs with custom metadata
- 💰 Buy and sell NFTs using ICP tokens
- 🔄 Resale functionality with price management
- 👥 Creator royalties (10% of each sale)
- 💼 Built-in balance management
- 🔒 Secure state management with upgrades
- 🏷️ NFT activation/deactivation controls

## 🛠️ Technology Stack

- [Internet Computer Protocol](https://internetcomputer.org/)
- [Rust](https://www.rust-lang.org/)
- [Candid](https://internetcomputer.org/docs/current/developer-docs/build/candid/)
- [IC CDK](https://docs.rs/ic-cdk)

## 📋 Prerequisites

- [DFX SDK](https://internetcomputer.org/docs/current/developer-docs/build/install-upgrade-remove)
- Rust and Cargo
- Node.js (for development)

## 🚀 Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/skilltree-nft

# Navigate to project directory
cd skilltree-nft

# Install dependencies
npm install

# Start local Internet Computer replica
dfx start --background

# Deploy the canister
dfx deploy
```