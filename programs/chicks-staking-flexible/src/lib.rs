///A Solana version of the xSushi contract for STEP
/// https://github.com/sushiswap/sushiswap/blob/master/contracts/SushiBar.sol
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::convert::TryInto;

#[cfg(not(feature = "local-testing"))]
declare_id!("6BWBw6SNMjYYQ2BB2BA8KxcZrifExt76MguDPg4ktdXW");
// declare_id!("XASp8U7ZSJ9sJfUaMKk5dxuw3Hf4xkLPBcoHZ4seoC1"); // for dev net

#[cfg(feature = "local-testing")]
declare_id!("XASp8U7ZSJ9sJfUaMKk5dxuw3Hf4xkLPBcoHZ4seoC1");

#[cfg(not(feature = "local-testing"))]
pub mod constants {
    pub const STEP_TOKEN_MINT_PUBKEY: &str = "cxxShYRVcepDudXhe7U62QHvw8uBJoKFifmzggGKVC2";
    // pub const STEP_TOKEN_MINT_PUBKEY: &str = "FUnRfJAJiTtpSGP9uP5RtFm4QPsYUPTVgSMoYrgVyNzQ"; // for dev net
    pub const STAKING_PDA_SEED: &[u8] = b"staking";
    pub const HODL_SERVICE_PUBKEY: &str = "7qLPnkAJneRFoVhW58UPGjySWyEE6VTz7gvvY6fDjPVA";
    // pub const HODL_SERVICE_PUBKEY: &str = "5hW2Y4KGNuo8Bh6ReD2D8VT3TTcBZSsxxJmnunvRmWc9"; // for dev net
}

#[cfg(feature = "local-testing")]
pub mod constants {
    pub const STEP_TOKEN_MINT_PUBKEY: &str = "teST1ieLrLdr4MJPZ7i8mgSCLQ7rTrPRjNnyFdHFaz9";
    pub const STAKING_PDA_SEED: &[u8] = b"staking";
    pub const HODL_SERVICE_PUBKEY: &str = "5hW2Y4KGNuo8Bh6ReD2D8VT3TTcBZSsxxJmnunvRmWc9";
}

