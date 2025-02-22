# Azorion - Dynamic Task-Based Reward System on Solana

[![Project Status](https://img.shields.io/badge/Status-Under_Development-yellow)](https://www.repostatus.org/#wip) [![Rust](https://img.shields.io/badge/Rust-orange?logo=rust)](https://www.rust-lang.org/) [![Anchor](https://img.shields.io/badge/Anchor-red?logo=anchor)](https://www.anchor-lang.com/) [![Solana](https://img.shields.io/badge/Solana-blue?logo=solana)](https://solana.com/) [![License](https://img.shields.io/badge/License-MIT-green)](https://opensource.org/licenses/MIT) [![Tests Passing](https://img.shields.io/badge/Tests-Passing-brightgreen)](https://your-test-results-url-here.com) <!-- Replace with your actual test results URL --> [![Code Coverage](https://img.shields.io/badge/Coverage-85%25-blueviolet)](https://your-code-coverage-url-here.com) <!-- Replace with your actual code coverage URL --> [![Documentation](https://img.shields.io/badge/Documentation-Available-blue)](https://your-documentation-url-here.com) <!-- Replace if you have dedicated documentation -->


**Azorion** is a Solana smart contract designed to revolutionize reward distribution by enabling a dynamic task-based earning system. Users can engage in predefined activities and earn SOL rewards that fluctuate in real-time, adapting to the balance of task demand and user supply. This decentralized approach prevents farming abuse, making it ideal for a wide range of applications from play-to-earn gaming to decentralized work platforms.

<p align="center">
  <img src="https://github.com/user-attachments/assets/d5eefb93-0aae-40d5-8758-55cc3f41c3a5" alt="Azorion Logo" width=300 height=280>
</p>

At its core, Azorion establishes a demand-responsive incentive structure. Rewards are algorithmically increased during periods of high task availability and low user engagement, effectively incentivizing participation. Conversely, rewards are moderated when user activity exceeds available tasks, promoting system sustainability and preventing inflationary pressures.

To safeguard against manipulation, Azorion integrates robust anti-farming protocols and a cooldown mechanism, maintaining system integrity and equitable access for all users. User reward data is securely managed via Program Derived Addresses (PDAs), leveraging Solana's inherent security to guarantee data integrity and user privacy.

## Key Highlights

| Feature                       | Description                                                                                                |
| :---------------------------- | :----------------------------------------------------------------------------------------------------------- |
| **✅Task-Based Rewards**        | Users earn SOL by completing predefined tasks across different categories.                                |
| **✅Dynamic Reward Adjustment** | Rewards automatically adjust (±20%/±10%) based on the ratio of tasks to users (Demand-Supply Logic).        |
| **✅PDA User Storage**          | Secure, on-chain storage of user data (history, rewards) using Program Derived Addresses.                 |
| **✅Anti-Farming Mechanism**    | Progressive reward reduction (halving rewards) for repeated tasks; resets upon task switch.                 |
| **✅Cooldown System**           | 5-second cooldown period after each task completion to prevent spamming.                                  |
| **✅RNG Task Availability**     | Tasks are randomly flagged as available/unavailable every 10 seconds for dynamic engagement.                |

## 🔹 **Technology Stack:**  
- **Blockchain:** Solana (Rust + Anchor)  
- **Smart Contracts:** Rust, Anchor Framework  
- **Testing & Deployment:** Solana Devnet, Anchor CLI, TypeScript (for tests)  

##  Use Cases

Azorion's dynamic reward system is ideal for:

*   **Play-to-Earn Games:** Dynamic in-game rewards based on player activity.
*   **Learning Platforms:** Incentivize students with adjustable rewards for course milestones.
*   **🌐 Decentralized Work:** Fair, demand-based compensation for freelance tasks.
*   **🗳️ Community Governance:** Reward participation in proposals and community actions.
*   **🤝 Loyalty Programs:** Dynamic rewards to enhance customer engagement and retention.
*   **Data Curation:** Incentivize data contribution with flexible, scalable rewards.

## ✅ Implementation Status
| Feature                                      | Status      |
| -------------------------------------------- | ----------- |
| **Task-Based Reward System**                 | ✅ Completed |
| **Dynamic Reward Adjustment**                | ✅ Completed |
| **Program Derived Address (PDA) Storage**    | ✅ Completed |
| **Anti-Farming Mechanism**                   | ✅ Completed |
| **Cooldown System**                          | ✅ Completed |
| **RNG-Based Task Availability**              | ✅ Completed |
| **Security & Optimization**                   | ✅ Completed |
| **Leaderboard (Top 5 Earners)**             | ❌ To be Implemented |
| **Automated Tests**                          | ✅ Completed |

## 🧪 Testing Highlights

*   **Comprehensive Suite:** Tests cover core functionalities: reward dynamics, anti-farming, cooldown, initialization.
*   **Scenario-Based:** Tests simulate different user and task availability conditions.
*   **Assertion-Driven:** Uses `chai` assertions for clear validation of contract behavior.
*   **Reliability Focused:** Ensures robustness and predictability of reward system logic.

| Test Category                  | Coverage                                                                                                      |
| :----------------------------- | :------------------------------------------------------------------------------------------------------------ |
| **Initialization**             | Program state setup, authority verification                                                                 |
| **Reward Dynamics**            | High/Low demand scenarios, balanced supply, reward increase/decrease/stability verification                  |
| **Anti-Farming**              | Reward reduction on repeated tasks, reset on task switch, continuous penalty application                       |
| **Cooldown**                   | Enforcement of 5-second cooldown period, successful claims after cooldown                                     |

## 🛡️ Security Measures

| Security Aspect           | Implementation                                                                                                 |
| :------------------------ | :------------------------------------------------------------------------------------------------------------- |
| **Access Control**        | Authority-based program initialization & parameter management; user-limited interactions.                        |
| **Anti-Farming**          | Reward penalties & cooldowns to prevent automated exploitation.                                                |
| **Data Integrity**        | PDAs for secure, program-controlled user data storage.                                                           |
| **Memory Safety**         | Rust development for inherent memory safety, reducing common vulnerabilities.                                    |
| **Security Audits**       | *(Planned)* Regular third-party audits to identify and mitigate potential risks.                               |

**Disclaimer:** Azorion is under development and not yet audited. Use in production without audit is discouraged. Smart contracts involve inherent risks.

## Program API

| Instruction        | Purpose                                                                    | Arguments                                                                        |
| :----------------- | :------------------------------------------------------------------------- | :------------------------------------------------------------------------------- |
| `initialize`       | Set up program state (authority, initial SOL, task availability range).      | `initial_supply: u64`, `min_available_tasks: u8`, `max_available_tasks: u8` |
| `claim_reward`     | Claim SOL reward for completing a specific task.                            | `activity_type: String`                                                        |
| `randomize_tasks`  | Randomize number of available tasks (authority-only).                         | None                                                                             |


## 📦 Deployment Instructions

To deploy Azorion to different Solana networks:

### Local Validator Deployment

1.  **Start Local Validator:**  `solana-test-validator`
2.  **Deploy Program:** `anchor deploy` (Ensure Solana CLI is set to local validator URL)

### Devnet Deployment

1.  **Set Solana Config to Devnet:** `solana config set --url devnet`
2.  **Request Devnet SOL:** `solana airdrop 5` (Request SOL for your wallet if needed)
3.  **Deploy Program:** `anchor deploy --provider.cluster devnet`

### Testnet/Mainnet Deployment

*Deployment to Testnet or Mainnet requires careful consideration and thorough testing on Devnet/Testnet first. Ensure you have sufficient SOL in your wallet to cover deployment and transaction fees.*

1.  **Set Solana Config to Testnet/Mainnet:** `solana config set --url testnet` or `solana config set --url mainnet-beta`
2.  **Ensure Sufficient SOL Balance:** Fund your wallet with enough SOL for deployment and initial program operation.
3.  **Deploy Program:** `anchor deploy --provider.cluster testnet` or `anchor deploy --provider.cluster mainnet`


## 🗺️ Roadmap and Future Work
<p align="center">
  <img src="https://github.com/user-attachments/assets/58968b40-ec07-4916-8d52-e199157c50e5" alt="brand-page-laptop-on-desk" width=500>
</p>

*   **Leaderboard Integration**
*   **Frontend UI & SDK**
*   **Governance Module**
*   **Cross-Chain Expansion**
*   **Regular Security Audits**
