// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

mod poseidon;
use poseidon::PoseidonHash;

#[macro_use]
extern crate alloc;

use alloc::{vec, vec::Vec};
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::U256,  prelude::*};

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct PoseidonContract {
      
    }
}

/// Declare that `Counter` is a contract with the following external methods.
#[public]
impl PoseidonContract {
   
    pub fn hash(&self, vec: Vec<U256>) -> U256 {
        let poseidon_input = PoseidonHash::hash(vec);
        return poseidon_input;
    }

     pub fn hash_function(&mut self, vec: Vec<U256>) -> U256 {
        let poseidon_input = PoseidonHash::hash(vec);
        return poseidon_input;
    }
}
