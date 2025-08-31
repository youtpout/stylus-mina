use poseidon_mina::PoseidonContract;
use stylus_sdk::alloy_primitives::U256;

#[cfg(test)]
mod test {

    use alloy_primitives::Uint;

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

        // tree is empty
        assert_eq!(tree.size(), alloy_primitives::U256::ZERO);

        // Default
        let root = tree.root();
        println!("Root : {:?}", root);

        assert_eq!(root, alloy_primitives::FixedBytes::ZERO);
    }
}
