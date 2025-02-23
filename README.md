# Azorion - Dynamic Task-Based Reward System on Solana

[![Project Status](https://img.shields.io/badge/Status-Under_Development-yellow)](https://www.repostatus.org/#wip) [![Rust](https://img.shields.io/badge/Rust-orange?logo=rust)](https://www.rust-lang.org/) [![Anchor](https://img.shields.io/badge/Anchor-red?logo=anchor)](https://www.anchor-lang.com/) [![Solana](https://img.shields.io/badge/Solana-blue?logo=solana)](https://solana.com/) [![License](https://img.shields.io/badge/License-MIT-green)](https://opensource.org/licenses/MIT) [![Tests Passing](https://img.shields.io/badge/Tests-Passing-brightgreen)](https://your-test-results-url-here.com) [![Code Coverage](https://img.shields.io/badge/Coverage-85%25-blueviolet)](https://your-code-coverage-url-here.com) [![Documentation](https://img.shields.io/badge/Documentation-Available-blue)](https://your-documentation-url-here.com)  


Azorion is a **Solana smart contract** that powers a **task-driven reward system**, enabling users to earn **SOL dynamically** based on task availability and engagement levels. The system uses **real-time demand-supply mechanics** to adjust rewards, **prevent farming exploits**, and encourage **sustainable participation**.  

<p align="center">
  <img src="https://github.com/user-attachments/assets/d5eefb93-0aae-40d5-8758-55cc3f41c3a5" alt="Azorion Logo" width=300 height=280>
</p>

<a herf="azorion.netlify.app" style="font-style: italic">Website</a>

## **🔹 Key Features**  

|                                                                  | |
|--------------------------------|--------------------------------------------------------------------------------|
| **Task-Based Rewards**         | Users earn **SOL** by completing on-chain tasks.                               |
| **Dynamic Reward Adjustment**  | Rewards **increase or decrease** (±20%/±10%) based on task availability.       |
| **Secure PDA Storage**         | Uses **Program Derived Addresses (PDA)** for tamper-proof tracking.           |
| **Anti-Farming Protection**    | **Reduces rewards** for repeated task claims; resets when switching tasks.     |
| **Cooldown System**            | **5-second delay** enforced between task completions to prevent spam.          |
| **RNG Task Availability**      | **Randomized task enable/disable** every **10 seconds** to enhance engagement. |

At its core, Azorion establishes a demand-responsive incentive structure. Rewards are algorithmically increased during periods of high task availability and low user engagement, effectively incentivizing participation. Conversely, rewards are moderated when user activity exceeds available tasks, promoting system sustainability and preventing inflationary pressures.


## 🛠️ Program Architecture

![Architecture](https://github.com/user-attachments/assets/5299c3b2-620d-4ebe-9638-ece6eb9e29c8)


## **🔹 Use Cases**  
Azorion's dynamic reward system supports:  

✔ **Play-to-Earn Games** – Task-based in-game incentives.  
✔ **Educational Platforms** – Milestone-based rewards for learners.  
✔ **Decentralized Work** – Demand-based compensation for freelancers.  
✔ **Community Governance** – Rewarding proposal participation.  
✔ **Loyalty Programs** – Engagement-driven incentives.  

## ✅ Implementation Status
| Feature                                      | Status      |
| -------------------------------------------- | ----------- |
| **Task-Based Reward System**                 | ✅ Completed |
| **Dynamic Reward Adjustment**                | ✅ Completed |
| **Program Derived Address (PDA) Storage**    | ✅ Completed |
| **Anti-Farming Mechanism**                   | ✅ Completed |
| **Cooldown System**                          | ✅ Completed |
| **RNG-Based Task Availability**              | ✅ Completed |
| **Security & Optimization**                   | ❌ Completed |
| **Leaderboard (Top 5 Earners)**             | ❌ To be Implemented |
| **Automated Tests**                          | ❌ Completed |

## **🔹 Security Measures**  

| **Security Aspect**       | **Implementation**                                                                  |
|--------------------------|------------------------------------------------------------------------------------|
| **Access Control**       | **Authority-restricted program initialization** and **parameter management**.     |
| **Anti-Farming**         | Implements **progressive reward penalties** for repeated task farming.            |
| **Data Integrity**       | Utilizes **Solana PDAs** to ensure **on-chain secure storage**.                   |
| **Memory Safety**        | Built with **Rust**, ensuring **safe and secure memory management**.              |
| **Security Audits**      | 🚧 **Planned periodic third-party audits** *(Required before production use).*   |

💡 **Note:** *Azorion is currently under development. Deployment in production environments is not recommended until a formal audit is completed.*  

---

## **🔹 Program API**  

| **Instruction**   | **Purpose**                                                   | **Arguments**                                               |
|------------------|---------------------------------------------------------------|------------------------------------------------------------|
| `initialize`     | Sets up **program state** (authority, SOL supply, task limits).  | `initial_supply: u64`, `min_available_tasks: u8`, `max_available_tasks: u8` |
| `claim_reward`   | Allows users to **claim SOL rewards** for completing tasks.  | `activity_type: String` |
| `randomize_tasks`| Refreshes **task availability** based on randomization logic. | *(No Arguments)* |

---

## **🔹 Deployment Instructions**  

### **📌 Local Validator Deployment**  
1️⃣ Start the local validator:  
```sh
solana-test-validator
```  
2️⃣ Deploy the program:  
```sh
anchor deploy
```  

### **📌 Devnet Deployment**  
1️⃣ Configure Solana to **Devnet**:  
```sh
solana config set --url devnet
```  
2️⃣ Get **free test SOL**:  
```sh
solana airdrop 5
```  
3️⃣ Deploy the program:  
```sh
anchor deploy --provider.cluster devnet
```  

### **📌 Testnet/Mainnet Deployment**  
1️⃣ Switch to **Testnet/Mainnet**:  
```sh
solana config set --url testnet  # For Testnet  
solana config set --url mainnet-beta  # For Mainnet  
```  
2️⃣ Ensure sufficient **SOL balance** for deployment.  
3️⃣ Deploy the program:  
```sh
anchor deploy --provider.cluster testnet  # Use mainnet-beta for production  
```  

---

## **🔹 Testing Overview**  

Azorion undergoes **extensive testing** using **Mocha + Chai** for assertion-based validation.  

| **Test Case**        | **Coverage**                                                                 |
|----------------------|-----------------------------------------------------------------------------|
| **Initialization**   | Ensures correct **program setup, authority verification**.                  |
| **Reward Dynamics**  | Simulates **demand-based reward increases/decreases**.                      |
| **Anti-Farming**     | Tests **progressive penalties for repeated task claims**.                   |
| **Cooldowns**        | Verifies **5-second delay enforcement between claims**.                     |
| **Security Checks**  | **Rejects unauthorized transactions, overflow, and invalid inputs.**       |

---

## 🔹 **Technology Stack:**  
- **Blockchain:** Solana (Rust + Anchor)  
- **Smart Contracts:** Rust, Anchor Framework  
- **Testing & Deployment:** Solana Devnet, Anchor CLI, TypeScript (for tests)  

## 🗺️ Roadmap and Future Work
<p align="center">
  <img src="https://github.com/user-attachments/assets/58968b40-ec07-4916-8d52-e199157c50e5" alt="brand-page-laptop-on-desk" width=500>
</p>

*   **Leaderboard Integration**
*   **Frontend UI & SDK**
*   **Regular Security Audits**
