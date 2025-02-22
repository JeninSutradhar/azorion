use anchor_lang::prelude::*;
// use anchor_lang::declare_id;
use anchor_lang::context::CpiContext;
use anchor_lang::solana_program::clock;
use anchor_lang::system_program; // Import system_program for transfer // Import CpiContext
pub const BASE_REWARD_MULTIPLIER: u64 = 1_000_000; // Equivalent to multiplying by 0.000001 SOL

declare_id!("YOUR_PROGRAM_ID"); //Replace with the program's public key, the address that the program is deployed to.

#[program]
pub mod azorion {
    use super::*;
    
    // set outside of this module }Scope errors} 
    // pub const BASE_REWARD_MULTIPLIER: u64 = 1_000_000; // Equivalent to multiplying by 0.000001 SOL

    pub fn initialize(
        ctx: Context<Initialize>,
        initial_supply: u64,
        min_available_tasks: u8,
        max_available_tasks: u8,
    ) -> Result<()> {
        let state = &mut ctx.accounts.program_state;

        //set authority
        state.authority = *ctx.accounts.authority.key;

        //Set global program params.
        state.total_sol_available = initial_supply * BASE_REWARD_MULTIPLIER;
        state.current_sol_balance = initial_supply * BASE_REWARD_MULTIPLIER; // Initialize with available supply
        state.min_available_tasks = min_available_tasks;
        state.max_available_tasks = max_available_tasks;
        state.available_tasks = min_available_tasks; // initialize available task to its defined minimal level

        state.task_last_updated = clock::Clock::get().unwrap().unix_timestamp;

        msg!("Program initialized with {} SOL available.", initial_supply);
        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>, activity_type: String) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let program_state_account = &ctx.accounts.program_state; // Immutable borrow here for checks

        //Enforce program ownership restrictions (using immutable borrow)
        if program_state_account.authority != *ctx.accounts.authority.key {
            return Err(error!(ErrorCode::Unauthorized));
        }

        //1. Validate activity (using immutable borrow)
        let task_type = validate_activity(&activity_type)?;

        //2. Validate if the Task is actually available (using immutable borrow)
        if !is_task_available(task_type, program_state_account)? {
            return Err(error!(ErrorCode::TaskUnavailable));
        }

        //3. Validate cooldown.
        if user.last_claimed_ts != 0
            && clock::Clock::get().unwrap().unix_timestamp - user.last_claimed_ts < 5
        {
            return Err(error!(ErrorCode::CooldownActive));
        }

        //4. Calculate reward (using immutable borrow)
        let base_reward = get_base_reward(task_type);
        let adjusted_reward = calculate_dynamic_reward(program_state_account, base_reward)?;

        msg!("Base Reward for activity :{}", base_reward);
        msg!("Adjusted Reward after supply logic: {}", adjusted_reward);

        //5. Apply anti-farming.  Store repetition count.
        let repetition_count = update_repetition_count(user, &activity_type);
        let farming_penalty = calculate_farming_penalty(repetition_count);
        let final_reward = (adjusted_reward as f64 * farming_penalty) as u64;

        msg!("Current count repeating the same Task {}", repetition_count);
        msg!("Farm Penalty Reduction Percentage: {}", farming_penalty);
        msg!(
            "Applying Farming Penalty Result: {} Lamports (SOL)",
            final_reward
        );

        //6. Transfer reward (using immutable borrow for check, then mutable for balance update)
        if program_state_account.current_sol_balance < final_reward {
            // Immutable check
            return Err(error!(ErrorCode::InsufficientBalance));
        }

