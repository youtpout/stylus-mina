// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

use openzeppelin_crypto::{
    arithmetic::uint::U256,
    field::{instance::FpKimchi, prime::PrimeField},
    poseidon_mina::{instance::kimchi::KimchiParams, PoseidonMina},
};

extern crate alloc;

pub mod merkle_map;

// Si tu veux exposer directement MerkleTree depuis lib.rs :
pub use merkle_map::MerkleMap;
pub use merkle_map::MerkleMapErrors;

use alloc::{vec, vec::Vec};
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::FixedBytes, prelude::*};

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct PoseidonContract {
    MerkleMap merkle_map;
    }
}

/// Declare that `Counter` is a contract with the following external methods.
#[public]
impl PoseidonContract {
    pub fn init(&mut self, h: alloy_primitives::U256) -> Result<(), MerkleMapErrors> {
        self.merkle_map.init(h)
    }

    pub fn set_leaf(
        &mut self,
        key: alloy_primitives::U256,
        leaf: FixedBytes<32>,
    ) -> Result<(), MerkleMapErrors> {
        self.merkle_map.set_leaf(key, leaf)
    }

    pub fn get_leaf(&self, key: alloy_primitives::U256) -> alloy_primitives::U256 {
        self.merkle_map.get_leaf(key)
    }

    pub fn get_root(&self) -> FixedBytes<32> {
        self.merkle_map.get_root()
    }

    pub fn hash(&self, vec: Vec<alloy_primitives::U256>) -> alloy_primitives::U256 {
        let mut poseidon = PoseidonMina::<KimchiParams, FpKimchi>::new();

        for input in vec.iter() {
            let fp = FpKimchi::from_bigint(U256::from(*input));
            poseidon.absorb(&[fp]);
        }

        let hash = poseidon.squeeze();
        hash.into_bigint().into()
    }

    pub fn hash_function(&mut self, vec: Vec<alloy_primitives::U256>) -> alloy_primitives::U256 {
        let mut poseidon = PoseidonMina::<KimchiParams, FpKimchi>::new();

        for input in vec.iter() {
            let fp = FpKimchi::from_bigint(U256::from(*input));
            poseidon.absorb(&[fp]);
        }

        let hash = poseidon.squeeze();
        hash.into_bigint().into()
    }
}
