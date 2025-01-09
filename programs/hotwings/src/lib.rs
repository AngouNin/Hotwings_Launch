use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("EBZJpxLE79aropXeAjtqbouWdF48iJGWFr89PoHSrXgs");

// Declare the Solana program module
#[program]
pub mod hotwings {
    use super::*;

    // Initialization function that mints the total supply of tokens to the specified project wallet.
    pub fn initialize(ctx: Context<Initialize>, total_supply: u64) -> Result<()> {
        // Prepare arguments to call the token program's mint_to function.
        let cpi_accounts = token::MintTo {
            // The Mint account where the tokens will be minted from.
            mint: ctx.accounts.mint.to_account_info(),
            // The TokenAccount where the minted tokens will be sent.
            to: ctx.accounts.project_wallet.to_account_info(),
            // The authority that is allowed to mint tokens.
            authority: ctx.accounts.authority.to_account_info(),
        };

        // Get the token program's account information.
        let cpi_program = ctx.accounts.token_program.to_account_info();

        // Perform the minting operation using a cross-program invocation (CPI).
        token::mint_to(CpiContext::new(cpi_program, cpi_accounts), total_supply)?;

        // Return Ok if the operation was successful.
        Ok(())
    }

    // Function to transfer tokens from one account to another, with potential anti-whale protection and transaction tax 
    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        // Logic to enforce transfer rules (anti-whale protection, transaction tax, etc.) will be implemented here.

        // Return Ok if the transfer rules are satisfied and the transfer operation is successful.
        Ok(())
    }
}

// Define the accounts context structure for the initialize function.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32)] // Initialize the mint account. `space` is used to allocate memory for the account.
    pub mint: Account<'info, Mint>, // Account where tokens will be minted from.

    #[account(mut)] // Mark the project wallet as mutable since it will receive minted tokens.
    pub project_wallet: Account<'info, TokenAccount>, // Token account where the minted tokens will be sent.

    pub authority: Signer<'info>, // The authority that initiates the minting process (must sign the transaction).

    pub token_program: Program<'info, Token>, // The associated token program, typically the SPL token program.
}

// Define the accounts context structure for the transfer function.
#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)] // Mark the sender's account as mutable since it will spend tokens.
    pub sender: Account<'info, TokenAccount>, // Token account from which tokens will be sent.

    #[account(mut)] // Mark the receiver's account as mutable since it will receive tokens.
    pub receiver: Account<'info, TokenAccount>, // Token account where the tokens will be received.

    pub token_program: Program<'info, Token>, // The associated token program, typically the SPL token program.
}