        // Construct CpiContext and perform transfer (now using ctx.accounts.program_state directly)
        let transfer_instruction = system_program::Transfer {
            from: ctx.accounts.program_state.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
        };
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_instruction,
        );
        system_program::transfer(cpi_context, final_reward)?;

        //7. Update program state and user info AFTER transfer (NOW get mutable borrow for updates)
        let program_state_mut = &mut ctx.accounts.program_state; // Mutable borrow NOW
        program_state_mut.current_sol_balance -= final_reward; // Reduce global balances after transfers
        user.reward_amount += final_reward;
        user.last_activity = activity_type;
        user.last_claimed_ts = clock::Clock::get().unwrap().unix_timestamp;

        msg!(
            "Reward of {} lamports transferred to {}",
            final_reward,
            ctx.accounts.user_reward_account.key()
        );

        Ok(())
    }

    //Only run to refresh if X Time has elapsed, to not over saturate instructions every 10 sec
    pub fn randomize_tasks(ctx: Context<RandomizeTasks>) -> Result<()> {
        let program_state = &mut ctx.accounts.program_state;

        //Enforce authority restriction.
        if program_state.authority != *ctx.accounts.authority.key {
            return Err(error!(ErrorCode::Unauthorized));
        }

        // Update every 10 seconds, check to spamming.
        let current_time = clock::Clock::get().unwrap().unix_timestamp;
        if current_time - program_state.task_last_updated < 10 {
            return Err(error!(ErrorCode::CooldownRngTasks));
        }

        program_state.task_last_updated = current_time;

        // Randomization logic to switch number availableTasks (simulating states where there is more or less
        // work available (the number of task)) with each user activity.

        //Randomize  within pre-defined  min - max
        let min: u8 = program_state.min_available_tasks;
        let max: u8 = program_state.max_available_tasks;

        program_state.available_tasks = randomize_available_tasks(min, max);
        msg!(
            "New number Randomized Total Tasks:  {}",
            program_state.available_tasks
        );
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(initial_supply: u64, min_available_tasks: u8, max_available_tasks:u8)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 32 + 1 + 1 + 8, // discriminator + balance +authority key + minimum & maximum allowed Tasks per block  + Last Random TS value  +8
        // seeds & bump
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

    /// CHECK: This account is safe because it's just used to receive rewards and we don't read or write to it directly
    #[account(mut)]
    pub user_reward_account: AccountInfo<'info>, // where to credit tokens.  this must be the recieving authority signer

    #[account(
        mut,
        seeds = [b"user", user_reward_account.key().as_ref()], // seeds can combine static key ("user", etc, ...,) with others variable based from different authorities like the payer here
        bump,
    )]
    pub user: Account<'info, User>,

    /// Authority Key
    #[account(mut)]
    pub authority: Signer<'info>, //user needs authority because transfers are authorized and managed centrally from Admin authority (DAO key);

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RandomizeTasks<'info> {
    #[account(mut)]
    pub program_state: Account<'info, ProgramState>,
    #[account(mut)]
    pub authority: Signer<'info>, //needs permission in this case also (unless if running it via dedicated program via timer/cron)
}

//------------------ Account Structs  --------------------------

#[account]
pub struct ProgramState {
    pub total_sol_available: u64, // Total SOL available to distribute.
    pub current_sol_balance: u64, // Current SOL balance available
    pub authority: Pubkey,        //Program  Manager to define program configs (ie. fees etc);
    pub min_available_tasks: u8,  //  Minimum available task.
    pub max_available_tasks: u8,  //Maximum tasks in the available taks stack.
    pub available_tasks: u8, //Number of tasks availabel from RNG  and the current state of tasks
    pub task_last_updated: i64, // Unix timestamp of the last random tasks; (so tasks states, available /unavailable).
}

#[account]
pub struct User {
    pub reward_amount: u64,          // Cumulative reward.
    pub last_activity: String, //Last reward actiivty claim (activity task string description ie:"daily checkin", "invite-023", etc)
    pub last_claimed_ts: i64,  //Last Unix Time-Stamp
    pub repetition_counts: [u8; 10], //Stores how many times has be repeated an array of MAX  last 10 elements claimed
    pub tasks: [String; 10], // array  where the user stores latest actions that happened (task string, similar last_activity task name "reward claim0221044, task".......)
}

//----------------------ENUM DEFINITIONS & UTILS FUNCTIONS  --------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

