// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// Modules and imports
mod erc20;

use crate::erc20::{Erc20, Erc20Error, Erc20Params};
use alloy_primitives::{Address, U256};
use stylus_sdk::{
    call::{Call, call},
    msg, 
    block,
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
        address router_address;
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
        // Calculate the % of the supply that the user has
        let supply = self.erc20.total_supply();
        let _percentage = amount_out / supply;
        
        let usdc_address = self.usdc_address.get();
        let router_address = self.router_address.get();
        
        // Get the tokens_held length and iterate manually
        let mut i = 0;
        loop {
            // Try to get the token at the current index
            let token_opt = self.tokens_held.get(i);
            
            // If we get None, we've reached the end of the array
            if token_opt.is_none() {
                break;
            }
            
            let token = token_opt.unwrap();
            
            // Get the balance of this token that the contract owns
            let token_contract = IERC20::new(token);
            let config = Call::new_in(self).gas(evm::gas_left() / 2);
            
            // Try to get the token balance, continue if it fails
            let token_balance = match token_contract.balance_of(config, contract::address()) {
                Ok(balance) => balance,
                Err(_) => {
                    i += 1;
                    continue;
                }
            };
            
            // Calculate the amount to swap based on user's percentage
            let amount_to_swap = token_balance * _percentage;
            
            if amount_to_swap > U256::ZERO {
                // Now create the config after getting the router address
                let config = Call::new_in(self).gas(evm::gas_left() / 2);
                let _ = token_contract.approve(config, router_address, amount_to_swap);
                
                // Perform the swap - ignore errors and continue
                let _ = self._swap_tokens(token, usdc_address, 3000, amount_to_swap, U256::ZERO);
            }
            
            i += 1;
        }
        
        // Burn the user's tokens after all swaps are done
        self.erc20.burn(msg::sender(), amount_out)?;
        
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

    /// Constructs calldata for the exactInputSingle function
    pub fn _create_swap_calldata(
        &self, 
        token_in: Address,
        token_out: Address,
        fee: u32,
        recipient: Address, 
        deadline: U256, 
        amount_in: U256, 
        amount_out_minimum: U256,
        sqrt_price_limit_x96: U256
    ) -> Vec<u8> {
        // Function signature for exactInputSingle is 0x414bf389
        let mut calldata = vec![0x41, 0x4b, 0xf3, 0x89];
        
        // Encode the ExactInputSingleParams struct
        // First encode the offset to the struct data (32 bytes for standard dynamic type offset)
        calldata.extend_from_slice(&U256::from(32).to_be_bytes::<32>());
        
        // Now encode the actual struct members according to the struct definition:
        // struct ExactInputSingleParams {
        //     address tokenIn;
        //     address tokenOut;
        //     uint24 fee;
        //     address recipient;
        //     uint256 deadline;
        //     uint256 amountIn;
        //     uint256 amountOutMinimum;
        //     uint160 sqrtPriceLimitX96;
        // }
        
        // First member: tokenIn (address)
        let mut token_in_bytes = [0u8; 32];
        token_in_bytes[12..].copy_from_slice(token_in.as_slice());
        calldata.extend_from_slice(&token_in_bytes);
        
        // Second member: tokenOut (address)
        let mut token_out_bytes = [0u8; 32];
        token_out_bytes[12..].copy_from_slice(token_out.as_slice());
        calldata.extend_from_slice(&token_out_bytes);
        
        // Third member: fee (uint24) - needs to be padded to 32 bytes
        let mut fee_bytes = [0u8; 32];
        fee_bytes[29] = ((fee >> 16) & 0xFF) as u8;
        fee_bytes[30] = ((fee >> 8) & 0xFF) as u8;
        fee_bytes[31] = (fee & 0xFF) as u8;
        calldata.extend_from_slice(&fee_bytes);
        
        // Fourth member: recipient (address)
        let mut recipient_bytes = [0u8; 32];
        recipient_bytes[12..].copy_from_slice(recipient.as_slice());
        calldata.extend_from_slice(&recipient_bytes);
        
        // Fifth member: deadline (uint256)
        calldata.extend_from_slice(&deadline.to_be_bytes::<32>());
        
        // Sixth member: amountIn (uint256)
        calldata.extend_from_slice(&amount_in.to_be_bytes::<32>());
        
        // Seventh member: amountOutMinimum (uint256)
        calldata.extend_from_slice(&amount_out_minimum.to_be_bytes::<32>());
        
        // Eighth member: sqrtPriceLimitX96 (uint160) - padded to 32 bytes
        calldata.extend_from_slice(&sqrt_price_limit_x96.to_be_bytes::<32>());
        
        calldata
    }

    pub fn _execute_swap(
        &mut self,
        contract: Address,
        calldata: Vec<u8>, // Calldata is supplied as a Vec<u8>
    ) -> Result<Vec<u8>, Vec<u8>> {
        // Perform a low-level `call`
        let return_data = call(
            Call::new_in(self) // Configuration for gas, value, etc.
                .gas(evm::gas_left() / 2), // Use half the remaining gas
            contract,  // The target contract address
            &calldata, // Raw calldata to be sent
        )?;

        
        // Return the raw return data from the contract call
        Ok(return_data)
    }

    /// Helper function to perform an exact input swap
    pub fn _swap_exact_input(
        &mut self,
        token_in: Address,
        token_out: Address,
        fee: u32,
        recipient: Address, 
        deadline: U256, 
        amount_in: U256, 
        amount_out_minimum: U256,
        sqrt_price_limit_x96: U256
    ) -> Result<U256, Vec<u8>> {
        // Get the router address
        let router = self.router_address.get();
        
        // Create the calldata for the swap
        let calldata = self._create_swap_calldata(
            token_in,
            token_out,
            fee,
            recipient,
            deadline,
            amount_in,
            amount_out_minimum,
            sqrt_price_limit_x96
        );
        
        // Execute the swap
        let return_data = self._execute_swap(router, calldata)?;
        
        // Parse the return data (uint256 amountOut)
        if return_data.len() >= 32 {
            let mut amount_out_bytes = [0u8; 32];
            amount_out_bytes.copy_from_slice(&return_data[0..32]);
            let amount_out = U256::from_be_bytes(amount_out_bytes);
            Ok(amount_out)
        } else {
            Err(return_data)
        }
    }

    /// Helper function to perform an exact input swap with some default values
    pub fn _swap_tokens(
        &mut self,
        token_in: Address,
        token_out: Address,
        fee: u32,
        amount_in: U256,
        amount_out_minimum: U256
    ) -> Result<U256, Vec<u8>> {
        let deadline = U256::from(block::timestamp()) + U256::from(180);
        
        // Set recipient to this contract
        let recipient = contract::address();
        
        // Set sqrtPriceLimitX96 to 0 (no price limit)
        let sqrt_price_limit_x96 = U256::ZERO;
        
        // Call the main swap function
        self._swap_exact_input(
            token_in,
            token_out,
            fee,
            recipient,
            deadline,
            amount_in,
            amount_out_minimum,
            sqrt_price_limit_x96
        )
    }
}   