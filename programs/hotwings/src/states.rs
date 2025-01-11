use anchor_lang::prelude::*;
use crate::consts::{YOUR_PROJECT_WALLET, YOUR_MARKET_WALLET, YOUR_BURN_WALLET};
use anchor_spl::token::{self, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(address = Your_Project_Wallet)]
    pub project_wallet: Account<'info, TokenAccount>,
    #[account(address = Your_Market_Wallet)]
    pub marketing_wallet: Account<'info, TokenAccount>,
    #[account(address = Your_Burn_Wallet)]
    pub user_wallet: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeLockedTokens<'info> {
    #[account(init, payer = authority, space = LockedTokens::LEN)]
    pub locked_tokens: Account<'info, LockedTokens>,
    pub authority: Signer<'info>, // Project wallet authority
    pub token_program: Program<'info, Token>, // Token program
    pub system_program: Program<'info, System>, // System program
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub sender: Account<'info, TokenAccount>, // Sender of the tokens
    #[account(mut)]
    pub receiver: Account<'info, TokenAccount>, // Receiver of the tokens
    pub token_program: Program<'info, Token>, // Token program

    #[account(mut)] 
    pub burn_wallet: Account<'info, TokenAccount>, // To burn tax
    #[account(mut)] 
    pub marketing_wallet: Account<'info, TokenAccount>, // To send marketing tax
    #[account(mut)] 
    pub project_wallet: Account<'info, TokenAccount>, // To check for taxable buy/sell interactions
}

impl<'info> Transfer<'info> {
    fn transfer_context(&self, amount: u64) -> CpiContext<'_, '_, '_, 'info, token::Transfer<'info>> {
        let cpi_accounts = token::Transfer {
            from: self.sender.to_account_info(),
            to: self.receiver.to_account_info(),
            authority: self.sender.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn burn_context(&self, amount: u64) -> CpiContext<'_, '_, '_, 'info, token::Transfer<'info>> {
        let cpi_accounts = token::Transfer {
            from: self.sender.to_account_info(),
            to: self.burn_wallet.to_account_info(),
            authority: self.sender.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn marketing_context(&self, amount: u64) -> CpiContext<'_, '_, '_, 'info, token::Transfer<'info>> {
        let cpi_accounts = token::Transfer {
            from: self.sender.to_account_info(),
            to: self.marketing_wallet.to_account_info(),
            authority: self.sender.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct UserTokenLock {
    pub user: Pubkey,         // User's wallet address
    pub total_locked: u64,    // Total amount of tokens locked for this user
    pub unlocked_amount: u64,  // Amount of tokens that have been unlocked
}

#[account]
pub struct LockedTokens {
    pub total_locked: u64,                     // Total tokens locked in the contract
    pub user_locks: Vec<UserTokenLock>,        // Store user locks
    pub distribution_timestamp: i64,            // Timestamp of distribution 
    pub has_full_unlocked: bool,                // Flag for if full unlock has occurred
    pub total_supply: u64, // Add to track total supply
}

impl LockedTokens {
    pub const LEN: usize = 8 + 4800 + 8 + 1 + 8 + 32; // Total space estimate
}

pub const MARKET_CAP_MILESTONES: [(u64, u64); 8] = [
    (45000, 10),
    (105500, 20),
    (225000, 30),
    (395000, 40),
    (650000, 50),
    (997000, 60),
    (1574000, 70),
    (2500000, 100),
];

#[derive(Accounts)]
pub struct PurchaseTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // User purchasing tokens
    #[account(mut)]
    pub locked_tokens: Account<'info, LockedTokens>, // Account holding all user locks
    #[account(mut)]
    pub project_wallet: AccountInfo<'info>, // Project wallet for restriction
    #[account(mut)]
    pub marketing_wallet: AccountInfo<'info>, // Marketing wallet for restriction
}

#[derive(Accounts)]
pub struct UnlockTokens<'info> {
    #[account(mut)]
    pub locked_tokens: Account<'info, LockedTokens>, // Account holding all user locks
    pub token_program: Program<'info, Token>, // Token program for transfers
    #[account(mut)]
    pub user_wallet: AccountInfo<'info>, // Each user's wallet
}

#[derive(Accounts)]
pub struct UnlockFull<'info> {
    #[account(mut)]
    pub locked_tokens: Account<'info, LockedTokens>, // Account holding all user locks
    pub token_program: Program<'info, Token>, // Token program for transfers
    #[account(mut)]
    pub user_wallet: AccountInfo<'info>, // Each user's wallet, similar to previous transfers
}

