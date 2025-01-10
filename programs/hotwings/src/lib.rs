use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

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

    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        let sender = &ctx.accounts.sender;
        let receiver = &ctx.accounts.receiver;

        let total_supply = 1_000_000_000;
        let max_hold = total_supply / 20;

        let rpc_url = String::from("https://api.devnet.solana.com");
        let connection = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let receiver_token_account = receiver.key(); 

        let receiver_balance = connection
            .get_token_account_balance(&receiver_token_account)
            .unwrap(); 

        require!(receiver_balance + amount <= max_hold, CustomError::MaxHoldExceeded);

        let tax = (amount as f64 * 0.015).round() as u64; 
        let to_burn = tax / 2; 
        let to_marketing = tax / 2; 

        let amount_after_tax = amount.checked_sub(tax).ok_or(CustomError::InsufficientFunds)?;
        
        token::transfer(
            ctx.accounts.transfer_context(amount_after_tax),
            amount_after_tax,
        )?;

        if to_burn > 0 {
            token::transfer(
                ctx.accounts.burn_context(to_burn),
                to_burn,
            )?;
        }

        if to_marketing > 0 {
            token::transfer(
                ctx.accounts.marketing_context(to_marketing),
                to_marketing,
            )?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub project_wallet: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub sender: Account<'info, TokenAccount>,
    #[account(mut)]
    pub receiver: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    
    #[account(mut)] 
    pub burn_wallet: Account<'info, TokenAccount>, 
    #[account(mut)] 
    pub marketing_wallet: Account<'info, TokenAccount>, 
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

#[error_code]
pub enum CustomError {
    #[msg("Insufficient funds in the sender's account.")]
    InsufficientFunds,
    #[msg("Max hold amount exceeded for the receiver's account.")]
    MaxHoldExceeded,
}