fn validate_activity(activity: &String) -> Result<ActivityType> {
    match activity.to_lowercase().as_str() {
        "check-in" => Ok(ActivityType::CheckIn),
        "view analytics" => Ok(ActivityType::ViewAnalytics),
        "vote in a poll" => Ok(ActivityType::VoteInPoll),
        "subscribe to a smart contract" => Ok(ActivityType::SubscribeContract),
        "leave feedback on a dapp" => Ok(ActivityType::LeaveFeedback),
        "complete a profile setup" => Ok(ActivityType::CompleteProfile),
        "cast a vote" => Ok(ActivityType::CastVote),
        "send a message" => Ok(ActivityType::SendMessage),
        "refer a user" => Ok(ActivityType::ReferUser),
        "complete a tutorial on solana usage" => Ok(ActivityType::CompleteTutorial),
        "test a beta feature on a dapp" => Ok(ActivityType::TestBetaFeature),
        "review a smart contract’s code" => Ok(ActivityType::ReviewSmartContract),
        "deploy a sample smart contract" => Ok(ActivityType::DeploySmartContract),
        "stake sol for at least 7 days" => Ok(ActivityType::StakeSol),
        "mint and transfer an nft" => Ok(ActivityType::MintNft),
        "provide liquidity to a protocol" => Ok(ActivityType::ProvideLiquidity),
        "run a validator node for 24 hours" => Ok(ActivityType::RunValidator),
        "contribute code to an open-source project" => Ok(ActivityType::ContributeCode),
        _ => Err(error!(ErrorCode::InvalidActivity)),
    }
}

fn is_task_available(
    task_type: ActivityType,
    program_state: &Account<ProgramState>,
) -> Result<bool> {
    let task_index = task_type as usize;
    if task_index > program_state.available_tasks as usize {
        //cast task type enumeration with current limit on  `avalailable task; `if enum exceds
        return Ok(false); //meaning tasks that are "turned OFF; can't be accessed by this program cycle."
    }
    return Ok(true); //tasks its on the stack its able to be acessed
}

fn get_base_reward(activity: ActivityType) -> u64 {
    match activity {
        ActivityType::CheckIn => 10 * BASE_REWARD_MULTIPLIER, // 0.01 SOL (as 1 SOL is equal to 10**9 Lamports
        ActivityType::ViewAnalytics => 10 * BASE_REWARD_MULTIPLIER, // 0.01 SOL
        ActivityType::VoteInPoll => 10 * BASE_REWARD_MULTIPLIER, // 0.01 SOL
        ActivityType::SubscribeContract => 10 * BASE_REWARD_MULTIPLIER, // 0.01 SOL
        ActivityType::LeaveFeedback => 10 * BASE_REWARD_MULTIPLIER, // 0.01 SOL
        ActivityType::CompleteProfile => 10 * BASE_REWARD_MULTIPLIER, // 0.01 SOL
        ActivityType::CastVote => 50 * BASE_REWARD_MULTIPLIER, // 0.05 SOL
        ActivityType::SendMessage => 50 * BASE_REWARD_MULTIPLIER, // 0.05 SOL
        ActivityType::ReferUser => 50 * BASE_REWARD_MULTIPLIER, // 0.05 SOL
        ActivityType::CompleteTutorial => 50 * BASE_REWARD_MULTIPLIER, // 0.05 SOL
        ActivityType::TestBetaFeature => 50 * BASE_REWARD_MULTIPLIER, // 0.05 SOL
        ActivityType::ReviewSmartContract => 50 * BASE_REWARD_MULTIPLIER, // 0.05 SOL
        ActivityType::DeploySmartContract => 100 * BASE_REWARD_MULTIPLIER, // 0.1 SOL
        ActivityType::StakeSol => 100 * BASE_REWARD_MULTIPLIER, // 0.1 SOL
        ActivityType::MintNft => 100 * BASE_REWARD_MULTIPLIER, // 0.1 SOL
        ActivityType::ProvideLiquidity => 100 * BASE_REWARD_MULTIPLIER, // 0.1 SOL
        ActivityType::RunValidator => 100 * BASE_REWARD_MULTIPLIER, // 0.1 SOL
        ActivityType::ContributeCode => 100 * BASE_REWARD_MULTIPLIER, // 0.1 SOL
    }
}

