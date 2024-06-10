# **Kintsu Audit Competition on Hats.finance** 


## Introduction to Hats.finance


Hats.finance builds autonomous security infrastructure for integration with major DeFi protocols to secure users' assets. 
It aims to be the decentralized choice for Web3 security, offering proactive security mechanisms like decentralized audit competitions and bug bounties. 
The protocol facilitates audit competitions to quickly secure smart contracts by having auditors compete, thereby reducing auditing costs and accelerating submissions. 
This aligns with their mission of fostering a robust, secure, and scalable Web3 ecosystem through decentralized security solutions​.

## About Hats Audit Competition


Hats Audit Competitions offer a unique and decentralized approach to enhancing the security of web3 projects. Leveraging the large collective expertise of hundreds of skilled auditors, these competitions foster a proactive bug hunting environment to fortify projects before their launch. Unlike traditional security assessments, Hats Audit Competitions operate on a time-based and results-driven model, ensuring that only successful auditors are rewarded for their contributions. This pay-for-results ethos not only allocates budgets more efficiently by paying exclusively for identified vulnerabilities but also retains funds if no issues are discovered. With a streamlined evaluation process, Hats prioritizes quality over quantity by rewarding the first submitter of a vulnerability, thus eliminating duplicate efforts and attracting top talent in web3 auditing. The process embodies Hats Finance's commitment to reducing fees, maintaining project control, and promoting high-quality security assessments, setting a new standard for decentralized security in the web3 space​​.

## Kintsu Overview

Next generation liquid staking infrastructure

## Competition Details


- Type: A public audit competition hosted by Kintsu
- Duration: 2 weeks
- Maximum Reward: $52,000
- Submissions: 66
- Total Payout: $52,000 distributed among 16 participants.

## Scope of Audit

## Project overview

Kintsu sAZERO is a Liquid Staking Protocol for Aleph Zero. The protocol is fully non-custodial. Staking happens directly from the Vault smart contract via nomination agents that leverage novel Substrate runtime calls to programatically bond and unbond from the pallet Nomination Pools. This closed loop solution accrues staking yield to the vault which the sAZERO LST is effectively a share of. 

Beyond making staking effectively liquid for the community & builders alike, the protocol makes staking yield composably by abstracting away the complexity. All you need to do is hold or collateralize the sAZERO token in a contract, "stake and use."

Most importantly the community is in control of the registry and which validators get a percentage of Kintsu's staking pool. It is set up to become fully permissionless in the future with roles that can progressively be given to governance. This means that even as the LST absorbs a strong amount of the Gas token into TVL, there is no concern about a central party choosing the stake or threatening the network's decentralization/security. 

We have full documentation here https://docs.kintsu.xyz/

## Audit competition scope

In Scope 

- nomination_agent
- registry
- share_token
- vault

## High severity issues


- **Duplicate Batch ID Entries Can Lead to Withdrawal Process Errors in AZERO Staking**

  Users who stake AZERO tokens can withdraw their stakes through a multi-step process involving `request_unlock` and then `send_batch_unlock_requests` after a designated period. The smart contract checks whether a `batch_id` has already been unlocked and reverts if it was. However, a loophole allows users to send duplicate `batch_ids` in a single call, resulting in the `total_pooled` amount being decreased multiple times by the same withdrawal request amount. This issue can lead to `total_pooled` approaching near zero, causing future `send_batch_unlock_requests` calls to revert due to an overflow error when trying to decrease `total_pooled` by an amount greater than what remains.

A proof-of-concept test shows this vulnerability by staking a total of 5 million AZERO and then processing withdrawal requests with duplicated `batch_id`. Although Alice only intended to withdraw 500,000 AZERO, the test shows `total_pooled` decreased by 1,000,000, proving the method's inefficiency.

