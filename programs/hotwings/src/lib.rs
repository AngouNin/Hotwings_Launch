use anchor_lang::prelude::*;
use anchor_spl::token;
use crate::consts::*;
// use crate::consts::MAX_HOLD_AMOUNT;
// use crate::consts::MAX_TOTAL_SUPPLY;
// use crate::consts::MARKET_CAP_MILESTONES;
use crate::errors::CustomError;
use crate::states::*;

pub mod consts;
pub mod errors;
pub mod states;

declare_id!("EBZJpxLE79aropXeAjtqbouWdF48iJGWFr89PoHSrXgs");

#[program]
pub mod hotwings {
    use super::*;

    /// Initialize the token mint and distribution wallets.
    pub fn initialize(ctx: Context<Initialize>, total_supply: u64) -> Result<()> {
        // Ensure only the authorized wallet can initialize
        require!(
            ctx.accounts.authority.key() == YOUR_PROJECT_WALLET,
            CustomError::Unauthorized
        );
        // Check if the requested total supply is within limits
        require!(total_supply <= MAX_TOTAL_SUPPLY, CustomError::SupplyExceeded);

        // Mint initial supply to the project's wallet
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: YOUR_PROJECT_WALLET.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::mint_to(CpiContext::new(cpi_program, cpi_accounts), total_supply)?;

        Ok(())
    }

    /// Initialize the `LockedTokens` account.
    pub fn initialize_locked_tokens(ctx: Context<InitializeLockedTokens>) -> Result<()> {
        let locked_tokens = &mut ctx.accounts.locked_tokens;

        // Set default values for the new locked tokens account
        locked_tokens.total_locked = 0;
        locked_tokens.user_locks = Vec::new();
        locked_tokens.distribution_timestamp = Clock::get()?.unix_timestamp;
        locked_tokens.has_full_unlocked = false;

        Ok(())
    }