fn calculate_dynamic_reward(
    program_state: &Account<ProgramState>,
    base_reward: u64,
) -> Result<u64> {
    let available_tasks = program_state.available_tasks as u64;

    //Simulated:

    //1. more task / than more uses to work with/claim, offer bonuses: -> High Demand;

    //  this current test can only represent the state
    //the  actual users its "always gonna be the current users", in theory it works because for that specific program
    // run instance/context is being able to have this count from other  program for a better supply-demand status logic.
    let number_of_users: u64 = 10; // Dummy variable here  ( number of tasks)

    msg!("Number Current Total Randomized Tasks: {}", available_tasks);

    //TODO:: we can add here the capacity or variable about how many accounts have signed since started  here
    msg!("Users are connected: {}", number_of_users); //current_tasks   usersCount() //<- from external data to manage this paramter

    if available_tasks > number_of_users {
        //Increase Reward -> 20% Max

        let increase_factor: f64 = 1.0 + 0.20; //  Fixed 20% of bonuses rewards that available tasks (ie. job offer; available offer more attractive);
        let adjusted_reward: f64 = base_reward as f64 * increase_factor;
        Ok(adjusted_reward as u64) //return lamports rewards to mint;
    } else if number_of_users > available_tasks {
        //Decrease Reward   (-10%); Max Loss of Tokens to distribute as rewerd because Low-Job supply tasks.

        let decrease_factor: f64 = 1.0 - 0.10; // 10% decrease of distribution based on avilable; (more claimants/ users and less SOL to destribute; rewards shrink, by a 10%)

        let adjusted_reward: f64 = base_reward as f64 * decrease_factor;
        Ok(adjusted_reward as u64) //return lamports to reward
    } else {
        Ok(base_reward) //return back originall Lamports; base as no majoraty change happens
    }
}

fn update_repetition_count(user: &mut Account<User>, activity: &String) -> u8 {
    let activity_task = activity.clone();
    let array_size: usize = user.repetition_counts.len(); // Renamed to snake_case

    //add values from 1 to the counter from new array, adding value always 0  if never repeats the task.

    if array_size > 0 {
        //Validate and push  shift and assign new valuer to this history record list of Tasks claimeds
        user.repetition_counts.rotate_left(1); // Shift elements by one position
        user.tasks.rotate_left(1);

        //compare both names from tasks
        let prev_activity = &user.tasks[array_size - 1];
        //store a "1" repetition if  compare the last and most new activity in the same name equals:

        let is_same: bool =
            prev_activity.to_lowercase().as_str() == activity_task.to_lowercase().as_str(); // Renamed to snake_case //prevent  cases  "votinpoll vs  votinpoll" (1 repetiion points);

        //add new action performed
        if is_same {
            user.repetition_counts[array_size - 1] = 1 + user.repetition_counts[array_size - 2];
        //add number points as repeating claims/
        } else {
            user.repetition_counts[array_size - 1] = 0; // task that did not perform the user; (claim diffiernt offer than last offer actioned);
            user.tasks[array_size - 1] = activity_task; // saves activity type/
        }
        return user.repetition_counts[array_size - 1]; //Return actual  count point on Task; if repited offer, count.
    }

    return 0; //return never be the point will not exist
}

fn calculate_farming_penalty(repetition_count: u8) -> f64 {
    if repetition_count == 0 {
        return 1.0; // no repitition  , its value 1  (No changes)
    }

    let rep: f64 = repetition_count as f64; // cast i32
    return 1.0 - (0.50f64 * rep); //apply 50% multiply on loop tasks until next offer/cycle its  be enabled back.
}

fn randomize_available_tasks(min: u8, max: u8) -> u8 {
    let result = generate_range(min, max); // Call generate_range directly
    msg!(
        "random results numbers:: from {} to {} {}",
        min,
        max,
        result
    );
    result as u8 // Cast to u8
}

//random num utils to implement rand task on epoch time:

pub fn generate_range(min: u8, max: u8) -> u64 {
    let cpi_context = Clock::get().unwrap(); // Returns system time

    if min >= max {
        panic!("min must be less than max");
    }

    let now_timestamp: u64 = cpi_context.unix_timestamp.unsigned_abs();

    let range: u64 = max as u64 - min as u64; //1   example case 3 and 2

    let rand: u64 =
        ((now_timestamp + range + (cpi_context.slot as u64)) % range as u64) + (min as u64); //3 add base 0 1  return rand [ min ;   <max) return values + or min

    msg!(
        "Generate Ramdon Tasks number from BlockNumber time_stamp {}.",
        rand
    );
    return rand;
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid activity type")]
    InvalidActivity,
    #[msg("Cooldown is active")]
    CooldownActive,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Unauthorized to update or initialize state")]
    Unauthorized,
    #[msg("Max Tasks Exceeded: Program Failure and Vulnerable Code to Review ")]
    MaxTasksExceeded,
    #[msg("This task no longer exits or became an unavailable Action per Cycle State Randomizaiton Task Stack Number:: try other one on loop;..;;;;")]
    TaskUnavailable,

    #[msg("Update tasks randomized, beign refreshed; you are attemping to many many randomized updates actions..")]
    CooldownRngTasks,
}
