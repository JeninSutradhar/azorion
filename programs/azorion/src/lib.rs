use anchor_lang::prelude::*;
use anchor_lang::context::CpiContext;
use anchor_lang::solana_program::clock;
// use anchor_lang::solana_program::hash; // No longer directly used in randomize_tasks, so removing to avoid "unused import" warnings
use anchor_lang::system_program;

pub const BASE_REWARD_MULTIPLIER: u64 = 1_000_000;

// REPLACE WITH URS PROGRAM ID
declare_id!("ARLUx3Yx7DKaTyKtngGr2ZahmAW2qUV7fzxttKudce4Y"); // Ensure YOUR_PROGRAM_ID is replaced after build

#[program]
pub mod azorion {
    use super::*;

    
    
    pub fn initialize(
        ctx: Context<Initialize>,
        initial_supply: u64,
        min_available_tasks: u8,
        max_available_tasks: u8,
    ) -> Result<()> {
        let state = &mut ctx.accounts.program_state;
        state.authority = *ctx.accounts.authority.key;
        state.total_sol_available = initial_supply * BASE_REWARD_MULTIPLIER;
        state.current_sol_balance = state.total_sol_available;
        state.min_available_tasks = min_available_tasks;
        state.max_available_tasks = max_available_tasks;
        state.available_tasks = min_available_tasks;
        state.task_last_updated = clock::Clock::get().unwrap().unix_timestamp;

        msg!("Program initialized with {} SOL available.", initial_supply);
        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>, activity_type_u8: u8) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let program_state_account = &ctx.accounts.program_state;

        if program_state_account.authority != *ctx.accounts.authority.key {
            return Err(error!(ErrorCode::Unauthorized));
        }

        let task_type: ActivityType = ActivityType::from_u8(activity_type_u8)
            .ok_or(ErrorCode::InvalidActivity)?;

        if !is_task_available(task_type, program_state_account)? {
            return Err(error!(ErrorCode::TaskUnavailable));
        }

        if user.last_claimed_ts != 0
            && clock::Clock::get().unwrap().unix_timestamp - user.last_claimed_ts < 5
        {
            return Err(error!(ErrorCode::CooldownActive));
        }

        let base_reward = get_base_reward(task_type);
        let adjusted_reward = calculate_dynamic_reward(program_state_account, base_reward)?;

        msg!("Base Reward: {} lamports", base_reward);
        msg!("Adjusted Reward: {} lamports", adjusted_reward);

        let repetition_count = update_repetition_count(user, task_type);
        let farming_penalty = calculate_farming_penalty(repetition_count);
        let final_reward = (adjusted_reward as f64 * farming_penalty) as u64;

        msg!("Repetition Count: {}", repetition_count);
        msg!("Farm Penalty: {}", farming_penalty * 100.0);
        msg!("Final Reward: {} Lamports", final_reward);

        if program_state_account.current_sol_balance < final_reward {
            return Err(error!(ErrorCode::InsufficientBalance));
        }

        let transfer_instruction = system_program::Transfer {
            from: ctx.accounts.program_state.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
        };
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_instruction,
        );
        system_program::transfer(cpi_context, final_reward)?;

        let program_state_mut = &mut ctx.accounts.program_state;
        program_state_mut.current_sol_balance -= final_reward;
        user.reward_amount += final_reward;
        user.last_activity = task_type.to_string();
        user.last_claimed_ts = clock::Clock::get().unwrap().unix_timestamp;

        msg!(
            "Reward {} lamports transferred to {}",
            final_reward,
            ctx.accounts.user_reward_account.key()
        );

        Ok(())
    }

    pub fn randomize_tasks(ctx: Context<RandomizeTasks>) -> Result<()> {
        let program_state = &mut ctx.accounts.program_state;

        if program_state.authority != *ctx.accounts.authority.key {
            return Err(error!(ErrorCode::Unauthorized));
        }

        let current_time = clock::Clock::get().unwrap().unix_timestamp;
        if current_time - program_state.task_last_updated < 10 {
            return Err(error!(ErrorCode::CooldownRngTasks));
        }

        program_state.task_last_updated = current_time;

        let min: u8 = program_state.min_available_tasks;
        let max: u8 = program_state.max_available_tasks;

        program_state.available_tasks = randomize_available_tasks(min, max)?;
        msg!(
            "Randomized Tasks Updated: Available Tasks = {}",
            program_state.available_tasks
        );
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(initial_supply: u64, min_available_tasks: u8, max_available_tasks: u8)]
pub struct Initialize<'info> {
    #[account(
        init,  // Corrected order: init, payer, space
        payer = authority,
        space = 8 + ProgramState::MAX_SIZE,
        seeds = [b"program_state"],
        bump
    )]
    pub program_state: Account<'info, ProgramState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub program_state: Account<'info, ProgramState>,
    /// CHECK: Account is safe as it's only used to receive rewards
    #[account(mut)]
    pub user_reward_account: AccountInfo<'info>,
    #[account(
        init,  // Added 'init' here as it might be missing and causing issue? (though should not be 'init' actually but trying now for error resolve)
        payer = authority, // 'payer' is required for 'init' accounts.  even if its not initialize directly , its required by syntax.
        seeds = [b"user", user_reward_account.key().as_ref()],
        bump,
        space = 8 + User::MAX_SIZE,
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RandomizeTasks<'info> {
    #[account(mut)]
    pub program_state: Account<'info, ProgramState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

