#![allow(dead_code)]

use openzeppelin_crypto::{
    arithmetic::{uint, BigInteger},
    field::{instance::FpKimchi, prime::PrimeField},
    poseidon_mina::{instance::kimchi::KimchiParams, PoseidonMina},
};

use alloc::{vec, vec::Vec};
use alloy_sol_types::sol;
use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256},
    crypto::keccak,
    prelude::*,
    storage::StorageVec,
};

/// Storage layout: height, zeroes array and a mapping from composite key -> bytes32 node
sol_storage! {
    #[entrypoint]
    pub struct MerkleTree {
        /// tree height (number of levels). Example: 32
        uint256 height;
        /// default zero-hashes per level (zeros[0]..zeros[height-1])
        bytes32[] zeroes;
        /// compact storage for nodes: mapping( bytes32 => bytes32 ) where key = keccak(level||index)
        mapping(uint256 => mapping(uint256=>bytes32)) nodes;
    }
}

sol! {
    error InvalidHeight();
    error AlreadyInitialized();
    error IndexOutOfRange();
    error KeyTooLarge();
}
#[derive(SolidityError)]
pub enum MerkleErrors {
    InvalidHeight(InvalidHeight),
    AlreadyInitialized(AlreadyInitialized),
    IndexOutOfRange(IndexOutOfRange),
    KeyTooLarge(KeyTooLarge),
}

#[public]
impl MerkleTree {
    /// Initialize the tree: set height and precompute zeroes
    pub fn init(&mut self, h: U256) -> Result<(), MerkleErrors> {
        if self.height.get() != U256::ZERO {
            return Err(MerkleErrors::AlreadyInitialized(AlreadyInitialized {}));
        }

        let height_usize: usize = h
            .try_into()
            .map_err(|_| MerkleErrors::InvalidHeight(InvalidHeight {}))?;
        self.height.set(h);

        let zero0 = FixedBytes::from([0u8; 32]);
        self.zeroes.push(zero0);

        for i in 1..height_usize {
            let prev = self.zeroes.get(i - 1).unwrap();
            let next = Self::hash_pair(prev, prev);
            self.zeroes.push(next);
        }

        Ok(())
    }

    /// Get a node (fallback to zeroes[level] if unset)
    pub fn get_node(&self, level: U256, index: U256) -> FixedBytes<32> {
        // Access nested mapping: nodes[level][index]
        let level_map = self.nodes.getter(level);
        let val = level_map.get(index);
        if val != FixedBytes::from([0u8; 32]) {
            // si c’est non nul, retourne-le
            return val;
        }

        // fallback to zeroes[level]
        self.zeroes.get(level).unwrap()
    }

    /// Set a leaf at index and update parents to root
    pub fn set_leaf(&mut self, key: U256, leaf: FixedBytes<32>) -> Result<(), MerkleErrors> {
        let index = Self::key_to_index(key);
        let h_usize: usize = self.height.get().try_into().unwrap();
        if index >= (U256::from(1u8) << (h_usize - 1)) {
            return Err(MerkleErrors::IndexOutOfRange(IndexOutOfRange {}));
        }

        // Set leaf
        self.nodes.setter(U256::ZERO).insert(index, leaf);

        let mut curr_index = index;
        for level in 1..h_usize {
            curr_index = curr_index >> 1;
            let left = self.get_node(U256::from(level - 1), curr_index << 1);
            let right = self.get_node(U256::from(level - 1), (curr_index << 1) + U256::from(1u8));
            let parent = Self::hash_pair(left, right);

            self.nodes
                .setter(U256::from(level))
                .insert(curr_index, parent);
        }

        Ok(())
    }

    /// key to index like in mina merkle map
    pub fn key_to_index(key: U256) -> U256 {
        let bytes = key.to_le_bytes::<32>();

        let mut n = U256::ZERO;
        let mut bit_count = 0;

        for &b in &bytes {
            for i in 0..8 {
                if bit_count == 255 {
                    break;
                }
                let bit = (b >> i) & 1;

                if bit_count == 254 && bit == 1 {
                    panic!("Key must be less than 2^254");
                }

                n = (n << 1) | U256::from(bit as u8);
                bit_count += 1;
            }
            if bit_count == 255 {
                break;
            }
        }

        n
    }

    pub fn get_root(&self) -> FixedBytes<32> {
        let h = self.height.get();
        let root_level = h - U256::from(1u8);
        self.get_node(root_level, U256::from(0u8))
    }

    // ----- hashing helper: Poseidon (Kimchi params) -----
    /// Hash a pair of 32-byte nodes with Poseidon Mina (Kimchi params).
    fn hash_pair(a: FixedBytes<32>, b: FixedBytes<32>) -> FixedBytes<32> {
        // Convert FixedBytes<32> into U256 (big-endian)
        let int_a = U256::from_be_bytes(a.0);
        let int_b = U256::from_be_bytes(b.0);

        // Create Poseidon hasher with Kimchi parameters
        let mut poseidon = PoseidonMina::<KimchiParams, FpKimchi>::new();

        // Convert U256 -> FpKimchi field elements and absorb them into the Poseidon state
        let fa = FpKimchi::from_bigint(uint::U256::from(int_a));
        let fb = FpKimchi::from_bigint(uint::U256::from(int_b));
        poseidon.absorb(&[fa, fb]);

        // Compute the Poseidon hash (squeeze one field element)
        let hash = poseidon.squeeze();

        // Convert back to [u8; 32] (big-endian) and wrap in FixedBytes<32>
        let out: U256 = hash.into_bigint().into();
        FixedBytes(out.to_be_bytes())
    }
}

pub fn storagevec_set<'a, S: SimpleStorageType<'a>>(
    vec: &mut StorageVec<S>,
    index: usize,
    value: S::Wraps<'a>,
) {
    if let Some(mut slot) = vec.setter(index) {
        slot.set_by_wrapped(value);
    } else {
        while vec.len() <= index {
            vec.grow();
        }
        let mut slot = vec.setter(index).expect("slot doit exister après grow");
        slot.set_by_wrapped(value);
    }
}
