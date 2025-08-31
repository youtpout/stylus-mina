#![allow(dead_code)]

use openzeppelin_crypto::{
    arithmetic::{uint, BigInteger},
    field::{instance::FpKimchi, prime::PrimeField},
    poseidon_mina::{instance::kimchi::KimchiParams, PoseidonMina},
};

use stylus_sdk::{
    alloy_primitives::{FixedBytes, U256},
    crypto::keccak,
    prelude::*,
    storage::StorageVec,
};

use alloc::{vec, vec::Vec};

/// Solidity-style storage layout for ease of use
sol_storage! {
    /// Append-only Merkle tree with Keccak-256 hashing
    #[entrypoint]
    pub struct MerkleTree {
        /// Tree depth (number of levels excluding the root level). Typical: 20-32
        uint256 depth;
        /// Next leaf index to insert
        uint256 nextIndex;
        /// Default zero hashes for each level (zeros[0]..zeros[depth])
        bytes32[] zeros; // dynamic to allow any depth set at init
        /// Last seen left sibling for each level when the current index was odd
        bytes32[] filledSubtrees; // same length as depth
        /// Current root
        bytes32 root;
        /// Whether the tree has been initialized
        bool initialized;
        /// Optional: store leaves to enable offchain proof generation or onchain auditing
        mapping(uint256 => bytes32) leaves;
    }
}

#[public]
impl MerkleTree {
    /// Initialize the tree for a given `depth`. Can be called only once.
    /// Depth must be > 0 and <= 64 (practical limit for gas + memory).
    pub fn init(&mut self, height: alloy_primitives::U256) -> Result<(), MerkleErrors> {
        if self.initialized.get() {
            return Err(MerkleErrors::AlreadyInitialized(AlreadyInitialized {}));
        }

        let h: u32 = height
            .try_into()
            .map_err(|_| MerkleErrors::InvalidDepth(InvalidDepth {}))?;
        if h == 0 || h > 256 {
            return Err(MerkleErrors::InvalidDepth(InvalidDepth {}));
        }

        self.depth.set(height);
        self.nextIndex.set(alloy_primitives::U256::ZERO);

        // zeros[0] = 0
        let zero0: FixedBytes<32> = FixedBytes::from([0u8; 32]);
        self.zeros.push(zero0);

        // 0 for each level
        for i in 1..h as usize {
            let prev = self.zeros.get(i - 1).expect("prev zero exist");
            let next = hash_pair(prev, prev);
            self.zeros.push(next);
        }

        // filledSubtrees = zeros[0..height-1]
        for i in 0..h as usize {
            let zi = self.zeros.get(i).expect("zero exist");
            self.filledSubtrees.push(zi);
        }

        // racine initiale = zeros[height-1]
        let root0 = self.zeros.get((h - 1) as usize).expect("root exist");
        self.root.set(root0);

        self.initialized.set(true);
        Ok(())
    }

    /// Insert a new leaf, returns the inserted index.
    pub fn insert(&mut self, leaf: FixedBytes<32>) -> Result<U256, MerkleErrors> {
        if !self.initialized.get() {
            return Err(MerkleErrors::NotInitialized(NotInitialized {}));
        }

        let depth_u32: u32 = self.depth.get().try_into().unwrap_or(0);
        let mut index: u128 = self
            .nextIndex
            .get()
            .try_into()
            .map_err(|_| MerkleErrors::TreeIsFull(TreeIsFull {}))?;
        if index >= (1u128 << depth_u32) {
            return Err(MerkleErrors::TreeIsFull(TreeIsFull {}));
        }

        self.leaves.insert(U256::from(index), leaf);

        let mut current = leaf;
        let mut idx = index;
        for level in 0..depth_u32 as usize {
            if idx % 2 == 0 {
                let right = self.zeros.get(level).unwrap();
                storagevec_set(&mut self.filledSubtrees, level, current);
                current = hash_pair(current, right);
            } else {
                let left = self.filledSubtrees.get(level).unwrap();
                current = hash_pair(left, current);
            }
            idx /= 2;
        }

        self.root.set(current);
        let ret = U256::from(index);
        self.nextIndex.set(U256::from(index + 1));
        Ok(ret)
    }