//------------------ Account Structs  --------------------------

#[account]
#[derive(Default)]
pub struct ProgramState {
    pub total_sol_available: u64,
    pub current_sol_balance: u64,
    pub authority: Pubkey,
    pub min_available_tasks: u8,
    pub max_available_tasks: u8,
    pub available_tasks: u8,
    pub task_last_updated: i64,
}

impl ProgramState {
    const MAX_SIZE: usize = 8 + 8 + 8 + 32 + 1 + 1 + 1 + 8;
}

#[account]
#[derive(Default)]
pub struct User {
    pub reward_amount: u64,
    pub last_activity: String,
    pub last_claimed_ts: i64,
    pub repetition_counts: [u8; 10],
    pub tasks: [String; 10],
}

impl User {
    const MAX_SIZE: usize = 8 +
        8 +
        (4 + 50) +
        8 +
        10 +
        (10 * (4 + 50));
}

//----------------------ENUM DEFINITIONS & UTILS FUNCTIONS  --------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub enum ActivityType {
    CheckIn,
    ViewAnalytics,
    VoteInPoll,
    SubscribeContract,
    LeaveFeedback,
    CompleteProfile,
    CastVote,
    SendMessage,
    ReferUser,
    CompleteTutorial,
    TestBetaFeature,
    ReviewSmartContract,
    DeploySmartContract,
    StakeSol,
    MintNft,
    ProvideLiquidity,
    RunValidator,
    ContributeCode,
}

impl ActivityType {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ActivityType::CheckIn),
            1 => Some(ActivityType::ViewAnalytics),
            2 => Some(ActivityType::VoteInPoll),
            3 => Some(ActivityType::SubscribeContract),
            4 => Some(ActivityType::LeaveFeedback),
            5 => Some(ActivityType::CompleteProfile),
            6 => Some(ActivityType::CastVote),
            7 => Some(ActivityType::SendMessage),
            8 => Some(ActivityType::ReferUser),
            9 => Some(ActivityType::CompleteTutorial),
            10 => Some(ActivityType::TestBetaFeature),
            11 => Some(ActivityType::ReviewSmartContract),
            12 => Some(ActivityType::DeploySmartContract),
            13 => Some(ActivityType::StakeSol),
            14 => Some(ActivityType::MintNft),
            15 => Some(ActivityType::ProvideLiquidity),
            16 => Some(ActivityType::RunValidator),
            17 => Some(ActivityType::ContributeCode),
            _ => None,
        }
    }

    fn to_string(&self) -> String {
        match self {
            ActivityType::CheckIn => "check-in".to_string(),
            ActivityType::ViewAnalytics => "view analytics".to_string(),
            ActivityType::VoteInPoll => "vote in a poll".to_string(),
            ActivityType::SubscribeContract => "subscribe to a smart contract".to_string(),
            ActivityType::LeaveFeedback => "leave feedback on a dapp".to_string(),
            ActivityType::CompleteProfile => "complete a profile setup".to_string(),
            ActivityType::CastVote => "cast a vote".to_string(),
            ActivityType::SendMessage => "send a message".to_string(),
            ActivityType::ReferUser => "refer a user".to_string(),
            ActivityType::CompleteTutorial => "complete a tutorial on solana usage".to_string(),
            ActivityType::TestBetaFeature => "test a beta feature on a dapp".to_string(),
            ActivityType::ReviewSmartContract => "review a smart contract’s code".to_string(),
            ActivityType::DeploySmartContract => "deploy a sample smart contract".to_string(),
            ActivityType::StakeSol => "stake sol for at least 7 days".to_string(),
            ActivityType::MintNft => "mint and transfer an nft".to_string(),
            ActivityType::ProvideLiquidity => "provide liquidity to a protocol".to_string(),
            ActivityType::RunValidator => "run a validator node for 24 hours".to_string(),
            ActivityType::ContributeCode => "contribute code to an open-source project".to_string(),
        }
    }
}


