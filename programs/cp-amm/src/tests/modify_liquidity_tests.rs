use crate::{
    constants::{MAX_SQRT_PRICE, MIN_SQRT_PRICE},
    state::{Pool, Position},
    tests::LIQUIDITY_MAX,
    u128x128_math::Rounding,
};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10000, .. ProptestConfig::default()
    })]
    #[test]
    fn test_modify_liquidit_wont_loss(
        sqrt_price in MIN_SQRT_PRICE..=MAX_SQRT_PRICE,
        liquidity_delta in 1..=LIQUIDITY_MAX,
    ) {
        let mut pool = Pool {
            sqrt_price,
            sqrt_min_price: MIN_SQRT_PRICE,
            sqrt_max_price: MAX_SQRT_PRICE,
            ..Default::default()
        };

        let mut position = Position::default();

        let result_0 = pool
            .get_amounts_for_modify_liquidity(liquidity_delta, Rounding::Up)
            .unwrap();

        println!("result_0 {:?}", result_0);
        pool.apply_add_liquidity(&mut position, liquidity_delta).unwrap();


        let result_1 = pool.get_amounts_for_modify_liquidity(liquidity_delta, Rounding::Down).unwrap();
        println!("result_1 {:?}", result_0);

        pool.apply_remove_liquidity(&mut position, liquidity_delta).unwrap();

        assert_eq!(pool.liquidity, 0);
        assert_eq!(position.unlocked_liquidity, 0);

        assert!(result_0.amount_a >= result_1.amount_a);
        assert!(result_0.amount_b >= result_1.amount_b);
    }
}
