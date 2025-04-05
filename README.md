# TaxMate: Automated Portfolio Management Vault

TaxMate is a smart contract vault built on Arbitrum Stylus using Rust. It allows users to deposit USDC tokens and rebalance their portfolio across multiple tokens with automated swap functionality through UniswapV3.

## Vault Smart Contract Overview

The `lib.rs` file implements a DeFi vault with the following key features:

- **ERC20 Tokenization**: Users receive Metric tokens representing their share of the vault's assets when they deposit.
- **Asset Management**: Supports multiple tokens including USDC and various ERC20 tokens.
- **Rebalancing**: Allows for swapping between tokens based on specified parameters.
- **UniswapV3 Integration**: Uses Uniswap router for token swaps.

### Key Functions

- `deposit(amount_in)`: Deposits USDC and mints vault tokens
- `withdraw(amount_out)`: Withdraws funds proportional to user's share
- `rebalance(tokens_to_swap, zero_to_one, amount_in)`: Rebalances portfolio by swapping tokens
- `initialize(metric_address, usdc_address, router_address, enabled_tokens)`: Sets up vault parameters

### Technical Architecture

The smart contract uses the Stylus SDK to compile Rust to WebAssembly, which is then executed on the Arbitrum Stylus chain. It leverages the `sol_storage!` and `sol_interface!` macros to interact with EVM storage and external contracts.

## Deployment Guide

You can deploy the TaxMate vault using the provided deployment script:

```bash
# Create the deployment script
cat > deploy_fixed.sh << 'EOF'
#!/bin/bash

# Check if Rust and required tools are installed
if ! command -v cargo &> /dev/null; then
    echo "Rust is not installed. Please install it first."
    exit 1
fi

# Install Stylus tools if not already installed
cargo install --force cargo-stylus cargo-stylus-check

# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Verify the contract compiles properly
echo "Checking contract compilation..."
cargo stylus check

# Deploy the contract
echo "Deploying contract..."
cargo stylus deploy \
  --endpoint='https://sepolia-rollup.arbitrum.io/rpc' \
  --private-key-path=./private-key.txt

echo "Contract deployed successfully!"
EOF

# Make the script executable
chmod +x deploy_fixed.sh
```

To deploy:

1. Create a text file named `private-key.txt` containing your private key
2. Run the deployment script: `./deploy_fixed.sh`
3. After deployment, initialize the contract with:
   ```
   cast send --rpc-url 'https://sepolia-rollup.arbitrum.io/rpc' \
     --private-key $(cat private-key.txt) \
     YOUR_CONTRACT_ADDRESS "initialize(address,address,address,address[])" \
     METRIC_ADDRESS USDC_ADDRESS ROUTER_ADDRESS '[TOKEN1, TOKEN2, TOKEN3]'
   ```

![Image](./header.png)

# Stylus Hello World

Project starter template for writing Arbitrum Stylus programs in Rust using the [stylus-sdk](https://github.com/OffchainLabs/stylus-sdk-rs). It includes a Rust implementation of a basic counter Ethereum smart contract:

// ... existing code ...