fn is_task_available(
    task_type: ActivityType,
    program_state: &Account<ProgramState>,
) -> Result<bool> {
    let task_index = task_type as usize;
    Ok(task_index <= program_state.available_tasks as usize)
}

fn get_base_reward(activity: ActivityType) -> u64 {
    match activity {
        ActivityType::CheckIn => 10 * BASE_REWARD_MULTIPLIER,
        ActivityType::ViewAnalytics => 10 * BASE_REWARD_MULTIPLIER,
        ActivityType::VoteInPoll => 10 * BASE_REWARD_MULTIPLIER,
        ActivityType::SubscribeContract => 10 * BASE_REWARD_MULTIPLIER,
        ActivityType::LeaveFeedback => 10 * BASE_REWARD_MULTIPLIER,
        ActivityType::CompleteProfile => 10 * BASE_REWARD_MULTIPLIER,
        ActivityType::CastVote => 50 * BASE_REWARD_MULTIPLIER,
        ActivityType::SendMessage => 50 * BASE_REWARD_MULTIPLIER,
        ActivityType::ReferUser => 50 * BASE_REWARD_MULTIPLIER,
        ActivityType::CompleteTutorial => 50 * BASE_REWARD_MULTIPLIER,
        ActivityType::TestBetaFeature => 50 * BASE_REWARD_MULTIPLIER,
        ActivityType::ReviewSmartContract => 50 * BASE_REWARD_MULTIPLIER,
        ActivityType::DeploySmartContract => 100 * BASE_REWARD_MULTIPLIER,
        ActivityType::StakeSol => 100 * BASE_REWARD_MULTIPLIER,
        ActivityType::MintNft => 100 * BASE_REWARD_MULTIPLIER,
        ActivityType::ProvideLiquidity => 100 * BASE_REWARD_MULTIPLIER,
        ActivityType::RunValidator => 100 * BASE_REWARD_MULTIPLIER,
        ActivityType::ContributeCode => 100 * BASE_REWARD_MULTIPLIER,
    }
}

fn calculate_dynamic_reward(
    program_state: &Account<ProgramState>,
    base_reward: u64,
) -> Result<u64> {
    let available_tasks = program_state.available_tasks as u64;
    let number_of_users: u64 = 10;

    if available_tasks > number_of_users {
        let increase_factor: f64 = 1.0 + 0.20;
        let adjusted_reward: f64 = base_reward as f64 * increase_factor;
        Ok(adjusted_reward as u64)
    } else if number_of_users > available_tasks {
        let decrease_factor: f64 = 1.0 - 0.10;
        let adjusted_reward: f64 = base_reward as f64 * decrease_factor;
        Ok(adjusted_reward as u64)
    } else {
        Ok(base_reward)
    }
}

fn update_repetition_count(user: &mut Account<User>, activity: ActivityType) -> u8 {
    let array_size: usize = user.repetition_counts.len();

    if array_size > 0 {
        user.repetition_counts.rotate_left(1);
        user.tasks.rotate_left(1);

        let prev_activity = &user.tasks[array_size - 1];
        let is_same: bool =
            prev_activity.to_lowercase().as_str() == activity.to_string().to_lowercase().as_str();

        if is_same {
            user.repetition_counts[array_size - 1] = 1 + user.repetition_counts[array_size - 2];
        } else {
            user.repetition_counts[array_size - 1] = 0;
            user.tasks[array_size - 1] = activity.to_string();
        }
        user.repetition_counts[array_size - 1]
    } else {
        0
    }
}

fn calculate_farming_penalty(repetition_count: u8) -> f64 {
    if repetition_count == 0 {
        1.0
    } else {
        let rep: f64 = repetition_count as f64;
        1.0 - (0.50f64 * rep)
    }
}

// Improved RNG using block hash
fn randomize_available_tasks(min: u8, max: u8) -> Result<u8> {
    let clock = Clock::get()?;
    let slot = clock.slot;
    let rand_value = (slot % (max - min + 1) as u64) as u8 + min;
    Ok(rand_value)
}


#[error_code]
pub enum ErrorCode {
    #[msg("Invalid activity type provided.")]
    InvalidActivity,
    #[msg("Cooldown period is still active. Please wait before claiming again.")]
    CooldownActive,
    #[msg("Insufficient program balance to process reward.")]
    InsufficientBalance,
    #[msg("Unauthorized action. Only program authority can perform this operation.")]
    Unauthorized,
    #[msg("Max tasks limit exceeded. Review program configuration.")]
    MaxTasksExceeded,
    #[msg("The selected task is currently unavailable. Please try another task.")]
    TaskUnavailable,
    #[msg("Task randomization cooldown active. Please wait before attempting again.")]
    CooldownRngTasks,
}
