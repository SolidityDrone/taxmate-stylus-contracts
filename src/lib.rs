// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// Modules and imports
mod erc20;

use crate::erc20::{Erc20, Erc20Error, Erc20Params};
use alloy_primitives::{Address, U256};
use stylus_sdk::{
    call::{Call},
    msg, 
    contract,
    evm,
    prelude::*,
};

/// Immutable definitions
struct VaultTokenParams;
impl Erc20Params for VaultTokenParams {
    const NAME: &'static str = "Erc20";
    const SYMBOL: &'static str = "Metric";
    const DECIMALS: u8 = 18;
}

// Define Solidity struct for Uniswap params using alloy_sol_types
use alloy_sol_types::sol;

sol! {
    struct ExactInputParams {
        bytes path;
        address recipient;
        uint256 deadline;
        uint256 amountIn;
        uint256 amountOutMinimum;
    }
}

sol_storage! {
    #[entrypoint]
    struct Vault {
        #[borrow]
        
        Erc20<VaultTokenParams> erc20;
        address metric_address;
        address[] tokens_held;
        address usdc_address;
    }
}

// Define external ERC20 interface for calling other contracts
sol_interface! {
    interface IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
        function approve(address spender, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
    }

    
    interface ISwapRouter {
        function exactInputSingle(ExactInputParams calldata params) external payable returns (uint256 amountOut);
    }   

}

#[public]
#[inherit(Erc20<VaultTokenParams>)]
impl Vault {
    pub fn deposit(&mut self, amount_in: U256) -> Result<(), Erc20Error> {
        // Get the USDC address first
        let usdc_address = self.usdc_address.get();
        let usdc = IERC20::new(usdc_address);
        let config = Call::new_in(self)   
            .gas(evm::gas_left() / 2);    
        // Call the transferFrom function on the USDC contract
        usdc.transfer_from(config, msg::sender(), contract::address(), amount_in);
        // Mint the vault tokens to the sender

        // {Calculate the amount of vault tokens to mint}
        self.erc20.mint(msg::sender(), amount_in)?;

        Ok(())
    }
    
    pub fn withdraw(&mut self, amount_out: U256) -> Result<(), Erc20Error> {
        // burn sender tokens   
        self.erc20.burn(msg::sender(), amount_out)?;
        // Calculte the % of the supply that the user has
        let supply = self.erc20.total_supply();
        let _percentage = amount_out / supply;
        
        
        Ok(())
    }

}

// internal functions   
impl Vault {
    pub fn _mint(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(msg::sender(), value)?;
        Ok(())
    }

    /// Mints tokens to another address
    pub fn _mint_to(&mut self, to: Address, value: U256) -> Result<(), Erc20Error> {
        self.erc20.mint(to, value)?;
        Ok(())
    }

    pub fn _burn(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.erc20.burn(msg::sender(), value)?;
        Ok(())
    }


}   