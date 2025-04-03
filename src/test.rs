mod vault_test {
    use alloy_primitives::{Address, U256};
    
    // A minimal mock implementation of the Vault struct to test the calldata creation function
    struct Vault;
    
    impl Vault {
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
    }
    
    #[cfg(test)]
    pub mod test;
    
    #[cfg(test)]
    mod tests {
        use super::*;
        use alloy_primitives::{address, U256};
        
        #[test]
        fn test_create_swap_calldata() {
            // Instantiate the vault
            let vault = Vault;
            
            // Create Address instances from the hardcoded values
            let token_in = address!("75faf114eafb1bdbe2f0316df893fd58ce46aa4d");
            let token_out = address!("980b62da83eff3d4576c647993b0c1d7faf17c73");
            let recipient = address!("bebbe2bacc1f5caf9a471838b7567ff636093c84");
            
            // Convert other values to appropriate types
            let fee: u32 = 3000;
            let amount_in = U256::from(1743707118u64);
            let amount_out_minimum = U256::from(100u64);
            let sqrt_price_limit_x96 = U256::ZERO;
            
            // Use a dummy deadline value for testing
            let deadline = U256::from(1234567890u64);
            
            // Call the function with the hardcoded values
            let calldata = vault._create_swap_calldata(
                token_in,
                token_out,
                fee,
                recipient,
                deadline,
                amount_in,
                amount_out_minimum,
                sqrt_price_limit_x96
            );
            
            // Print the calldata as hex for inspection
            println!("Calldata (hex): 0x{}", hex::encode(&calldata));
            
            // You can also print the calldata length for verification
            println!("Calldata length: {} bytes", calldata.len());
            
            // Helper function to print the calldata in chunks for better readability
            fn print_calldata_chunks(data: &[u8]) {
                // Function selector
                println!("Function selector: 0x{}", hex::encode(&data[0..4]));
                
                // Print each parameter (each 32 bytes)
                let mut offset = 4;
                let chunks = ["tokenIn", "tokenOut", "fee", "recipient", "deadline", "amountIn", "amountOutMinimum", "sqrtPriceLimitX96"];
                
                for (i, chunk_name) in chunks.iter().enumerate() {
                    let chunk_end = offset + 32;
                    if chunk_end <= data.len() {
                        println!("{}: 0x{}", chunk_name, hex::encode(&data[offset..chunk_end]));
                    }
                    offset = chunk_end;
                }
            }
            
            // Print the calldata in a more readable format
            print_calldata_chunks(&calldata);
        }
    }
}

// A main function to run the test directly
fn main() {
    // Run the test
    vault_test::tests::test_create_swap_calldata();
}