use poseidon_mina::PoseidonContract;
use stylus_sdk::alloy_primitives::U256;

#[cfg(test)]
mod test {

    use std::str::FromStr;

    use alloy_primitives::{FixedBytes, Uint};

    use super::*;

    #[test]
    fn test_hash_list() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let contract = PoseidonContract::from(&vm);
        let vec: Vec<U256> = vec![U256::from(3412), U256::from(548748548)];
        let result = contract.hash(vec);

        assert_eq!(
            U256::from_str_radix(
                "24245350037390325723675562428846509781869515058976947458013661211417354108422",
                10
            )
            .unwrap(),
            result
        );
    }

    #[test]
    fn test_hash_function() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = PoseidonContract::from(&vm);
        let vec: Vec<U256> = vec![U256::from(12)];
        let result = contract.hash_function(vec);

        assert_eq!(
            U256::from_str_radix(
                "20307190475163560179843878304233687113040243867319358507811895775846718326775",
                10
            )
            .unwrap(),
            result
        );
    }

    #[test]
    fn test_merkle_tree_default_root() {
        use stylus_sdk::testing::*;
        let host = TestVM::default();
        let mut tree = PoseidonContract::from(&host).merkle;

        // Depth 256
        let depth = alloy_primitives::U256::from(256u32);
        tree.init(depth);


        // Default
        let root = tree.get_root();
        println!("Root : {:?}", root);

        let default: FixedBytes<32> = alloy_primitives::FixedBytes::from_str(
            "0x32415c1274eff27e74c93b8b6b41ced16bd64621a9830fe6fac8c2c31cb91fc0",
        )
        .unwrap();

        assert_eq!(root, default);

        let intermediary: FixedBytes<32> = alloy_primitives::FixedBytes::from_str(
            "0x10fed4b36f9b7d58be9571a20f064288e5d5c9241fcb2b4a54f5900c88450741",
        )
        .unwrap();

        tree.set_leaf(
            alloy_primitives::U256::from(333u32),
            u32_to_fixedbytes32(1234u32),
        );
        let int_root = tree.get_root();

        assert_eq!(int_root, intermediary);

        tree.set_leaf(
            alloy_primitives::U256::from(55125u32),
            u32_to_fixedbytes32(88884u32),
        );
        let new_root = tree.get_root();
        let updated: FixedBytes<32> = alloy_primitives::FixedBytes::from_str(
            "0x2807b1d968f8ef5ba5a983bfe70a1a6270fd35a28445c2a71bbafd7b70008c6b",
        )
        .unwrap();

        assert_eq!(new_root, updated);
    }

    fn u32_to_fixedbytes32(x: u32) -> FixedBytes<32> {
        let mut bytes = [0u8; 32];
        // place the u32 at the end, big-endian
        bytes[28..].copy_from_slice(&x.to_be_bytes());
        FixedBytes(bytes)
    }
}
