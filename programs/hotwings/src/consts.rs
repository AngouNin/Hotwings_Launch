use anchor_lang::prelude::Pubkey;
use std::str::FromStr;

pub const YOUR_PROJECT_WALLET: Pubkey = Pubkey::from_str("7LGEzNNoenXjSzebmjR7p538W3EbofJ6L7WTyoGFKDsf").unwrap();
pub const YOUR_MARKET_WALLET: Pubkey = Pubkey::from_str("Fn3Co7FJyMHM6RpPD74TX4Ah2ShLhyNHzNie19jNg8BG").unwrap();
pub const YOUR_BURN_WALLET: Pubkey = Pubkey::from_str("B1opJeR2emYp75spauVHkGXfyxkYSW7GZaN9B3XoUeGK").unwrap();
pub const MAX_HOLD_AMOUNT: u64 = 50_000_000;
pub const MAX_TOTAL_SUPPLY: u64 = 1_000_000_000;
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