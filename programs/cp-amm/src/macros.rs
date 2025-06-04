//! Macro functions
macro_rules! pool_authority_seeds {
    ($bump:expr) => {
        &[b"pool_authority".as_ref(), &[$bump]]
    };
}

macro_rules! position_nft_account_seeds {
    ($position_nft_mint:expr, $bump:expr) => {
        &[
            b"position_nft_account".as_ref(),
            $position_nft_mint.as_ref(),
            &[$bump],
        ]
    };
}
