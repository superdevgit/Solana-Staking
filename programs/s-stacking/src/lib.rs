use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_lang::solana_program::{clock};
use crate::constants::*;


declare_id!("AuCtWxqpFjNhHzNBJ831c7TEkwG2FQyB4G7hAsyatcMt");

mod constants {
    use anchor_lang::prelude::Pubkey;

    pub const DECIMAL: u64 = 1000000000;
    pub const BASE_REWARD: u32 = 5;
    pub const DAY_TIME: u32 = 86400;
    pub const STATISTIC_SEEDS: &str = "statistic";
    pub const POOL_SEEDS: &str = "pool";
    pub const POOL_DATA_SEEDS: &str = "pool data";

    pub const START_TIME: u32 = 1669852800;
    pub const DAYS: [u8;13] = [31, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

}

#[program]
pub mod s_stacking {
    use super::*;

    use anchor_lang::AccountsClose;
    
    pub fn initialize(ctx: Context<InitializeContext>, ADMIN_KEY: Pubkey, COLLECTION_KEY: Pubkey) -> Result<()> {
        let a_statistic = &mut ctx.accounts.statistic;
        a_statistic.staked_count = 0;
        a_statistic.staked_currency = 0;

        a_statistic.admin_key = ADMIN_KEY;
        a_statistic.collection_key = COLLECTION_KEY;
        Ok(())
    }

    pub fn stake(ctx: Context<StakeContext>) -> Result<()> {
        let a_user = &ctx.accounts.user;
        let a_statistic = &mut ctx.accounts.statistic;
        let a_pool = &mut ctx.accounts.pool;
        let a_pool_data = &mut ctx.accounts.pool_data;
        let a_mint = &ctx.accounts.mint;
        let a_token_program = &ctx.accounts.token_program;
        let a_token_from = &ctx.accounts.token_from;
        let a_token_to = &ctx.accounts.token_to;


        let m_data = &mut ctx.accounts.metadata.try_borrow_data()?;
        let metadata = mpl_token_metadata::state::Metadata::deserialize(&mut &m_data[..])?;

        let collection_not_proper = metadata
        .data
        .creators
        .as_ref()
        .unwrap()
        .iter()
        .filter(|item|{
            a_statistic.collection_key == item.address && item.verified
        })
        .count() == 0;

        require!(!collection_not_proper && metadata.mint == ctx.accounts.mint.key(), CustomError::InvalidNft);

        let clock = clock::Clock::get().unwrap();

        let cpi_ctx = CpiContext::new (
            a_token_program.to_account_info(),
            token::Transfer {
                from: a_token_from.to_account_info(),
                to: a_token_to.to_account_info(),
                authority: a_user.to_account_info()
            }
        );

        token::transfer(cpi_ctx, 1)?;

        a_statistic.staked_count += 1;

        a_pool.user = a_user.to_account_info().key();
        a_pool.staked_count += 1;

        a_pool_data.user = a_user.to_account_info().key();
        a_pool_data.mint = a_mint.to_account_info().key();
        a_pool_data.start_time = clock.unix_timestamp as u32;

        Ok(())
    }