#[program]
pub mod chicks_staking_flexible {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        _nonce_vault: u8,
        _nonce_staking: u8,
        lock_time: u64,
        fee_percent: u16
    ) -> Result<()> {
        ctx.accounts.staking_account.initializer_key = *ctx.accounts.initializer.key;
        ctx.accounts.staking_account.lock_time = lock_time;
        ctx.accounts.staking_account.fee_percent = fee_percent;

        Ok(())
    }

    pub fn update_lock_time(
        ctx: Context<UpdateStakingAccountField>,
        _nonce_staking: u8,
        new_lock_time: u64,
    ) -> Result<()> {
        ctx.accounts.staking_account.lock_time = new_lock_time;

        Ok(())
    }

    pub fn update_fee_percent(
        ctx: Context<UpdateStakingAccountField>,
        _nonce_staking: u8,
        new_fee_percent: u16,
    ) -> Result<()> {
        ctx.accounts.staking_account.fee_percent = new_fee_percent;

        Ok(())
    }

    pub fn toggle_freeze_program(ctx: Context<FreezeProgram>, _nonce_staking: u8) -> Result<()> {
        ctx.accounts.staking_account.freeze_program = !ctx.accounts.staking_account.freeze_program;

        Ok(())
    }

    pub fn stake(
        ctx: Context<Stake>,
        _nonce_vault: u8,
        _nonce_staking: u8,
        _nonce_user_staking: u8,
        handle: String,
        amount: u64,
    ) -> Result<()> {
        let total_token = ctx.accounts.token_vault.amount;
        let total_x_token = ctx.accounts.staking_account.total_x_token;
        let old_price = get_price(&ctx.accounts.token_vault, &ctx.accounts.staking_account);
        let now_ts = Clock::get().unwrap().unix_timestamp;
        ctx.accounts.user_staking_account.start_time = now_ts as u64;
        msg!("stake - handle {}", handle);
        //mint x tokens
        if total_token == 0 || total_x_token == 0 {
            ctx.accounts.staking_account.total_x_token =
                (ctx.accounts.staking_account.total_x_token as u128)
                    .checked_add(amount as u128)
                    .unwrap()
                    .try_into()
                    .unwrap();
            ctx.accounts.user_staking_account.x_token_amount =
                (ctx.accounts.user_staking_account.x_token_amount as u128)
                    .checked_add(amount as u128)
                    .unwrap()
                    .try_into()
                    .unwrap();
        } else {
            let what: u64 = (amount as u128)
                .checked_mul(total_x_token as u128)
                .unwrap()
                .checked_div(total_token as u128)
                .unwrap()
                .try_into()
                .unwrap();

            ctx.accounts.staking_account.total_x_token =
                (ctx.accounts.staking_account.total_x_token as u128)
                    .checked_add(what as u128)
                    .unwrap()
                    .try_into()
                    .unwrap();
            ctx.accounts.user_staking_account.x_token_amount =
                (ctx.accounts.user_staking_account.x_token_amount as u128)
                    .checked_add(what as u128)
                    .unwrap()
                    .try_into()
                    .unwrap();
        }

        //transfer the users tokens to the vault
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_from.to_account_info(),
                to: ctx.accounts.token_vault.to_account_info(),
                authority: ctx.accounts.token_from_authority.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount)?;

        (&mut ctx.accounts.token_vault).reload()?;

        //plus user staking amount
        ctx.accounts.user_staking_account.amount = (ctx.accounts.user_staking_account.amount
            as u128)
            .checked_add(amount as u128)
            .unwrap()
            .try_into()
            .unwrap();

        let new_price = get_price(&ctx.accounts.token_vault, &ctx.accounts.staking_account);

        emit!(PriceChange {
            old_step_per_xstep_e9: old_price.0,
            old_step_per_xstep: old_price.1,
            new_step_per_xstep_e9: new_price.0,
            new_step_per_xstep: new_price.1,
        });

        Ok(())
    }

    pub fn unstake(
        ctx: Context<Unstake>,
        nonce_vault: u8,
        _nonce_staking: u8,
        _nonce_user_staking: u8,
        handle: String,
        amount: u64,
    ) -> Result<()> {
        msg!("unstake - handle {}", handle);
        let now_ts = Clock::get().unwrap().unix_timestamp;
        let lock_time = ctx.accounts.staking_account.lock_time;
        let start_time = ctx.accounts.user_staking_account.start_time;

        let total_token = ctx.accounts.token_vault.amount;
        let total_x_token = ctx.accounts.staking_account.total_x_token;
        let old_price = get_price(&ctx.accounts.token_vault, &ctx.accounts.staking_account);

        //burn what is being sent
        ctx.accounts.staking_account.total_x_token = (ctx.accounts.staking_account.total_x_token
            as u128)
            .checked_sub(amount as u128)
            .unwrap()
            .try_into()
            .unwrap();
        ctx.accounts.user_staking_account.x_token_amount = 0;

        let mut what:u64 = (amount as u128)
            .checked_mul(total_token as u128)
            .unwrap()
            .checked_div(total_x_token as u128)
            .unwrap()
            .try_into()
            .unwrap();
        if (now_ts as u64) < (start_time + lock_time) {
            let fee_amount: u64 = (what as u128)
                .checked_mul(ctx.accounts.staking_account.fee_percent as u128)
                .unwrap()
                .checked_div(1000 as u128)
                .unwrap()
                .try_into()
                .unwrap();

            what = (what as u128).checked_sub(fee_amount as u128)
                .unwrap()
                .try_into()
                .unwrap();
        }
        //compute vault signer seeds
        let token_mint_key = ctx.accounts.token_mint.key();
        let seeds = &[token_mint_key.as_ref(), &[nonce_vault]];
        let signer = &[&seeds[..]];

        //transfer from vault to user
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_vault.to_account_info(),
                to: ctx.accounts.token_to.to_account_info(),
                authority: ctx.accounts.token_vault.to_account_info(),
            },
            signer,
        );
        token::transfer(cpi_ctx, what)?;

        (&mut ctx.accounts.token_vault).reload()?;

        ctx.accounts.user_staking_account.amount = 0;

        let new_price = get_price(&ctx.accounts.token_vault, &ctx.accounts.staking_account);

        emit!(PriceChange {
            old_step_per_xstep_e9: old_price.0,
            old_step_per_xstep: old_price.1,
            new_step_per_xstep_e9: new_price.0,
            new_step_per_xstep: new_price.1,
        });

        Ok(())
    }

    pub fn stake_by_service(
        ctx: Context<StakeByService>,
        _nonce_vault: u8,
        _nonce_staking: u8,
        _nonce_user_staking: u8,
        handle: String,
        amount: u64,
        start_time: u64,
    ) -> Result<()> {
        msg!("stake_by_service - handle {}", handle);
        let total_token = ctx.accounts.token_vault.amount;
        let total_x_token = ctx.accounts.staking_account.total_x_token;
        let old_price = get_price(&ctx.accounts.token_vault, &ctx.accounts.staking_account);
        // let now_ts = Clock::get().unwrap().unix_timestamp;
        ctx.accounts.user_staking_account.start_time = start_time as u64;

        //mint x tokens
        if total_token == 0 || total_x_token == 0 {
            ctx.accounts.staking_account.total_x_token =
                (ctx.accounts.staking_account.total_x_token as u128)
                    .checked_add(amount as u128)
                    .unwrap()
                    .try_into()
                    .unwrap();
            ctx.accounts.user_staking_account.x_token_amount =
                (ctx.accounts.user_staking_account.x_token_amount as u128)
                    .checked_add(amount as u128)
                    .unwrap()
                    .try_into()
                    .unwrap();
        } else {
            let what: u64 = (amount as u128)
                .checked_mul(total_x_token as u128)
                .unwrap()
                .checked_div(total_token as u128)
                .unwrap()
                .try_into()
                .unwrap();

            ctx.accounts.staking_account.total_x_token =
                (ctx.accounts.staking_account.total_x_token as u128)
                    .checked_add(what as u128)
                    .unwrap()
                    .try_into()
                    .unwrap();
            ctx.accounts.user_staking_account.x_token_amount =
                (ctx.accounts.user_staking_account.x_token_amount as u128)
                    .checked_add(what as u128)
                    .unwrap()
                    .try_into()
                    .unwrap();
        }

        //transfer the users tokens to the vault
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_from.to_account_info(),
                to: ctx.accounts.token_vault.to_account_info(),
                authority: ctx.accounts.token_from_authority.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, amount)?;

        (&mut ctx.accounts.token_vault).reload()?;

        //plus user staking amount
        ctx.accounts.user_staking_account.amount = (ctx.accounts.user_staking_account.amount
            as u128)
            .checked_add(amount as u128)
            .unwrap()
            .try_into()
            .unwrap();

        let new_price = get_price(&ctx.accounts.token_vault, &ctx.accounts.staking_account);

        emit!(PriceChange {
            old_step_per_xstep_e9: old_price.0,
            old_step_per_xstep: old_price.1,
            new_step_per_xstep_e9: new_price.0,
            new_step_per_xstep: new_price.1,
        });

        Ok(())
    }

    pub fn emit_price(ctx: Context<EmitPrice>) -> Result<()> {
        let price = get_price(&ctx.accounts.token_vault, &ctx.accounts.staking_account);
        emit!(Price {
            step_per_xstep_e9: price.0,
            step_per_xstep: price.1,
        });
        Ok(())
    }

    pub fn emit_reward(ctx: Context<EmitReward>) -> Result<()> {
        let total_token = ctx.accounts.token_vault.amount;
        let total_x_token = ctx.accounts.staking_account.total_x_token;
        let reward: u64 = (ctx.accounts.user_staking_account.x_token_amount as u128)
            .checked_mul(total_token as u128)
            .unwrap()
            .checked_div(total_x_token as u128)
            .unwrap()
            .checked_sub(ctx.accounts.user_staking_account.amount as u128)
            .unwrap()
            .try_into()
            .unwrap();
        emit!(Reward {
            deposit: ctx.accounts.user_staking_account.amount,
            reward: reward,
        });
        Ok(())
    }
}

