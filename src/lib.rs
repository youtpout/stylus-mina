// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

use openzeppelin_crypto::{
    arithmetic::uint::U256,
    field::{instance::FpKimchi, prime::PrimeField},
    poseidon_mina::{instance::kimchi::KimchiParams, PoseidonMina},
};

extern crate alloc;

pub mod merkle;

// Si tu veux exposer directement MerkleTree depuis lib.rs :
pub use merkle::MerkleErrors;
pub use merkle::MerkleTree;

use alloc::{vec, vec::Vec};
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::prelude::*;

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct PoseidonContract {
    MerkleTree merkle;
    }
}

/// Declare that `Counter` is a contract with the following external methods.
#[public]
impl PoseidonContract {
    pub fn init_merkle(&mut self, depth: alloy_primitives::U256) -> Result<(), MerkleErrors> {
        self.merkle.init(depth)
    }

    pub fn insert_leaf(
        &mut self,
        leaf: alloy_primitives::FixedBytes<32>,
    ) -> Result<alloy_primitives::U256, MerkleErrors> {
        self.merkle.insert(leaf)
    }

    pub fn merkle_root(&self) -> alloy_primitives::FixedBytes<32> {
        self.merkle.root()
    }

    pub fn merkle_size(&self) -> alloy_primitives::U256 {
        self.merkle.size()
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