    pub fn unstake(ctx: Context<UnstakeContext>) -> Result<()> {
        let a_user = &ctx.accounts.user;
        let a_statistic = &mut ctx.accounts.statistic;
        let a_pool = &mut ctx.accounts.pool;
        let a_pool_data = &mut ctx.accounts.pool_data;
        let a_token_from = &ctx.accounts.token_from;
        let a_token_to = &ctx.accounts.token_to;
        let a_token_program = &ctx.accounts.token_program;
        let clock = clock::Clock::get().unwrap();
        
        let (_pool, pool_bump) =
            Pubkey::find_program_address(&[
                POOL_SEEDS.as_ref(), 
                a_user.to_account_info().key.as_ref()
        ], ctx.program_id);

        let pool_seeds = &[
            POOL_SEEDS.as_ref(),
            a_user.to_account_info().key.as_ref(),    
            &[pool_bump],
        ];

        let pool_signer = &[&pool_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            a_token_program.to_account_info(),
            token::Transfer {
                from: a_token_from.to_account_info(),
                to: a_token_to.to_account_info(),
                authority: a_pool.to_account_info()
            },
            pool_signer
        );

        token::transfer(cpi_ctx, 1)?;

        a_statistic.staked_count -= 1;
        a_pool.staked_count -= 1 ;

        
        let mut start_time = a_pool_data.start_time;
        let current_time = clock.unix_timestamp as u32;
        let mut end_time = START_TIME;

        /*
        for i in 0..12 {
            end_time += DAY_TIME * DAYS[i] as u32;

            if start_time <= end_time {
                if end_time <= current_time {
                    a_pool.total_reward += (BASE_REWARD + i as u32) as u64 * (end_time - start_time) as u64 * DECIMAL / DAY_TIME as u64;
                    start_time = end_time;
                }
                else {
                    a_pool.total_reward += (BASE_REWARD + i as u32) as u64 * (current_time - start_time) as u64 * DECIMAL / DAY_TIME as u64;
                    break;
                }
            }
        }
        */

        a_pool_data.close(a_user.to_account_info())?;

        Ok(())
    }

    pub fn fund(ctx: Context<FundContext>, currency_amount: u64) -> Result<()> {
        let admin = &ctx.accounts.admin;
        let a_statistic = &mut ctx.accounts.statistic;
        let a_pool = &mut ctx.accounts.pool;
        let a_pool_data = &mut ctx.accounts.pool_data;
        let a_system_program = &ctx.accounts.system_program;
        let a_vault_from = &ctx.accounts.vault_from;
        let a_vault_to = &ctx.accounts.vault_to;

        let clock = clock::Clock::get().unwrap();

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &a_vault_from.key(),
            &a_vault_to.key(),
            currency_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                a_vault_from.to_account_info(),
                a_vault_from.to_account_info(),
            ],
        );

        a_statistic.staked_currency += currency_amount;

        a_pool.user = admin.to_account_info().key();
        a_pool.staked_currency += currency_amount;

        a_pool_data.user = admin.to_account_info().key();
        a_pool_data.start_time = clock.unix_timestamp as u32;

