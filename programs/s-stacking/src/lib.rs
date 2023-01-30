use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_lang::solana_program::{clock};
use crate::constants::*;


declare_id!("3LnWTYwD16Dh4Ly6RZiJxSsMP8HWbKoGtZGi1MmJ9Dub");

mod constants {
    use anchor_lang::prelude::Pubkey;

    pub const STATISTIC_SEEDS: &str = "statistic";
    pub const POOL_SEEDS: &str = "pool";
    pub const POOL_DATA_SEEDS: &str = "pool data";
    pub const FUND_SEED: &str = "fund_data_seed";
    pub const SECONDS_PER_DATE: u32 = 86400;
    pub const ADMIN_KEY: Pubkey = anchor_lang::solana_program::pubkey!("GQXMX2RVvuppFs2owKysJsuS686vNZpBusdgynZV86LS");
}

#[program]
pub mod staking_test {
    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
        let a_statistic = &mut ctx.accounts.statistic;
        a_statistic.staked_count = 0;
        a_statistic.currency_count = 0;
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
        a_pool_data.distribute_id = 0;

        let clock = clock::Clock::get().unwrap();
        a_pool_data.start_time = clock.unix_timestamp as u32;  /*1671300000;*/

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

        Ok(())
    }

    pub fn fund(ctx: Context<FundContext> , amount : u64 ) -> Result<()> {
       let a_admin = &mut ctx.accounts.admin;
       let a_fund_pool =&mut  ctx.accounts.fund_pool;
       
       let a_statistic = &mut ctx.accounts.statistic;

       let ix = anchor_lang::solana_program::system_instruction::transfer(
           &a_admin.key(),  &a_fund_pool.key(), amount
       );

       anchor_lang::solana_program::program::invoke(
           &ix,&[a_admin.to_account_info(), a_fund_pool.to_account_info() ]
       );

       a_statistic.currency_count += amount;
       // a_fund_pool.price_amount += amount;
        
        Ok(())
    }

    pub fn refund(ctx: Context<ReFundContext> , amount : u64 ) -> Result<()> {
        let a_statistic = &mut ctx.accounts.statistic;
        **ctx.accounts.fund_pool.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.admin.try_borrow_mut_lamports()? += amount;

        a_statistic.currency_count -= amount;
        // require(amount > a_fund_pool.price_amount )
        // let a_admin = &mut ctx.accounts.admin;
        // let a_fund_pool = &mut ctx.accounts.fund_pool;
        // let a_system_program = &ctx.accounts.system_program;
        // let (_pool, pool_bump) = Pubkey::find_program_address(
        //     &[FUND_SEED.as_ref(), a_admin.to_account_info().key.as_ref()], ctx.program_id
        // );

        // let pool_seeds = &[FUND_SEED.as_ref(), a_admin.to_account_info().key.as_ref(), &[pool_bump]];

        // let pool_signer = &[&pool_seeds[..]];

        // let ix = anchor_lang::solana_program::system_instruction::transfer(
        //    &a_fund_pool.key(),  &a_admin.key(), amount
        // );

        // anchor_lang::solana_program::program::invoke_signed(
        //     &ix,&[
        //         a_fund_pool.to_account_info().clone(), 
        //         a_admin.to_account_info().clone(),
        //         ctx.accounts.system_program.to_account_info().clone()
        //          ],pool_signer
        // )?;

        // a_fund_pool.price_amount -= amount;

        // let cpi_ctx = CpiContext::new_with_signer(
        //     a_system_program.to_account_info(),
        //     anchor_lang::system_program::Transfer {
        //         from: a_fund_pool.to_account_info(),
        //         to: a_admin.to_account_info(),
        //     },
        //     pool_signer
        // );

        // anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn distribute(ctx: Context<DistributeContext>, index: u32, nft_count: u32 ) -> Result<()> {
        let admin = &mut ctx.accounts.admin;
        let statistic = &mut ctx.accounts.statistic;
        let a_distribute = &mut ctx.accounts.distribute_data;

        let clock = clock::Clock::get().unwrap();
        a_distribute.reward_id = index;
        a_distribute.start_time = clock.unix_timestamp as u32;
        a_distribute.rewards_amount = statistic.currency_count / nft_count as u64;
        Ok(())
    }

    pub fn claim(ctx: Context<ClaimContext>) -> Result<()>{
        let user = &ctx.accounts.user;
        let pool_data = &mut ctx.accounts.pool_data;
        let distribute_data = &mut ctx.accounts.distribute_data;

        let clock = clock::Clock::get().unwrap();
        let last_time = clock.unix_timestamp as u32;

        if pool_data.start_time + SECONDS_PER_DATE * 14 < last_time 
        {
            return Err(CustomError::InvalidNft.into());
        }

        pool_data.distribute_id = pool_data.distribute_id + 1;
        pool_data.start_time = last_time;
        
        **ctx.accounts.fund_pool.to_account_info().try_borrow_mut_lamports()? -= distribute_data.rewards_amount;
        **ctx.accounts.user.try_borrow_mut_lamports()? += distribute_data.rewards_amount;
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeContext<'info> {
    #[account(init, seeds = [STATISTIC_SEEDS.as_ref()], bump, payer = admin, space = 8 + 4 + 8)]
    pub statistic: Account<'info, Statistic>,
    #[account(mut, constraint = admin.key() == ADMIN_KEY)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct StakeContext<'info> {
   #[account(mut)]
    pub statistic: Account<'info, Statistic>,
    #[account(init_if_needed, seeds = [POOL_SEEDS.as_ref(), user.key().as_ref()], bump, payer = user, space = 8 + 32 + 4 + 8 + 8)]
    pub pool: Account<'info, Pool>,
    #[account(init_if_needed, seeds = [POOL_DATA_SEEDS.as_ref(), user.key().as_ref(), mint.key().as_ref()], bump, payer = user, space = 8 + 32 + 32 + 4 + 4)]
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
    /// CHECK: it's not dangerous
    #[account(init_if_needed, seeds = [FUND_SEED.as_ref(), admin.key().as_ref()], bump, payer = admin, space = 0)]
    pub fund_pool : AccountInfo<'info >,
    #[account(mut, constraint = admin.key() == ADMIN_KEY)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ReFundContext<'info> {
    #[account(mut)]
    pub statistic: Account<'info, Statistic>,
    /// CHECK: it's not dangerous
    #[account(mut, seeds = [FUND_SEED.as_ref(), admin.key().as_ref()], bump)]
    pub fund_pool : AccountInfo<'info >,
    #[account(mut, constraint = admin.key() == ADMIN_KEY)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(index: u32)]
pub struct DistributeContext<'info>{
    #[account(mut)]
    pub statistic: Account<'info, Statistic>,
    #[account(mut, constraint = admin.key() == ADMIN_KEY)]
    pub admin: Signer<'info>,
    #[account(init, seeds = [format!("distribute_data_seed{}", index).as_ref(), admin.key().as_ref()], bump, payer = admin, space = 8 + 4 + 4 + 8)]
    pub distribute_data: Account<'info, DistributeData>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ClaimContext<'info>{
     /// CHECK: it's not dangerous
    #[account(mut, seeds = [FUND_SEED.as_ref()], bump)]
    pub fund_pool : AccountInfo<'info >,
    pub user: Signer<'info>,
    #[account(mut, constraint = pool_data.user == user.key())]
    pub pool_data: Account<'info, PoolData>,
    pub distribute_data: Account<'info, DistributeData>,
    pub system_program: Program<'info, System>
}

#[account]
pub struct Statistic {
    pub staked_count: u32,
    pub currency_count: u64,
}

#[account]
pub struct Pool {
    pub user: Pubkey,
    pub staked_count: u32,
    pub total_reward: u64,
    pub transfer_amount: u64,
}

#[account]
pub struct PoolData {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub start_time: u32,
    pub distribute_id: u32
}

#[account]
pub struct DistributeData{
    pub reward_id: u32,
    pub start_time: u32,  
    pub rewards_amount: u64,
}

#[error_code]
pub enum CustomError {
    #[msg("Invalid Nft.")]
    InvalidNft,
    #[msg("Transfer too much.")]
    TooMuchTransfer
}