const E9: u128 = 1_000_000_000;

pub fn get_price<'info>(
    vault: &Account<'info, TokenAccount>,
    staking: &Account<'info, StakingAccount>,
) -> (u64, String) {
    let total_token = vault.amount;
    let total_x_token = staking.total_x_token;

    if total_x_token == 0 {
        return (0, String::from("0"));
    }

    let price_uint = (total_token as u128)
        .checked_mul(E9 as u128)
        .unwrap()
        .checked_div(total_x_token as u128)
        .unwrap()
        .try_into()
        .unwrap();
    let price_float = (total_token as f64) / (total_x_token as f64);
    return (price_uint, price_float.to_string());
}

#[derive(Accounts)]
#[instruction(_nonce_vault: u8, _nonce_staking: u8)]
pub struct Initialize<'info> {
    #[account(
    address = constants::STEP_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
    init,
    payer = initializer,
    token::mint = token_mint,
    token::authority = token_vault, //the PDA address is both the vault account and the authority (and event the mint authority)
    seeds = [ constants::STEP_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap().as_ref() ],
    bump,
    )]
    ///the not-yet-created, derived token vault pubkey
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
    init,
    payer = initializer,
    seeds = [ constants::STAKING_PDA_SEED.as_ref() ],
    bump,
    space = 8 + STAKE_DATA_SIZE
    )]
    pub staking_account: Account<'info, StakingAccount>,

    #[account(mut)]
    ///pays rent on the initializing accounts
    pub initializer: Signer<'info>,

    ///used by anchor for init of the token
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(_nonce_staking: u8)]
pub struct UpdateStakingAccountField<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,

    #[account(
    mut,
    seeds = [ constants::STAKING_PDA_SEED.as_ref() ],
    bump,
    constraint = staking_account.initializer_key == *initializer.key,
    )]
    pub staking_account: Account<'info, StakingAccount>,
}

