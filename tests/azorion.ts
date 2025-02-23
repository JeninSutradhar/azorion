import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Azorion } from "../target/types/azorion";
import { assert } from "chai";

// Define Task Types Enum as numbers, matching Rust Enum for testing consistency
const TASK_TYPES = {
    CheckIn: 0,
    ViewAnalytics: 1,
    VoteInPoll: 2,
    SubscribeContract: 3,
    LeaveFeedback: 4,
    CompleteProfile: 5,
    CastVote: 6,
    SendMessage: 7,
    ReferUser: 8,
    CompleteTutorial: 9,
    TestBetaFeature: 10,
    ReviewSmartContract: 11,
    DeploySmartContract: 12,
    StakeSol: 13,
    MintNft: 14,
    ProvideLiquidity: 15,
    RunValidator: 16,
    ContributeCode: 17,
};

describe("azorion_task_reward_program_tests", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Azorion as Program<Azorion>;
    const provider = anchor.getProvider();

    let programStateAccount: anchor.web3.PublicKey;
    let authority: anchor.web3.Keypair;
    let userRewardAccount: anchor.web3.Keypair;
    let user: anchor.web3.PublicKey;
    let programStateBump: number;
    let userBump: number;

    before(async () => {
        authority = anchor.web3.Keypair.generate();

        // Airdrop SOL to authority account for transaction fees
        await provider.connection.confirmTransaction(
            await provider.connection.requestAirdrop(authority.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
            "processed"
        );

        userRewardAccount = anchor.web3.Keypair.generate();

        // Derive Program State Account PDA
        [programStateAccount, programStateBump] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from("program_state")],
            program.programId
        );

        // Derive User Account PDA
        [user, userBump] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from("user"), userRewardAccount.publicKey.toBuffer()],
            program.programId
        );

        const initialSupply = new anchor.BN(10); // 10 SOL for initial supply
        const minTasks = 5;
        const maxTasks = 10;

        // Initialize the program state
        await program.methods.initialize(initialSupply, minTasks, maxTasks).accounts({
            programState: programStateAccount,
            authority: authority.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        }).signers([authority]).rpc();
    });

    it("Initialize Program State", async () => {
        const state = await program.account.programState.fetch(programStateAccount);
        assert.ok(state.authority.equals(authority.publicKey), "Authority should be correctly initialized");
    });

    describe("Reward Calculation Scenarios", () => {
        it("More Tasks Than Users - Reward Increase", async () => {
            const initialProgramState = await program.account.programState.fetch(programStateAccount);
            const initialBalance = initialProgramState.currentSolBalance.toNumber();
            const activityType = TASK_TYPES.CheckIn; // Use Enum Number

            await program.methods.claimReward(activityType).accounts({
                programState: programStateAccount,
                userRewardAccount: userRewardAccount.publicKey,
                user: user,
                authority: authority.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
            }).signers([authority, userRewardAccount]).rpc();

            const finalProgramState = await program.account.programState.fetch(programStateAccount);
            const finalBalance = finalProgramState.currentSolBalance.toNumber();
            const rewardGiven = initialBalance - finalBalance;
            const baseReward = 0.01 * 1_000_000;
            const expectedMaxReward = baseReward * 1.20;

            assert.isAbove(rewardGiven, baseReward, "Reward should increase in high demand scenario");
            assert.isAtMost(rewardGiven, expectedMaxReward, "Reward increase should not exceed 20%");
        });

        it("More Users Than Tasks - Reward Decrease", async () => {
            const initialProgramState = await program.account.programState.fetch(programStateAccount);
            const initialBalance = initialProgramState.currentSolBalance.toNumber();
            const activityType = TASK_TYPES.VoteInPoll; // Use Enum Number

            await program.methods.claimReward(activityType).accounts({
                programState: programStateAccount,
                userRewardAccount: userRewardAccount.publicKey,
                user: user,
                authority: authority.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
            }).signers([authority, userRewardAccount]).rpc();

            const finalProgramState = await program.account.programState.fetch(programStateAccount);
            const finalBalance = finalProgramState.currentSolBalance.toNumber();
            const rewardGiven = initialBalance - finalBalance;
            const baseReward = 0.01 * 1_000_000;
            const expectedMinReward = baseReward * 0.90;

            assert.isBelow(rewardGiven, baseReward, "Reward should decrease in low demand scenario");
            assert.isAtLeast(rewardGiven, expectedMinReward, "Reward decrease should not be below 10%");
        });

        it("Balanced Demand-Supply - Reward Unchanged", async () => {
            const initialProgramState = await program.account.programState.fetch(programStateAccount);
            const initialBalance = initialProgramState.currentSolBalance.toNumber();
            const activityType = TASK_TYPES.CompleteProfile; // Use Enum Number

            await program.methods.claimReward(activityType).accounts({
                programState: programStateAccount,
                userRewardAccount: userRewardAccount.publicKey,
                user: user,
                authority: authority.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
            }).signers([authority, userRewardAccount]).rpc();

            const finalProgramState = await program.account.programState.fetch(programStateAccount);
            const finalBalance = finalProgramState.currentSolBalance.toNumber();
            const rewardGiven = initialBalance - finalBalance;
            const baseReward = 0.01 * 1_000_000;

            assert.equal(rewardGiven, baseReward, "Reward should remain unchanged in balanced scenario");
        });
    });

    describe("Anti-Farming Mechanism Tests", () => {
        it("Repeat Same Task 3 Times - Reward Reduced by 50%", async () => {
            const activityType = TASK_TYPES.VoteInPoll;
            let reward1: number, reward2: number, reward3: number;

            // 1st time - Normal reward
            const balanceBefore1 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            await program.methods.claimReward(activityType).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            const balanceAfter1 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            reward1 = balanceBefore1 - balanceAfter1;

            // 2nd time - Normal reward
            const balanceBefore2 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            await program.methods.claimReward(activityType).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            const balanceAfter2 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            reward2 = balanceBefore2 - balanceAfter2;

            // 3rd time - Reward reduced by 50%
            const balanceBefore3 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            await program.methods.claimReward(activityType).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            const balanceAfter3 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            reward3 = balanceBefore3 - balanceAfter3;

            assert.approximately(reward3, reward1 / 2, 100, "Third reward should be approximately 50% of the first");
        });

        it("Switch Task After 2 Repetitions - Reward Resets", async () => {
            const farmTask = TASK_TYPES.VoteInPoll;
            const switchTask = TASK_TYPES.ViewAnalytics;
            let rewardFarm1: number, rewardFarm2: number, rewardSwitch: number;

            // 1st time Farm Task
            const balanceBeforeFarm1 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            await program.methods.claimReward(farmTask).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            const balanceAfterFarm1 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            rewardFarm1 = balanceBeforeFarm1 - balanceAfterFarm1;

            // 2nd time Farm Task
            const balanceBeforeFarm2 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            await program.methods.claimReward(farmTask).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            const balanceAfterFarm2 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            rewardFarm2 = balanceBeforeFarm2 - balanceAfterFarm2;

            // Switch task
            const balanceBeforeSwitch = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            await program.methods.claimReward(switchTask).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            const balanceAfterSwitch = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            rewardSwitch = balanceBeforeSwitch - balanceAfterSwitch;

            assert.approximately(rewardSwitch, rewardFarm1, 100, "Switched task reward should be approximately equal to normal reward");
        });

        it("Repeat Same Task Continuously - Reward Halving", async () => {
            const activityType = TASK_TYPES.LeaveFeedback;
            let reward3: number, reward4: number, reward5: number, reward2: number;

            // 2nd time - Normal reward (needed for comparison)
            const balanceBefore2 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            await program.methods.claimReward(activityType).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            const balanceAfter2 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            reward2 = balanceBefore2 - balanceAfter2;

            // 3rd time - 50% reduction
            const balanceBefore3 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            await program.methods.claimReward(activityType).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            const balanceAfter3 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            reward3 = balanceBefore3 - balanceAfter3;

            // 4th time - Further 50% reduction
            const balanceBefore4 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            await program.methods.claimReward(activityType).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            const balanceAfter4 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            reward4 = balanceBefore4 - balanceAfter4;

            // 5th time - Further 50% reduction
            const balanceBefore5 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            await program.methods.claimReward(activityType).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            const balanceAfter5 = (await program.account.programState.fetch(programStateAccount)).currentSolBalance.toNumber();
            reward5 = balanceBefore5 - balanceAfter5;

            assert.approximately(reward3, reward2 / 2, 100, "Third reward should be 50% of normal");
            assert.approximately(reward4, reward3 / 2, 100, "Fourth reward should be 50% of third reward");
            assert.approximately(reward5, reward4 / 2, 100, "Fifth reward should be 50% of fourth reward");
        });
    });

    describe("RNG Task Availability Tests", () => {
        it("Task availability changes after randomize_tasks call", async () => {
            const initialProgramState = await program.account.programState.fetch(programStateAccount);
            const initialAvailableTasks = initialProgramState.availableTasks;

            await program.methods.randomizeTasks().accounts({
                programState: programStateAccount,
                authority: authority.publicKey,
            }).signers([authority]).rpc();

            const finalProgramState = await program.account.programState.fetch(programStateAccount);
            const finalAvailableTasks = finalProgramState.availableTasks;

            assert.notEqual(initialAvailableTasks, finalAvailableTasks, "Task availability should change after randomize_tasks");
        });
    });


    describe("User Cooldown Test", () => {
        it("User must wait 5 seconds before claiming again", async () => {
            const activityType = TASK_TYPES.CheckIn;

            // Claim reward successfully
            await program.methods.claimReward(activityType).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();

            // Attempt to claim reward immediately again - should fail due to cooldown
            let cooldownError: Error | null = null;
            try {
                await program.methods.claimReward(activityType).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            } catch (err) {
                cooldownError = err as Error;
            }

            assert.isNotNull(cooldownError, "Second claim should have resulted in an error due to cooldown");
            assert.include(cooldownError?.message || '', "Cooldown is active", "Error message should indicate cooldown active");

            // Wait for 6 seconds (5-second cooldown + 1 second buffer)
            await new Promise(resolve => setTimeout(resolve, 6000));

            // Claim reward again after cooldown - should succeed
            let claimAfterCooldownError: Error | null = null;
            try {
                await program.methods.claimReward(activityType).accounts(getClaimAccounts(programStateAccount, userRewardAccount, authority, user)).signers([authority, userRewardAccount]).rpc();
            } catch (err) {
                claimAfterCooldownError = err as Error;
            }
            assert.isNull(claimAfterCooldownError, "Claim after cooldown should succeed");
        });
    });

});


// Helper function to get accounts for claimReward instruction (DRY principle)
const getClaimAccounts = (
    programStateAccount: anchor.web3.PublicKey,
    userRewardAccount: anchor.web3.Keypair,
    authority: anchor.web3.Keypair,
    user: anchor.web3.PublicKey,
): { [key: string]: anchor.web3.PublicKey } => ({
    programState: programStateAccount,
    userRewardAccount: userRewardAccount.publicKey,
    user,
    authority: authority.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
});
