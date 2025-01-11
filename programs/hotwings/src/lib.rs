use anchor_lang::prelude::*;
use crate::consts::MAX_HOLD_AMOUNT;
use crate::errors::CustomError;
use crate::state::*;

pub mod consts;
pub mod errors;
pub mod states;

declare_id!("EBZJpxLE79aropXeAjtqbouWdF48iJGWFr89PoHSrXgs");

#[program]
pub mod hotwings {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, total_supply: u64) -> Result<()> {
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.project_wallet.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        token::mint_to(CpiContext::new(cpi_program, cpi_accounts), total_supply)?;
        
        Ok(())
    }

    pub fn initialize_locked_tokens(ctx: Context<InitializeLockedTokens>) -> Result<()> {
        let locked_tokens = &mut ctx.accounts.locked_tokens;
    
        // Initialize the LockedTokens account with predefined values
        locked_tokens.total_locked = 0;
        locked_tokens.user_locks = vec![]; // Initialize the user locks
        locked_tokens.distribution_timestamp = Clock::get()?.unix_timestamp; // Set current time
        locked_tokens.has_full_unlocked = false; // Initially set to false
    
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        let sender = &ctx.accounts.sender;
        let receiver = &ctx.accounts.receiver;
    
        // Check if the transaction involves the project or marketing wallet
        let is_taxable = 
            sender.key() == ctx.accounts.project_wallet.key() || 
            receiver.key() == ctx.accounts.project_wallet.key() || 
            sender.key() == ctx.accounts.marketing_wallet.key() || 
            receiver.key() == ctx.accounts.marketing_wallet.key();
    
        if is_taxable {
            // Apply tax logic only for these cases (e.g. buy or sell transactions).
            let tax = (amount as f64 * 0.015).round() as u64;
            let to_burn = tax / 2;
            let to_marketing = tax / 2;
    
            // Calculate amount after tax
            let amount_after_tax = amount.checked_sub(tax).ok_or(CustomError::InsufficientFunds)?;
    
            // Transfer the taxed amount
            token::transfer(ctx.accounts.transfer_context(amount_after_tax), amount_after_tax)?;
    
            // Burn 0.75% (half of the tax)
            if to_burn > 0 {
                token::transfer(ctx.accounts.burn_context(to_burn), to_burn)?;
            }
    
            // Send 0.75% (other half) to the marketing wallet
            if to_marketing > 0 {
                token::transfer(ctx.accounts.marketing_context(to_marketing), to_marketing)?;
            }
        } else {
            // Standard user-to-user transfer with no tax
            token::transfer(ctx.accounts.transfer_context(amount), amount)?;
        }
    
        Ok(())
    }

    pub fn purchase_tokens(ctx: Context<PurchaseTokens>, amount: u64) -> Result<()> {
        let locked_tokens = &mut ctx.accounts.locked_tokens;
    
        let user = &ctx.accounts.user.key();

        let mut total_balance = 0u64;

        let mut user_found = false;
        for user_lock in locked_tokens.user_locks.iter_mut() {
            if user_lock.user == *user {
                user_found = true;
                break;
            }
        }
    
        // If user is not found, create a new entry
        if user_found {
            for user_lock in locked_tokens.user_locks.iter_mut() {
                if user_lock.user == *user {
                    total_balance = user_lock.total_locked + user_lock.unlocked_amount;
                    if !locked_tokens.has_full_unlocked {
                        if (total_balance + amount) > MAX_HOLD_AMOUNT &&
                           ctx.accounts.user.key() != &ctx.accounts.project_wallet.key() &&
                           ctx.accounts.user.key() != &ctx.accounts.marketing_wallet.key() {
                            return Err(CustomError::MaxHoldExceeded);
                        }
                        else {
                            user_lock.total_locked += amount; // Update locked amount
                        }
                    }
                    else {
                        user_lock.total_locked += amount; // Update locked amount
                    }
                    
                    break;
                };
            }
        }
        else {
            locked_tokens.user_locks.push(UserTokenLock {
                user: *user,
                total_locked: amount,
                unlocked_amount: 0,
            });
        };

        // Update the total locked tokens
        locked_tokens.total_locked += amount;
    
        Ok(())
    }

    pub fn unlock_tokens(ctx: Context<UnlockTokens>, current_market_cap: u64) -> Result<()> {
        let locked_tokens = &mut ctx.accounts.locked_tokens;
    
        // Determine the percentage to unlock based on milestones
        let mut unlock_percentage = 0;
    
        for (milestone, percentage) in MARKET_CAP_MILESTONES.iter() {
            if current_market_cap >= *milestone {
                unlock_percentage = *percentage;
            } else {
                break; // Exit the loop since milestones are ordered
            }
        }
    
        // Distribute unlocked tokens to all users
        for user_lock in &mut locked_tokens.user_locks {
            // Calculate the amount to unlock for this user
            let unlock_amount = (user_lock.total_locked * unlock_percentage) / 100;
    
            if unlock_amount > 0 {
                // Adjust unlocked amounts and total locked
                user_lock.unlocked_amount += unlock_amount;
                user_lock.total_locked -= unlock_amount;
                locked_tokens.total_locked -= unlock_amount;
    
                // Transfer unlocked tokens to users' wallets
                let transfer_accounts = token::Transfer {
                    from: ctx.accounts.locked_tokens.to_account_info(),
                    to: ctx.accounts.user_wallet.to_account_info(), // User's wallet
                    authority: ctx.accounts.locked_tokens.to_account_info(),
                };
    
                // Transfer tokens using CPI
                token::transfer(
                    CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_accounts),
                    unlock_amount,
                )?;
            }
        }
    
        // Check for full unlock condition after market cap milestone reached
        if current_market_cap >= 2500000 {
            locked_tokens.has_full_unlocked = true; // Set flag if all tokens are unlocked
        }

        Ok(())
    }

    pub fn check_full_unlock(ctx: Context<UnlockFull>, current_timestamp: i64) -> Result<()> {
        let locked_tokens = &mut ctx.accounts.locked_tokens;
    
        // Check if 3 months have passed since distribution timestamp
        if locked_tokens.total_locked > 0 && (current_timestamp - locked_tokens.distribution_timestamp) >= 60 * 60 * 24 * 90 {
            // Iterate through user locks to unlock all remaining tokens
            for user_lock in &mut locked_tokens.user_locks {
                if user_lock.total_locked > 0 {
                    // Transfer all remaining locked tokens
                    let transfer_accounts = token::Transfer {
                        from: ctx.accounts.locked_tokens.to_account_info(),
                        to: ctx.accounts.user_wallet.to_account_info(), // User's wallet
                        authority: ctx.accounts.locked_tokens.to_account_info(),
                    };
    
                    token::transfer(
                        CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_accounts),
                        user_lock.total_locked,
                    )?;
    
                    // Update the user's unlocked amount
                    user_lock.unlocked_amount += user_lock.total_locked; 
                    locked_tokens.total_locked -= user_lock.total_locked; // Clear locked amount
                    user_lock.total_locked = 0; // Reset locked amount for this user
                }
            }
            locked_tokens.has_full_unlocked = true; // Set flag to show full unlock has occurred
        }

        if locked_tokens.total_locked == 0 {
            locked_tokens.has_full_unlocked = true;
        }
    
        Ok(())
    }
    
}
