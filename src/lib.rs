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
    contract,
    evm,
    prelude::*,
};
use alloy_sol_types::sol;

/// Immutable definitions
struct VaultTokenParams;
impl Erc20Params for VaultTokenParams {
    const NAME: &'static str = "Metric";
    const SYMBOL: &'static str = "Metric";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    #[entrypoint]
    struct Vault {
        #[borrow]
        
        Erc20<VaultTokenParams> erc20;
        address metric_address;
        address[] enabled_tokens;
        address usdc_address;
        address router_address;
    }
}

// Define external ERC20 interface for calling other contracts
sol_interface! {
    interface IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amou2nt) external returns (bool);
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
        // Calculate the % of the supply that the user has with scaling to maintain precision
        let supply = self.erc20.total_supply();
        // Use a scaling factor of 10^18 to handle decimal percentages
        let scaling_factor = U256::from(10).pow(U256::from(18));
        let percentage = (amount_out * scaling_factor) / supply;
        
        let usdc_address = self.usdc_address.get();
        let router_address = self.router_address.get();
        
        // Get the enabled_tokens length and iterate manually
        let mut i = 0;
        loop {
            // Try to get the token at the current index
            let token_opt = self.enabled_tokens.get(i);
            
            // If we get None, we've reached the end of the array
            if token_opt.is_none() {
                break;
            }
            
            let token = token_opt.unwrap();
            
            // Get the balance of this token that the contract owns
            let token_contract = IERC20::new(token);
            let config = Call::new_in(self).gas(evm::gas_left() / 2);
            
            // Try to get the token balance, handle errors properly
            let token_balance = match token_contract.balance_of(config, contract::address()) {
                Ok(balance) => balance,
                Err(_) => {
                    // Skip this token if balance check fails, continue to next token
                    i += 1;
                    continue;
                }
            };
            
            // Calculate the amount to transfer based on user's percentage
            // Divide by scaling factor to get actual amount
            let share_total = (token_balance * percentage) / scaling_factor;
            
            if share_total > U256::ZERO {
                if token != usdc_address {
                    // For non-USDC tokens, approve and swap to USDC
                    let approve_config = Call::new_in(self).gas(evm::gas_left() / 2);
                    let _ = token_contract.approve(approve_config, router_address, share_total);
                    
                    // Perform the swap - ignore errors and continue
                    let _ = self._swap_tokens(token, usdc_address, 3000, share_total, U256::ZERO);
                } else {
                    // For USDC, transfer directly to user
                    let transfer_config = Call::new_in(self).gas(evm::gas_left() / 2);
                    let _ = token_contract.transfer(transfer_config, msg::sender(), share_total);
                }
            }
            
            i += 1;
        }
        
        // Burn the user's tokens after all swaps are done
        self.erc20.burn(msg::sender(), amount_out)?;
        
        Ok(())
    }

    pub fn initialize(&mut self, metric_address: Address, usdc_address: Address, router_address: Address, enabled_tokens: Vec<Address>) {
        self.metric_address.set(metric_address);
        self.usdc_address.set(usdc_address);
        self.router_address.set(router_address);
        
        // Clear any existing tokens and add new ones
        let length = self.enabled_tokens.len();
        for _ in 0..length {
            self.enabled_tokens.pop();
        }
        
        // Add each token from the input vector
        for token in enabled_tokens {
            self.enabled_tokens.push(token);
        }
    }

    /*  
        pub fn total_assets(&self) -> Result<U256, Vec<u8>> {
            Ok(self.erc20.total_supply.get())
        }

        pub fn usdc_address(&self) -> Result<Address, Vec<u8>> {
            Ok(self.usdc_address.get())
        }

        pub fn router_address(&self) -> Result<Address, Vec<u8>> {
            Ok(self.router_address.get())
        }   

        pub fn metric_address(&self) -> Result<Address, Vec<u8>> {
            Ok(self.metric_address.get())
        }   
    */
    pub fn vault_balances(&mut self) -> Result<Vec<U256>, Vec<u8>> {
        let mut balances = Vec::new();
        
        // Get the length of enabled_tokens
        let length = self.enabled_tokens.len();
        
        // Iterate through indices manually
        for i in 0..length {
            // Safe unwrap since we're checking within bounds
            if let Some(token) = self.enabled_tokens.get(i) {
                let token_contract = IERC20::new(token);
                let config = Call::new_in(self).gas(evm::gas_left() / 2);
                
                // Handle the Result returned by balance_of
                match token_contract.balance_of(config, contract::address()) {
                    Ok(balance) => balances.push(balance),
                    Err(_) => balances.push(U256::ZERO)  // Push zero if there's an error
                }
            }
        }
        Ok(balances)
    }


    
  /*   pub fn get_balance(&self, token: Address) -> Result<U256, Vec<u8>> {
        // Get the token contract
        let token_contract = IERC20::new(token);
        
        // Create config with proper mutability
        let config = Call::new();
        
        // Get balance with proper error handling
        match token_contract.balance_of(config, contract::address()) {
            Ok(balance) => Ok(balance),
            Err(_) => Err(vec![0]) // Return a simple error vector instead of trying to convert the Error
        }
    } */