#[derive(Accounts)]
#[instruction(_nonce_staking: u8)]
pub struct FreezeProgram<'info> {
    pub initializer: Signer<'info>,

    #[account(
    mut,
    seeds = [ constants::STAKING_PDA_SEED.as_ref() ],
    bump,
    constraint = staking_account.initializer_key == *initializer.key,
    )]
    pub staking_account: Account<'info, StakingAccount>,
}

fn name_seed(name: &str) -> &[u8] {
    let b = name.as_bytes();
    if b.len() > 32 {
        &b[0..32]
    } else {
        b
    }
}

#[derive(Accounts)]
#[instruction(_nonce_vault: u8, _nonce_staking: u8, _nonce_user_staking: u8, handle: String)]
pub struct Stake<'info> {
    #[account(
    address = constants::STEP_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    //the token account to withdraw from
    pub token_from: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    //the authority allowed to transfer from token_from
    pub token_from_authority: Signer<'info>,

    #[account(
    mut,
    seeds = [ token_mint.key().as_ref() ],
    bump,
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
    mut,
    seeds = [ constants::STAKING_PDA_SEED.as_ref() ],
    bump,
    constraint = !staking_account.freeze_program,
    )]
    pub staking_account: Account<'info, StakingAccount>,

    #[account(
    init_if_needed,
    payer = token_from_authority,
    seeds = [ token_from_authority.key().as_ref(), name_seed(&handle) ],
    bump,
    space = 8 + USER_STAKE_DATA_SIZE
    )]
    pub user_staking_account: Account<'info, UserStakingAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(_nonce_vault: u8, _nonce_staking: u8, _nonce_user_staking: u8, handle: String)]