    /// Transfer tokens between users, with optional taxation logic.
    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {

        // Ensure the authority has the correct permissions for specific wallets
        let is_authorized = ctx.accounts.sender.key() == YOUR_PROJECT_WALLET
        || ctx.accounts.sender.key() == YOUR_MARKET_WALLET
        || ctx.accounts.sender.key() == YOUR_BURN_WALLET;

        require!(is_authorized, CustomError::Unauthorized);
        
        let sender = &ctx.accounts.sender;
        let receiver = &ctx.accounts.receiver;

        // Check if either wallet is involved in project operations
        let is_taxable = sender.key() == YOUR_PROJECT_WALLET ||
                         receiver.key() == YOUR_PROJECT_WALLET ||
                         sender.key() == YOUR_MARKET_WALLET ||
                         receiver.key() == YOUR_MARKET_WALLET;

        if is_taxable {
            // Tax logic applies to project/marketing wallet interactions
            let tax = (amount as f64 * 0.015).round() as u64;
            let amount_after_tax = amount.checked_sub(tax).ok_or(CustomError::InsufficientFunds)?;

            // Burn 0.75% of the transaction amount
            let to_burn = tax / 2;
            if to_burn > 0 {
                let burn_accounts = token::Transfer {
                    from: sender.to_account_info(),
                    to: YOUR_BURN_WALLET.to_account_info(),
                    authority: sender.to_account_info(),
                };
                token::transfer(
                    CpiContext::new(ctx.accounts.token_program.to_account_info(), burn_accounts),
                    to_burn,
                )?;
            }

            // Transfer 0.75% to the marketing wallet
            let to_marketing = tax - to_burn; // Remaining tax
            if to_marketing > 0 {
                let marketing_accounts = token::Transfer {
                    from: sender.to_account_info(),
                    to: YOUR_MARKET_WALLET.to_account_info(),
                    authority: sender.to_account_info(),
                };
                token::transfer(CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts), to_marketing)?;
            }

            // Transfer the amount after tax
            let cpi_accounts = token::Transfer {
                from: sender.to_account_info(),
                to: receiver.to_account_info(),
                authority: sender.to_account_info(),
            };
            token::transfer(CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts), amount_after_tax)?;
        } else {
            // Standard user-to-user transfer (no tax)
            let cpi_accounts = token::Transfer {
                from: sender.to_account_info(),
                to: receiver.to_account_info(),
                authority: sender.to_account_info(),
            };
            token::transfer(CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts), amount)?;
        }

        Ok(())
    }

    /// Purchase tokens and lock them if applicable.
    pub fn purchase_tokens(ctx: Context<PurchaseTokens>, amount: u64) -> Result<()> {
        
        let locked_tokens = &mut ctx.accounts.locked_tokens;
        let user = ctx.accounts.user.key();

        // Enforce max hold restriction for non-exempt users
        let mut total_balance = 0u64;
        if let Some(user_lock) = locked_tokens.user_locks.iter().find(|lock| lock.user == user) {
            total_balance = user_lock.total_locked + user_lock.unlocked_amount;
        }
        if total_balance + amount > MAX_HOLD_AMOUNT &&
           ctx.accounts.user.key() != YOUR_PROJECT_WALLET &&
           ctx.accounts.user.key() != YOUR_MARKET_WALLET() {
            return Err(CustomError::MaxHoldExceeded.into());
        }

        // Update user's lock entry or create one if new
        if let Some(user_lock) = locked_tokens.user_locks.iter_mut().find(|lock| lock.user == user) {
            user_lock.total_locked += amount;
        } else {
            locked_tokens.user_locks.push(UserTokenLock {
                user,
                total_locked: amount,
                unlocked_amount: 0,
            });
        }

        locked_tokens.total_locked += amount;

        Ok(())
    }

    /// Unlock remaining tokens if 3 months have passed since distribution timestamp
    pub fn check_full_unlock(ctx: Context<UnlockFull>) -> Result<()> {
        // Only allow the project wallet to initiate the unlock process
        require!(
            ctx.accounts.authority.key() == YOUR_PROJECT_WALLET,
            CustomError::Unauthorized
        );

        let locked_tokens = &mut ctx.accounts.locked_tokens;

        // Check if 3 months (90 days) have passed since the distribution timestamp
        let now = Clock::get()?.unix_timestamp;
        require!(
            now - locked_tokens.distribution_timestamp >= 90 * 24 * 60 * 60,
            CustomError::ThreeMonthsNotPassed
        );

        // Unlock all remaining tokens for all users
        for user_lock in locked_tokens.user_locks.iter_mut() {
            if user_lock.total_locked > 0 {
                let transfer_accounts = token::Transfer {
                    from: ctx.accounts.locked_tokens.to_account_info(),
                    to: ctx.accounts.user_wallet.to_account_info(),
                    authority: ctx.accounts.locked_tokens.to_account_info(),
                };

                token::transfer(
                    CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_accounts),
                    user_lock.total_locked,
                )?;

                // Update state
                user_lock.unlocked_amount += user_lock.total_locked;
                locked_tokens.total_locked -= user_lock.total_locked;
                user_lock.total_locked = 0;
            }
        }

        // Mark all tokens as fully unlocked
        locked_tokens.has_full_unlocked = true;

        Ok(())
    }

    /// Unlock tokens for all users based on market cap milestones
    pub fn unlock_tokens(ctx: Context<UnlockTokens>, current_market_cap: u64) -> Result<()> {

        // Only allow the project wallet to initiate the unlock process
        require!(
            ctx.accounts.authority.key() == YOUR_PROJECT_WALLET,
            CustomError::Unauthorized
        );

        let locked_tokens = &mut ctx.accounts.locked_tokens;
        let mut unlock_percentage = 0;

        // Calculate percentage to unlock based on market cap
        MARKET_CAP_MILESTONES.iter().for_each(|(milestone, percentage)| {
            if current_market_cap >= *milestone {
                unlock_percentage = *percentage;
            }
        });

        for lock in locked_tokens.user_locks.iter_mut() {
            let unlock_amount = (lock.total_locked * unlock_percentage) / 100;

            if unlock_amount > 0 {
                // Unlock tokens to user's wallet
                let transfer_accounts = token::Transfer {
                    from: ctx.accounts.locked_tokens.to_account_info(),
                    to: ctx.accounts.user_wallet.to_account_info(),
                    authority: ctx.accounts.locked_tokens.to_account_info(),
                };
                token::transfer(
                    CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_accounts),
                    unlock_amount,
                )?;

                lock.unlocked_amount += unlock_amount;
                lock.total_locked -= unlock_amount;
                locked_tokens.total_locked -= unlock_amount;
            }
        }

        if current_market_cap >= 2_500_000 {
            locked_tokens.has_full_unlocked = true;
        }

        Ok(())
    }
    
}