/*     
    pub fn enabled_tokens_length(&self) -> Result<U256, Vec<u8>> {
        let length = self.enabled_tokens.len();
        Ok(U256::from(length as u64))
    } */

   /*  pub fn enabled_tokens(&self) -> Result<Vec<Address>, Vec<u8>> {
        let mut result = Vec::new();
        let length = self.enabled_tokens.len();
        
        // Safely collect tokens
        for i in 0..length {
            if let Some(token) = self.enabled_tokens.get(i) {
                result.push(token);
            }
        }
        
        Ok(result)
    }
 */
    pub fn rebalance(&mut self, tokens_to_swap: Vec<Address>, zero_to_one: Vec<bool>, amount_in: Vec<U256>) {
          /* if msg::sender() != self.metric_address.get() {
            return Err(Erc20Error::NotAuthorized);
        }   
         */
        let usdc_address = self.usdc_address.get();
        let router_address = self.router_address.get();
        let max_len = tokens_to_swap.len().min(zero_to_one.len());
        
        for i in 0..max_len {
            let token = tokens_to_swap[i];
            let is_zero_to_one = zero_to_one[i];
            
            if is_zero_to_one {
                // Swapping token -> USDC
                let token_contract = IERC20::new(token);
                
                // Create a new config for each call
                let config = Call::new_in(self).gas(evm::gas_left());
                let token_balance = match token_contract.balance_of(config, contract::address()) {
                    Ok(balance) => balance,
                    Err(_) => continue,
                };
                
                if token_balance > U256::ZERO {
                    // Create a new config for the approve call
                    let config = Call::new_in(self).gas(evm::gas_left());
                    let _ = token_contract.approve(config, router_address, amount_in[i]);
                    
                    // Swap token -> USDC
                    let _ = self._swap_tokens(token, usdc_address, 3000, amount_in[i], U256::ZERO);
                }
            } else {
                // Swapping USDC -> token
                // Get USDC contract
                let usdc_contract = IERC20::new(usdc_address);
                
                // Create a new config for balance check
                let config = Call::new_in(self).gas(evm::gas_left());
                let usdc_balance = match usdc_contract.balance_of(config, contract::address()) {
                    Ok(balance) => balance,
                    Err(_) => continue,
                };
                
                if usdc_balance > U256::ZERO {
                    // Create a new config for approve
                    let config = Call::new_in(self).gas(evm::gas_left());
                    let _ = usdc_contract.approve(config, router_address, amount_in[i]);
                    
              
                    let _ = self._swap_tokens(usdc_address, token, 3000, amount_in[i], U256::ZERO);
                }
            }
        }
    }
}

// internal functions   
impl Vault {

   
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
        amount_in: U256, 
        amount_out_minimum: U256,
        sqrt_price_limit_x96: U256
    ) -> Vec<u8> {
        // Function signature for exactInputSingle (0x04e45aaf)
        let mut calldata = vec![0x04, 0xe4, 0x5a, 0xaf];
        
        // For a tuple parameter, we only need the function selector followed by the tuple values
        // No need for an offset since it's a direct tuple, not a dynamic type
        
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
        
        // Fifth member: amountIn (uint256)
        calldata.extend_from_slice(&amount_in.to_be_bytes::<32>());
        
        // Sixth member: amountOutMinimum (uint256)
        calldata.extend_from_slice(&amount_out_minimum.to_be_bytes::<32>());
        
        // Seventh member: sqrtPriceLimitX96 (uint160) - padded to 32 bytes
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
            amount_in,
            amount_out_minimum,
            sqrt_price_limit_x96
        )
    }
}   