pub struct StakeByService<'info> {
    #[account(
    address = constants::STEP_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    //the token account to withdraw from
    pub token_from: Box<Account<'info, TokenAccount>>,

    //target_user_account
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub target_user_account: AccountInfo<'info>,

    //the authority allowed to transfer from token_from
    #[account(
    mut,
    address = constants::HODL_SERVICE_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub token_from_authority: Signer<'info>,

    #[account(
    mut,
    seeds = [ token_mint.key().as_ref() ],
    bump,
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
    mut,
    seeds = [ constants::STAKING_PDA_SEED.as_ref() ],
    bump,
    constraint = !staking_account.freeze_program,
    )]
    pub staking_account: Account<'info, StakingAccount>,

    #[account(
    init_if_needed,
    payer = token_from_authority,
    seeds = [ target_user_account.to_account_info().key.as_ref(), name_seed(&handle) ],
    bump,
    space = 8 + USER_STAKE_DATA_SIZE
    )]
    pub user_staking_account: Account<'info, UserStakingAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(nonce_vault: u8, _nonce_staking: u8, _nonce_user_staking: u8, handle: String, amount: u64)]
pub struct Unstake<'info> {
    #[account(
    address = constants::STEP_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub token_mint: Box<Account<'info, Mint>>,

    //the authority allowed to transfer from x_token_from
    pub x_token_from_authority: Signer<'info>,

    #[account(
    mut,
    seeds = [ token_mint.key().as_ref() ],
    bump,
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
    mut,
    seeds = [ constants::STAKING_PDA_SEED.as_ref() ],
    bump,
    constraint = !staking_account.freeze_program,
    )]
    pub staking_account: Account<'info, StakingAccount>,

    #[account(
    mut,
    seeds = [ x_token_from_authority.key().as_ref(), name_seed(&handle) ],
    bump,
    constraint = user_staking_account.x_token_amount == amount,
    close=x_token_from_authority
    )]
    pub user_staking_account: Account<'info, UserStakingAccount>,

    #[account(mut)]
    //the token account to send token
    pub token_to: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EmitPrice<'info> {
    #[account(
    address = constants::STEP_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
    seeds = [ token_mint.key().as_ref() ],
    bump,
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
    seeds = [ constants::STAKING_PDA_SEED.as_ref() ],
    bump,
    )]
    pub staking_account: Account<'info, StakingAccount>,
}

#[derive(Accounts)]
pub struct EmitReward<'info> {
    #[account(
    address = constants::STEP_TOKEN_MINT_PUBKEY.parse::<Pubkey>().unwrap(),
    )]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
    seeds = [ token_mint.key().as_ref() ],
    bump,
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
    seeds = [ constants::STAKING_PDA_SEED.as_ref() ],
    bump,
    )]
    pub staking_account: Account<'info, StakingAccount>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_from_authority: AccountInfo<'info>,

    #[account(
    seeds = [ token_from_authority.key().as_ref() ],
    bump,
    )]
    pub user_staking_account: Account<'info, UserStakingAccount>,
}

pub const STAKE_DATA_SIZE : usize = 51; // 32 + 8 + 8 + 1 + 2;

#[account]
#[derive(Default)]
pub struct StakingAccount {
    pub initializer_key: Pubkey,
    pub lock_time: u64,
    pub total_x_token: u64,
    pub freeze_program: bool,
    pub fee_percent: u16,
}

pub const USER_STAKE_DATA_SIZE : usize = 24; // 8 + 8

#[account]
#[derive(Default)]
pub struct UserStakingAccount {
    pub amount: u64,
    pub start_time: u64,
    pub x_token_amount: u64,
}

#[event]
pub struct PriceChange {
    pub old_step_per_xstep_e9: u64,
    pub old_step_per_xstep: String,
    pub new_step_per_xstep_e9: u64,
    pub new_step_per_xstep: String,
}

#[event]
pub struct Price {
    pub step_per_xstep_e9: u64,
    pub step_per_xstep: String,
}

#[event]
pub struct Reward {
    pub deposit: u64,
    pub reward: u64,
}

#[error]
pub enum ErrorCode {
    #[msg("Not exceed lock end date")]
    NotExceedLockEndDate,
}
