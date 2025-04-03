// Remove the module nesting and make it a standalone file
use alloy_primitives::{address, Address, U256};


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
}

fn main() {
    // Instantiate the vault
    let vault = Vault;
    
    // Create Address instances from the hardcoded values
    let token_in = address!("75faf114eafb1bdbe2f0316df893fd58ce46aa4d");
    let token_out = address!("980b62da83eff3d4576c647993b0c1d7faf17c73");
    let recipient = address!("bebbe2bacc1f5caf9a471838b7567ff636093c84");
    
    // Convert other values to appropriate types
    let fee: u32 = 3000;
    let amount_in = U256::from(100u64);
    let amount_out_minimum = U256::from(100u64);
    let sqrt_price_limit_x96 = U256::ZERO;
    
    // Call the function with the hardcoded values
    let calldata = vault._create_swap_calldata(
        token_in,
        token_out,
        fee,
        recipient,
        amount_in,
        amount_out_minimum,
        sqrt_price_limit_x96
    );
    
    // Print in 0x-prefixed hex format
    println!("0x{}", calldata.iter().map(|b| format!("{:02x}", b)).collect::<String>());
}