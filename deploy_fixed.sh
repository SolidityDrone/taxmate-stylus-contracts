#!/bin/bash

# Exit on error
set -e

# Convert .env file to Unix format (remove CRLF)
if [ -f .env ]; then
    # Create a temporary file with Unix line endings
    tr -d '\r' < .env > .env.tmp
    mv .env.tmp .env
    source .env
else
    echo "Error: .env file not found"
    exit 1
fi

# Check if required variables are set
required_vars=("PRIVATE_KEY" "RPC_URL" "CONTRACT_ADDRESS" "USDC_ADDRESS" "WETH_ADDRESS" "ROUTER_ADDRESS")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo "Error: $var is not set in .env file"
        exit 1
    fi
done

# Remove any 0x prefix if present
PRIVATE_KEY=$(echo "$PRIVATE_KEY" | sed 's/^0x//')

# Function to display a loading bar
show_loading_bar() {
    echo -ne "Processing: ["
    for i in {1..30}; do
        echo -ne "="
        sleep 0.1
    done
    echo -ne ">]\n"
}

echo "Starting deployment and operations..."
echo "======================================"

# 1. Initialize the contract - using exactly the command from notes.txt
echo "1. Initializing contract..."
cast send --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" "$CONTRACT_ADDRESS" "initialize(address,address,address,address[])" "$USDC_ADDRESS" "$USDC_ADDRESS" "$ROUTER_ADDRESS" "[0x980B62Da83eFf3D4576C647993b0c1D7faf17c73, 0xfEDD4b1fFe0deeF84F22E42aC94904142Ba99807, 0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d]"
show_loading_bar
echo "Initialization completed."
sleep 5

# 2. Approve USDC for the contract
echo "2. Approving USDC tokens..."
cast send --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" "$USDC_ADDRESS" "approve(address,uint256)" "$CONTRACT_ADDRESS" 1000
show_loading_bar
echo "Approval completed."
sleep 5

# 3. Deposit USDC into the contract
echo "3. Depositing USDC tokens..."
cast send --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" "$CONTRACT_ADDRESS" "deposit(uint256)" 1000 --gas-limit 500000
show_loading_bar
echo "Deposit completed."
sleep 5

# 4. Rebalance - Swap some USDC to WETH
echo "4. Rebalancing funds (USDC -> WETH)..."
cast send --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" "$CONTRACT_ADDRESS" "rebalance(address[],bool[],uint256[])" "[$WETH_ADDRESS]" "[false]" "[100]" --gas-limit 800000
show_loading_bar
echo "Rebalance completed."
sleep 5

# 5. Withdraw tokens
echo "5. Withdrawing tokens..."
cast send --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" "$CONTRACT_ADDRESS" "withdraw(uint256)" 100 --gas-limit 500000
show_loading_bar
echo "Withdrawal completed."

echo ""
echo "All operations completed successfully!"