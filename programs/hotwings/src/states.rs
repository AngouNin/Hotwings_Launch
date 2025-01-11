use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeLockedTokens<'info> {
    #[account(init, payer = authority, space = LockedTokens::LEN)]
    pub locked_tokens: Account<'info, LockedTokens>,
    #[account(mut)]
    pub authority: Signer<'info>,  
    pub system_program: Program<'info, System>,  
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub sender: Account<'info, TokenAccount>,  
    #[account(mut)]
    pub receiver: Account<'info, TokenAccount>,  
    pub token_program: Program<'info, Token>,  
}

#[account]
pub struct LockedTokens {
    pub total_locked: u64,                      
    pub user_locks: Vec<UserTokenLock>,         
    pub distribution_timestamp: i64,             
    pub has_full_unlocked: bool,                
}

impl LockedTokens {
    pub const LEN: usize = 8 + 48 + 8 + 1 + 8 + (1000 * UserTokenLock::LEN);
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct UserTokenLock {
    pub user: Pubkey,          
    pub total_locked: u64,     
    pub unlocked_amount: u64,   
}

impl UserTokenLock {
    pub const LEN: usize = 32 + 8 + 8;
}

#[derive(Accounts)]
pub struct PurchaseTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>, 
    #[account(mut)]
    pub locked_tokens: Account<'info, LockedTokens>,  
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>, 
    pub token_program: Program<'info, Token>,  
}

#[derive(Accounts)]
pub struct UnlockTokens<'info> {
    #[account(mut)]
    pub locked_tokens: Account<'info, LockedTokens>, 
    pub token_program: Program<'info, Token>,  
    #[account(mut)]
    pub user_wallet: AccountInfo<'info>,  
    #[account(mut)]
    pub authority: Signer<'info>,  
}

#[derive(Accounts)]
pub struct UnlockFull<'info> {
    #[account(mut)]
    pub locked_tokens: Account<'info, LockedTokens>, 
    #[account(mut)]
    pub user_wallet: AccountInfo<'info>,  
    pub token_program: Program<'info, Token>,  
    #[account(mut)]
    pub authority: Signer<'info>, 
}

