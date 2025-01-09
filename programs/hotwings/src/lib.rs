use anchor_lang::prelude::*;

declare_id!("EBZJpxLE79aropXeAjtqbouWdF48iJGWFr89PoHSrXgs");

#[program]
pub mod hotwings {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