        Ok(())
    }

    pub fn unfund(ctx: Context<UnfundContext>, currency_amount: u64)-> Result<()> {
        let admin = &ctx.accounts.admin;
        let a_statistic = &mut ctx.accounts.statistic;
        let a_pool = &mut ctx.accounts.pool;
        let a_pool_data = &mut ctx.accounts.pool_data;
        let a_vault_from = &ctx.accounts.vault_from;
        let a_vault_to = &ctx.accounts.vault_to;
        let a_system_program = &ctx.accounts.system_program;
        let clock = clock::Clock::get().unwrap();
        
        let (_pool, pool_bump) =
            Pubkey::find_program_address(&[
                POOL_SEEDS.as_ref(), 
                admin.to_account_info().key.as_ref()
        ], ctx.program_id);

        let pool_seeds = &[
            POOL_SEEDS.as_ref(),
            admin.to_account_info().key.as_ref(),    
            &[pool_bump],
        ];

        let pool_signer = &[&pool_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            a_system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: a_vault_from.to_account_info(),
                to: a_vault_to.to_account_info(),
            },
            pool_signer
        );

        anchor_lang::system_program::transfer(cpi_ctx, currency_amount)?;

        a_statistic.staked_currency -= currency_amount;
        a_pool.staked_currency -= currency_amount ;

        
        let mut start_time = a_pool_data.start_time;
        let current_time = clock.unix_timestamp as u32;
        let mut end_time = START_TIME;

        /*
        for i in 0..12 {
            end_time += DAY_TIME * DAYS[i] as u32;

            if start_time <= end_time {
                if end_time <= current_time {
                    a_pool.total_reward += (BASE_REWARD + i as u32) as u64 * (end_time - start_time) as u64 * DECIMAL / DAY_TIME as u64;
                    start_time = end_time;
                }
                else {
                    a_pool.total_reward += (BASE_REWARD + i as u32) as u64 * (current_time - start_time) as u64 * DECIMAL / DAY_TIME as u64;
                    break;
                }
            }
        }
        */

        a_pool_data.close(admin.to_account_info())?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeContext<'info> {
    #[account(init, seeds = [STATISTIC_SEEDS.as_ref()], bump, payer = admin, space = 8 + 4)]
    pub statistic: Account<'info, Statistic>,
    #[account(mut, constraint = admin.key() == statistic.admin_key)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct StakeContext<'info> {
    #[account(mut)]
    pub statistic: Account<'info, Statistic>,
    #[account(init_if_needed, seeds = [POOL_SEEDS.as_ref(), user.key().as_ref()], bump, payer = user, space = 8 + 32 + 4 + 8 + 8)]
    pub pool: Account<'info, Pool>,
    #[account(init_if_needed, seeds = [POOL_DATA_SEEDS.as_ref(), user.key().as_ref(), mint.key().as_ref()], bump, payer = user, space = 8 + 32 + 32 + 4)]
    pub pool_data: Account<'info, PoolData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    /// CHECK: it's not dangerous
    pub metadata: AccountInfo<'info>, 
    #[account(mut, constraint = token_from.mint == mint.key() && token_from.owner == user.key() && token_from.amount == 1)]
    pub token_from: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = token_to.mint == mint.key() && token_to.owner == pool.key())]
    pub token_to: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct UnstakeContext<'info> {
    #[account(mut)]
    pub statistic: Account<'info, Statistic>,
    #[account(mut, constraint = pool.user == user.key())]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = pool_data.user == user.key() && pool_data.mint == mint.key())]
    pub pool_data: Account<'info, PoolData>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut, constraint = token_from.mint == mint.key() && token_from.owner == pool.key() && token_from.amount == 1)]
    pub token_from: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = token_to.mint == mint.key() && token_to.owner == user.key())]
    pub token_to: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct FundContext<'info> {
    #[account(mut)]
    pub statistic: Account<'info, Statistic>,
    #[account(init_if_needed, seeds = [POOL_SEEDS.as_ref(), admin.key().as_ref()], bump, payer = admin, space = 8 + 32 + 4 + 8 + 8)]
    pub pool: Account<'info, Pool>,
    #[account(init_if_needed, seeds = [POOL_DATA_SEEDS.as_ref(), admin.key().as_ref()], bump, payer = admin, space = 8 + 32 + 32 + 4)]
    pub pool_data: Account<'info, PoolData>,
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: it's not dangerous
    pub metadata: AccountInfo<'info>, 
    #[account(mut, constraint = vault_from.owner == admin.key())]
    pub vault_from: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = vault_to.owner == pool.key())]
    pub vault_to: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct UnfundContext<'info> {
    #[account(mut)]
    pub statistic: Account<'info, Statistic>,
    #[account(mut, constraint = pool.user == admin.key())]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = pool_data.user == admin.key())]
    pub pool_data: Account<'info, PoolData>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, constraint = vault_from.owner == pool.key())]
    pub vault_from: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = vault_to.owner == admin.key())]
    pub vault_to: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[account]
pub struct Statistic {
    pub admin_key: Pubkey,
    pub collection_key: Pubkey, 

    pub staked_count: u32,
    pub staked_currency: u64
}

#[account]
pub struct Pool {
    pub user: Pubkey,
    pub staked_count: u32,
    pub staked_currency: u64,
    pub total_reward: u64,
    pub transfer_amount: u64
}

#[account]
pub struct PoolData {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub start_time: u32,
}

#[error_code]
pub enum CustomError {
    #[msg("Invalid Nft.")]
    InvalidNft,
    #[msg("Transfer too much.")]
    TooMuchTransfer
}
