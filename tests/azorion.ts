import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Azorion } from "../target/types/azorion"; // Updated program name
import { assert } from "chai";

describe("azorion_task_reward_program_tests", () => {
  // Updated describe block name
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Azorion as Program<Azorion>; // Updated program name
  const provider = anchor.getProvider();

  let programStateAccount: anchor.web3.PublicKey;
  let authority: anchor.web3.Keypair;
  let userRewardAccount: anchor.web3.Keypair;
  let user: anchor.web3.PublicKey;
  let programStateBump: number;
  let userBump: number;

  before(async () => {
    // Setup before all tests
    authority = anchor.web3.Keypair.generate();

    // Airdrop to authority
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        authority.publicKey,
        10 * anchor.web3.LAMPORTS_PER_SOL
      ),
      "processed"
    );

    userRewardAccount = anchor.web3.Keypair.generate();

    [programStateAccount, programStateBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("program_state")],
        program.programId
      );

    [user, userBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("user"), userRewardAccount.publicKey.toBuffer()],
      program.programId
    );

    const initialSupply = new anchor.BN(10);
    const minTasks = 5;
    const maxTasks = 10;

    await program.methods
      .initialize(initialSupply, minTasks, maxTasks)
      .accounts({
        programState: programStateAccount,
        authority: authority.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([authority])
      .rpc();
  });

  it("Initialize Program State", async () => {
    const state = await program.account.programState.fetch(programStateAccount);
    assert.ok(state.authority.equals(authority.publicKey));
  });

  describe("Reward Calculation Scenarios", () => {
    it("More Tasks Than Users (High Demand, Low Supply) - Reward Increase", async () => {
      // Simulate more tasks than users (programmatically in contract logic for now)
      // In a real scenario, you would control the 'available_tasks' in program state externally or through mock

      const initialProgramState = await program.account.programState.fetch(
        programStateAccount
      );
      const initialBalance = initialProgramState.currentSolBalance.toNumber();

      const activityStr: string = "check-in";
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();

      const finalProgramState = await program.account.programState.fetch(
        programStateAccount
      );
      const finalBalance = finalProgramState.currentSolBalance.toNumber();

      const rewardGiven = initialBalance - finalBalance;
      const baseReward = 0.01 * 1_000_000; // Base reward for "check-in"
      const expectedMaxReward = baseReward * 1.2; // Up to 20% increase

      assert.isAbove(rewardGiven, baseReward, "Reward should be increased");
      assert.isAtMost(
        rewardGiven,
        expectedMaxReward,
        "Reward increase should be within 20%"
      );
    });

    it("More Users Than Tasks (Low Demand, High Supply) - Reward Decrease", async () => {
      // Simulate more users than tasks (programmatically in contract logic for now)
      // In a real scenario, you would control the 'available_tasks' externally or through mock

      const initialProgramState = await program.account.programState.fetch(
        programStateAccount
      );
      const initialBalance = initialProgramState.currentSolBalance.toNumber();

      const activityStr: string = "vote in a poll"; // Basic task with 0.01 SOL base reward
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();

      const finalProgramState = await program.account.programState.fetch(
        programStateAccount
      );
      const finalBalance = finalProgramState.currentSolBalance.toNumber();
      const rewardGiven = initialBalance - finalBalance;

      const baseReward = 0.01 * 1_000_000; // Base reward for "vote in a poll"
      const expectedMinReward = baseReward * 0.9; // Up to 10% decrease

      assert.isBelow(rewardGiven, baseReward, "Reward should be decreased");
      assert.isAtLeast(
        rewardGiven,
        expectedMinReward,
        "Reward decrease should be within 10%"
      );
    });

    it("Balanced Demand-Supply (Equal Users and Tasks) - Reward Unchanged", async () => {
      // Simulate balanced demand-supply (programmatically in contract logic for now)
      // In a real scenario, you'd control 'available_tasks' and 'users' count

      const initialProgramState = await program.account.programState.fetch(
        programStateAccount
      );
      const initialBalance = initialProgramState.currentSolBalance.toNumber();

      const activityStr: string = "complete a profile setup"; // Basic task, 0.01 SOL base
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();

      const finalProgramState = await program.account.programState.fetch(
        programStateAccount
      );
      const finalBalance = finalProgramState.currentSolBalance.toNumber();
      const rewardGiven = initialBalance - finalBalance;

      const baseReward = 0.01 * 1_000_000; // Base reward for "complete profile setup"

      assert.equal(rewardGiven, baseReward, "Reward should remain unchanged");
    });
  });

  describe("Anti-Farming Mechanism Tests", () => {
    it("Repeat Same Task 3 Times - Reward Reduced by 50%", async () => {
      const activityStr: string = "vote in a poll"; // Use "Vote in a Poll" for farming tests
      let reward1, reward2, reward3;

      // 1st time - Normal reward
      const balanceBefore1 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();
      const balanceAfter1 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      reward1 = balanceBefore1 - balanceAfter1;

      // 2nd time - Normal reward
      const balanceBefore2 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();
      const balanceAfter2 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      reward2 = balanceBefore2 - balanceAfter2;

      // 3rd time - Reward reduced by 50%
      const balanceBefore3 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();
      const balanceAfter3 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      reward3 = balanceBefore3 - balanceAfter3;

      assert.equal(
        reward1,
        reward2,
        "First two rewards should be equal (normal)"
      );
      assert.approximately(
        reward3,
        reward1 / 2,
        100,
        "Third reward should be approximately 50% of the first"
      ); // Using approximately for potential slight variations
    });

    it("Switch Task After 2 Repetitions - Reward Resets to Normal", async () => {
      const farmTask: string = "vote in a poll";
      const switchTask: string = "view analytics";
      let rewardFarm1, rewardFarm2, rewardSwitch;

      // 1st time Farm Task - Normal reward
      const balanceBeforeFarm1 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      await program.methods
        .claimReward(farmTask)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();
      const balanceAfterFarm1 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      rewardFarm1 = balanceBeforeFarm1 - balanceAfterFarm1;

      // 2nd time Farm Task - Normal reward
      const balanceBeforeFarm2 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      await program.methods
        .claimReward(farmTask)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();
      const balanceAfterFarm2 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      rewardFarm2 = balanceBeforeFarm2 - balanceAfterFarm2;

      // Switch to Switch Task - Reward should be normal again
      const balanceBeforeSwitch = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      await program.methods
        .claimReward(switchTask)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();
      const balanceAfterSwitch = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      rewardSwitch = balanceBeforeSwitch - balanceAfterSwitch;

      assert.equal(
        rewardFarm1,
        rewardFarm2,
        "First two farm task rewards should be equal"
      );
      assert.approximately(
        rewardSwitch,
        rewardFarm1,
        100,
        "Switched task reward should be approximately normal again"
      ); // Compare to normal farm reward
    });

    it("Repeat Same Task Continuously After 3 Times - Reward Continues Reducing by 50%", async () => {
      const activityStr: string = "leave feedback on a dapp"; // Choose "Leave Feedback" for continuous farming test
      let reward1, reward2, reward3, reward4, reward5;

      // 1st time - Normal reward
      const balanceBefore1 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();
      const balanceAfter1 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      reward1 = balanceBefore1 - balanceAfter1;

      // 2nd time - Normal reward
      const balanceBefore2 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();
      const balanceAfter2 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      reward2 = balanceBefore2 - balanceAfter2;

      // 3rd time - 50% reduction
      const balanceBefore3 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();
      const balanceAfter3 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      reward3 = balanceBefore3 - balanceAfter3;

      // 4th time - Additional 50% reduction (halving again)
      const balanceBefore4 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();
      const balanceAfter4 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      reward4 = balanceBefore4 - balanceAfter4;

      // 5th time - Further 50% reduction (halving again)
      const balanceBefore5 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();
      const balanceAfter5 = (
        await program.account.programState.fetch(programStateAccount)
      ).currentSolBalance.toNumber();
      reward5 = balanceBefore5 - balanceAfter5;

      assert.equal(reward1, reward2, "First two rewards normal");
      assert.approximately(
        reward3,
        reward1 / 2,
        100,
        "Third reward 50% reduced"
      );
      assert.approximately(
        reward4,
        reward3 / 2,
        100,
        "Fourth reward further 50% reduced"
      );
      assert.approximately(
        reward5,
        reward4 / 2,
        100,
        "Fifth reward further 50% reduced"
      );
    });
  });

  describe("User Cooldown Test", () => {
    it("User must wait 5 seconds before claiming again", async () => {
      const activityStr: string = "check-in";

      // Claim reward successfully
      await program.methods
        .claimReward(activityStr)
        .accounts({
          programState: programStateAccount,
          userRewardAccount: userRewardAccount.publicKey,
          user: user,
          authority: authority.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([authority, userRewardAccount])
        .rpc();

      // Attempt to claim reward immediately again - should fail due to cooldown
      let cooldownError = null;
      try {
        await program.methods
          .claimReward(activityStr)
          .accounts({
            programState: programStateAccount,
            userRewardAccount: userRewardAccount.publicKey,
            user: user,
            authority: authority.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([authority, userRewardAccount])
          .rpc();
      } catch (err) {
        cooldownError = err;
      }

      assert.isNotNull(
        cooldownError,
        "Second claim should have resulted in an error"
      );
      assert.include(
        cooldownError.message,
        "Cooldown is active",
        "Error message should indicate cooldown"
      );

      // Wait for 5 seconds + a bit more to be sure
      await new Promise((resolve) => setTimeout(resolve, 6000));

      // Claim reward again after cooldown - should succeed
      let claimAfterCooldownError = null;
      try {
        await program.methods
          .claimReward(activityStr)
          .accounts({
            programState: programStateAccount,
            userRewardAccount: userRewardAccount.publicKey,
            user: user,
            authority: authority.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([authority, userRewardAccount])
          .rpc();
      } catch (err) {
        claimAfterCooldownError = err;
      }

      assert.isNull(
        claimAfterCooldownError,
        "Claim after cooldown should succeed"
      );
    });
  });

  // RNG-Based Task Availability and Dynamic User Generation tests are more complex to simulate
  // and might require more sophisticated test setup, time manipulation, and user simulation.
  // Basic functionality of reward dynamics and anti-farming are covered in the above tests.
});