    pub fn root(&self) -> FixedBytes<32> {
        self.root.get()
    }

    pub fn size(&self) -> U256 {
        self.nextIndex.get()
    }

    pub fn verify(
        &self,
        leaf: FixedBytes<32>,
        index: U256,
        proof: Vec<FixedBytes<32>>,
        expected_root: FixedBytes<32>,
    ) -> bool {
        let mut computed = leaf;
        let mut idx: u128 = match index.try_into() {
            Ok(v) => v,
            Err(_) => return false,
        };

        for (lvl, sibling) in proof.iter().enumerate() {
            if idx % 2 == 0 {
                computed = hash_pair(computed, *sibling);
            } else {
                computed = hash_pair(*sibling, computed);
            }
            idx /= 2;
            if (lvl as u32) >= self.depth.get().try_into().unwrap_or(0) {
                return false;
            }
        }
        computed == expected_root
    }
}

impl MerkleTree {
    pub fn get_proof(&self, index: U256) -> Result<Vec<FixedBytes<32>>, MerkleErrors> {
        if !self.initialized.get() {
            return Err(MerkleErrors::NotInitialized(NotInitialized {}));
        }
        let depth_u32: u32 = self.depth.get().try_into().unwrap_or(0);
        let mut proof = Vec::with_capacity(depth_u32 as usize);
        let mut idx: u128 = index
            .try_into()
            .map_err(|_| MerkleErrors::IndexOutOfRange(IndexOutOfRange {}))?;
        let size: u128 = self.nextIndex.get().try_into().unwrap_or(0);
        if idx >= size {
            return Err(MerkleErrors::IndexOutOfRange(IndexOutOfRange {}));
        }

        let mut current_level_hashes: Vec<FixedBytes<32>> = Vec::with_capacity(size as usize);
        for i in 0..(1u128 << depth_u32) {
            if i < size {
                current_level_hashes.push(self.leaves.get(U256::from(i)));
            } else {
                current_level_hashes.push(self.zeros.get(0).unwrap());
            }
        }
        let mut cur_idx = idx as usize;
        for level in 0..depth_u32 as usize {
            let sibling_idx = if cur_idx % 2 == 0 {
                cur_idx + 1
            } else {
                cur_idx - 1
            };
            let sib = if sibling_idx < current_level_hashes.len() {
                current_level_hashes[sibling_idx]
            } else {
                self.zeros.get(level).unwrap()
            };
            proof.push(sib);

            let mut next_level: Vec<FixedBytes<32>> =
                Vec::with_capacity((current_level_hashes.len() + 1) / 2);
            let mut i = 0;
            while i < current_level_hashes.len() {
                let left = current_level_hashes[i];
                let right = if i + 1 < current_level_hashes.len() {
                    current_level_hashes[i + 1]
                } else {
                    self.zeros.get(level).unwrap()
                };
                next_level.push(hash_pair(left, right));
                i += 2;
            }
            current_level_hashes = next_level;
            cur_idx /= 2;
        }
        Ok(proof)
    }
}

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
    let out:U256 = hash.into_bigint().into();
    FixedBytes(out.to_be_bytes())
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

use alloy_sol_types::sol;
sol! {
    error InvalidDepth();
    error AlreadyInitialized();
    error NotInitialized();
    error TreeIsFull();
    error IndexOutOfRange();
}

#[derive(SolidityError)]
pub enum MerkleErrors {
    InvalidDepth(InvalidDepth),
    AlreadyInitialized(AlreadyInitialized),
    NotInitialized(NotInitialized),
    TreeIsFull(TreeIsFull),
    IndexOutOfRange(IndexOutOfRange),
}
