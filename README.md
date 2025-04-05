
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
cargo stylus deploy \
  --endpoint='https://sepolia-rollup.arbitrum.io/rpc' \
  --private-key=<PRIVATE_KEY>
```

then you can update your ```.env``` variables and run:

```bash
./deploy.sh
```

make sure you give chmod -x ./deploy.sh permission to execute that file. 
This will
    1. Initialize
    2. Approve 
    3. Deposit
    4. trigger rebalance
    5. withdraw position
