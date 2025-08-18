use poseidon_mina::PoseidonContract;
use stylus_sdk::{alloy_primitives::U256, prelude::*};

#[cfg(test)]
mod test {
    use stylus_sdk::console;

    use super::*;

    #[test]
    fn test_hash_list() {
        use stylus_sdk::testing::*;
        let vm = TestVM::default();
        let mut contract = PoseidonContract::from(&vm);
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
        let vec: Vec<U256> = vec![U256::from(3412), U256::from(548748548)];
        let result = contract.hash_function(vec);

        assert_eq!(
            U256::from_str_radix(
                "24245350037390325723675562428846509781869515058976947458013661211417354108422",
                10
            )
            .unwrap(),
            result
        );
    }
}
