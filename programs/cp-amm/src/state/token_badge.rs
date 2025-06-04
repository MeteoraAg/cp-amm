use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;

#[account(zero_copy)]
#[derive(InitSpace, Debug)]
/// Parameter that set by the protocol
pub struct TokenBadge {
    /// token mint
    pub token_mint: Pubkey,
    /// immutable position owner
    pub immutable_position_owner: u8,
    /// Reserve
    pub _padding: [u8; 127],
}

const_assert_eq!(TokenBadge::INIT_SPACE, 160);

impl TokenBadge {
    pub fn initialize(&mut self, token_mint: Pubkey, immutable_position_owner: u8) -> Result<()> {
        self.token_mint = token_mint;
        self.immutable_position_owner = immutable_position_owner;
        Ok(())
    }
}