The recommendation is to prevent duplicate `batch_ids` in the `send_batch_unlock_requests` function to mitigate this issue. A user highlighted that once a batch request is processed, it should be removed from the list, but tried the provided proof-of-concept and confirmed the issue. The issue was addressed in an update, which ensures the incoming list is sorted, preventing the duplicate processing vulnerability. The fix has been reviewed and confirmed to resolve the problem by ensuring unique `batch_ids` during withdrawal requests.


  **Link**: [Issue #28](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/28)

## Medium severity issues


- **Inconsistent Unbonding Mechanism in Nomination Pools Module Causes Reversion Bug**

  The `nomination_pools` module uses the `staked` variable to manage agent participation in pools. The reduction of `self.staked` occurs during the `start_unbond` function rather than the `withdraw_unbonded` function, causing inconsistencies. This allows an agent to attempt to rejoin a pool after being unbonded but not withdrawn, potentially leading to a denial-of-service (DOS) vulnerability in the stake function. In a typical scenario, an agent joins a pool, gets unbonded but not withdrawn, resulting in `staked = 0`, and then attempts to rejoin, causing the system to revert. The suggested code revision introduces a `joined` variable to track agent status accurately, preventing rejoining if the agent is unbonded but not yet withdrawn, averting the DOS vulnerability.


  **Link**: [Issue #29](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/29)


- **Revert in One Agent’s Compound Call Causes Entire Function to Revert**

  The `delegate_compound` function iterates over all agents and calls their `compound` function. If any agent's `compound` function reverts, it causes the entire `delegate_compound` function to revert. This issue arises if an agent has not yet joined the nomination pool or has already reaped from it, leading to a potential Denial of Service (DOS) vulnerability. Two solutions are proposed: first, adding a check to ensure an agent has joined the pool before calling `compound`, and second, allowing the call to fail gracefully without reverting, similar to the `withdraw` function. The likelihood of a DOS attack is low but it can still be mitigated by these changes.


  **Link**: [Issue #30](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/30)


- **Stake Function Fails When Split Amounts Fall Below Minimum Agent Requirement**

  The minimum stake allowed through a nomination pool is 10 AZERO, verified by the `stake` function contract. However, this check should apply to each agent, not just the total stake amount. When staking amounts are split among multiple agents and each portion falls below the minimum required stake, the `stake` function may fail, leading to a Denial of Service (DoS) scenario. For example, if a user stakes 10 AZERO and there are two agents, each agent receives 5 AZERO, which is below the minimum stake requirement, causing the call to revert with a `CallRuntimeFailed` error. The solution is to ensure that the lowest amount allocated to any agent meets the minimum stake requirement.


  **Link**: [Issue #48](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/48)


- **Potential Loss of GasToken During Nomination Pools Unbonded Withdrawal in Destroy Mode**

  When a pool is set to destroy mode and the target is not the depositor, the `nomination-pools.withdraw_unbonded` function can be called by anyone. This process is consistent with the internal checks and results in the GasToken (AZERO) being transferred to the `member_account`, which in this context is the `nomination_agent`. If someone calls the `withdraw_unbonded` function before the vault does, the GasToken will be transferred to the `nomination_agent`. As a result, when the vault subsequently calls `withdraw_unbonded`, the process will yield no tokens (`withdrawn = after - before` will be zero), and the GasToken will not be transferred to the vault. This could potentially lead to a critical issue where the vault does not receive the intended GasToken.


  **Link**: [Issue #50](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/50)


- **Denial of Service in Stake and Compound Functions Due to Blocked Pools**

  Nomination agents linked to pools that are in a destroying or blocked state will cause the `compound` and `stake` functions to revert. Attempts to bond extra or join these pools will fail, leading to a denial of service (DoS) for these functions. If one of the pools transitions to such a state, any further attempts to interact with it will also revert, causing failures in `compound` and `stake`. To avoid this, it is suggested to check the state of each associated pool before attempting to bond extra or join. If the pool is in a destroying or blocked state, the function should skip that nomination agent to prevent DoS.


  **Link**: [Issue #51](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/51)


- **Incorrect Delegate Unbonding Calculation Causes DOS in Send Batch Unlock Requests**

  Agents must either fully unbond or retain at least the minimum required bond (`minimum_stake`) after using the `unbond` function. The current `delegate_unbonding` implementation neglects this rule, creating situations where a single agent's unbonding causes the `send_batch_unlock_requests` function to fail, resulting in a denial of service (DOS). Specifically, if an unbonding attempt leaves an agent's stake between 0 and 10 AZERO, the function reverts. This DOS vulnerability can be resolved by modifying the `delegate_unbonding` function to verify that each agent's remaining stake after unbonding is either zero or at least the `minimum_stake`. Any unbonding operation that would leave an agent's stake in the problematic range should be adjusted accordingly.


  **Link**: [Issue #61](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/61)


- **Unbond Operation Does Not Sync Staked Value, Causing Potential DOS Issues**

  An unbonding process initiates a function that decreases the `staked` value. The issue arises because the unbond operation can be permissionlessly triggered by anyone when the pool is being destroyed, and the member is not the depositor. This causes the `staked` value to become unsynced.

The outdated `staked` value affects subsequent processes. When the `get_staked_value` method fetches this unsynced value, it adversely impacts `delegate_unbonding` allocation, potentially causing it to revert and creating a denial-of-service (DOS) scenario. 

To fix this, the system should fetch the actual `staked` values directly from the pool during such operations to ensure accuracy and prevent these issues.

In conclusion, both issues (#50 and #63) have the same root cause—permissionless action when the pool is in destroy mode. Proper mitigation would address both.


  **Link**: [Issue #63](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/63)

## Low severity issues


- **Add Events to nomination_agent Functions for Increased Transparency and Information**

  The `nomination_agent` contract contains functions that change the staked token balance, such as `deposit()`, `start_unbond()`, `withdraw_unbonded()`, and `compound()`. However, these functions do not emit events, which reduces transparency and complicates tracking changes off-chain. Emitting events for these functions would improve transparency and user trust.


  **Link**: [Issue #4](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/4)


- **Implementing `add_agent` Function Exceeds Storage Cell Limit for Agent Entries**

  The `add_agent` function in the system adds agents to a `Vec` limited to one storage cell (~16MB). This restricts the number of agents, causing the function to panic if the limit is exceeded. A recommended solution is to use ink! 5's `StorageVec` or replace the `Vec` with `Mapping` structures.


  **Link**: [Issue #9](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/9)


- **Use 365.25 Days for More Accurate Year Calculation in Staking Rewards**

  In the given Rust code, the constant `YEAR` is defined as `DAY * 365`, which may not be accurate for precise calculations. It is recommended to use `DAY * 365.25` instead, aligning with the standard used in AZERO staking rewards to improve precision.


  **Link**: [Issue #11](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/11)


- **Enhance remove_agent function to check for bonded AZERO tokens before removal**

  The `remove_agent` function only checks the agent's weight and does not verify whether the agent has bonded AZERO tokens. This omission could lead to the accidental removal of agents with bonded tokens, negatively impacting unstaking and compounding operations. It is recommended to enhance the function to check for bonded AZERO tokens before removing an agent.


  **Link**: [Issue #12](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/12)


- **Fee Accumulation in `update_fees` Function Doesn't Consider Owner's Virtual Shares**

  The current `update_fees` function does not account for the owner's virtual shares (`total_shares_virtual`), only considering `total_shares_minted`. This discrepancy causes the owner to lose some funds. The proposed code revision corrects this by including both `total_shares_minted` and `total_shares_virtual` in the fee calculation, ensuring a more accurate representation of the owner's shares.


  **Link**: [Issue #18](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/18)


- **NominationAgent Lacks Admin Transfer Ownership Function Causing Potential Upgrade Issues**

  The `NominationAgent` contract has an `admin` variable set at construction, which allows the admin to upgrade the contract. Unlike the vault contract, `NominationAgent` lacks a transfer ownership function, making it difficult to change admins without disruption. Implementing a transfer admin function is recommended to enhance functionality and ease of management.


  **Link**: [Issue #22](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/22)


- **Correct `vault::compound` event to emit accurate virtual shares at current time**

  The `vault::compound` function emits an event with an incorrect value for virtual shares, using `self.data.total_shares_virtual` instead of calculating the current virtual shares. This can mislead users or systems relying on event logs. The proposed fix calculates virtual shares at the current timestamp before emitting the event to ensure accuracy.


  **Link**: [Issue #56](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/56)



## Conclusion

The Hats.finance audit competition for Kintsu's liquid staking protocol, sAZERO, concluded successfully with 66 submissions, awarding a total payout of $52,000 among 16 participants. Kintsu's protocol aimed to secure its smart contracts by engaging a vast pool of auditors in a competitive, time-bound, results-driven audit process. The audit identified several issues ranging in severity from high to low. The most critical among these involved duplicate batch IDs causing withdrawal errors due to faulty handling of the batch unlocking process. Medium severity issues included inconsistent unbonding mechanisms that led to potential Denial of Service (DoS) vulnerabilities, incorrect calculations affecting staking, and the potential for unauthorized withdrawal in destroy mode. Additionally, low severity issues such as missing event emissions and inadequate function checks were highlighted. All identified issues had recommended fixes, with many already addressed, proving the effectiveness of Hats.finance's decentralized audit model in enhancing the security and robustness of DeFi protocols.

## Disclaimer


This report does not assert that the audited contracts are completely secure. Continuous review and comprehensive testing are advised before deploying critical smart contracts.


The Kintsu audit competition illustrates the collaborative effort in identifying and rectifying potential vulnerabilities, enhancing the overall security and functionality of the platform.


Hats.finance does not provide any guarantee or warranty regarding the security of this project. Smart contract software should be used at the sole risk and responsibility of users.

