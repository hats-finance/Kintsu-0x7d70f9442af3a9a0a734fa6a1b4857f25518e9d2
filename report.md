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


- **Duplicate Batch IDs in `send_batch_unlock_requests` Can Decrease 'total_pooled' Incorrectly**

  Staking AZERO allows users to withdraw their stakes by first calling `request_unlock` and then, after a waiting period, calling `send_batch_unlock_requests` to retrieve their tokens. The contract correctly checks if a `batch_id` was already unlocked to avoid processing it again. However, it has a flaw where users can send duplicate `batch_ids` in a single request. This results in the `total_pooled` amount being decreased multiple times by the same requested withdrawal amount. Consequently, the `total_pooled` can be brought near zero, causing future `send_batch_unlock_requests` to fail due to overflow errors. This would prevent users from withdrawing their properly requested amounts.

To demonstrate this issue, a test was conducted where multiple users staked a total of 5 million AZERO, and then `request_unlock` was called with duplicate `batch_ids`. The test showed that `total_pooled` was incorrectly reduced, leaving insufficient pooled tokens to cover legitimate withdrawal requests. The suggested fix is to prohibit duplicate `batch_ids` within the `send_batch_unlock_requests` function.

Discussions among developers and a follow-up fix have addressed this by ensuring that incoming requests are sorted, thus preventing the abuse of duplicate `batch_ids`. Subsequent reviews verified this fix as adequate.


  **Link**: [Issue #28](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/28)

## Medium severity issues


- **Inconsistency in Nomination Pools Leading to Potential DOS Vulnerability in Stake Function**

  The `nomination_pools` module uses the `staked` variable to determine whether an agent can join a nomination pool. The issue arises because the reduction of `self.staked` happens in the `start_unbond` function rather than in the `withdraw_unbonded` function. This discrepancy leads to a scenario where an agent, after being unbonded but not withdrawn, may attempt to rejoin a pool, causing a denial-of-service (DOS) vulnerability in the stake function. The proposed code revision introduces a `joined` variable to properly track whether an agent is eligible to rejoin the pool, ensuring they are not allowed to rejoin if they have been unbonded but not withdrawn, thus preventing the DOS vulnerability.


  **Link**: [Issue #29](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/29)


- **Improvements to Delegate Compound Function to Prevent Reverts from Individual Agent Failures**

  The `delegate_compound` function, which iterates over all agents and calls `compound` on them, faces a DOS vulnerability. If one of the agents has not yet joined the nomination pool or has already reaped from it, the `nomination_agent::compound` function will revert, causing the entire `delegate_compound` function to revert. This could disrupt the compounding process for all agents. To mitigate this, a check should be implemented to ensure that an agent has joined the nomination pool before calling the `compound` function, or the function should be modified to prevent reversion, similar to how the `withdraw` function operates. This will help prevent potential DOS attacks.


  **Link**: [Issue #30](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/30)


- **Ensure Minimum Stake Is Checked for Each Agent in Nomination Pool**

  The current implementation checks that the user's total stake meets the minimum requirement of 10 AZERO. However, this check should be performed for each agent rather than the total stake. This oversight can lead to a Denial of Service (DoS) scenario if the stake is divided into amounts below the minimum threshold for each individual agent. For example, if a user stakes 10 AZERO across two agents, each agent would receive 5 AZERO, failing the minimum stake requirement. This failure results in transaction reversion and a `CallRuntimeFailed` error. The solution proposed is to adjust the code to check the minimum stake for each agent's allocation rather than the overall user stake, to avoid such issues.


  **Link**: [Issue #48](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/48)


- **Potential Token Transfer Issue in Nomination Pools During Destroy Mode**

  When a pool is in destroy mode and the target is not the depositor, the `nomination-pools.withdraw_unbonded` function can be called by anyone. This is consistent with the `nomination-pools.ok_to_withdraw_unbonded_with` function. At the end of `nomination-pools.withdraw_unbonded`, the GasToken (AZERO) is transferred to the `member_account`, which in this context is `nomination_agent`. If an unauthorized party calls `nomination-pools.withdraw_unbonded` before the vault calls `nomination_agent.withdraw_unbonded`, the GasToken will already be in the `nomination_agent`. Consequently, when the vault attempts to `withdraw_unbonded`, it will register a zero withdrawal, meaning the GasToken won't be transferred to the vault.


  **Link**: [Issue #50](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/50)


- **Stake and Compound Functions Fail When Linked Pools Are Destroying or Blocked**

  Nomination agents linked to pools in a destroying or blocked state cause the `compound` and `stake` functions to revert, as attempts to bond extra or join these pools fail. This leads to a denial of service (DoS) for these functions, preventing users from staking or compounding their funds. To address this issue, it's recommended to check the state of each pool associated with a nomination agent before proceeding. If the pool is in a destroying or blocked state, the function should skip that agent. While some argue that reverting is expected, there's a consensus that these scenarios need to be mitigated, potentially by controlling the linked pools to prevent future issues.


  **Link**: [Issue #51](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/51)


- **Denial of Service in `send_batch_unlock_requests` Due to Improper Unbond Amount Handling**

  The `delegate_unbonding` function does not ensure that agents either fully unbond or maintain the minimum required bond (`minimum_stake`) after unbonding. This oversight can lead to the `send_batch_unlock_requests` function reverting if an agent falls below the required bond threshold during unbonding. Specifically, if the `send_batch_unlock_requests` function attempts to unbond an amount that results in an agent's stake being between 0 and 10 AZERO, the function will revert, causing a denial of service (DOS). For example, if there are 10 agents with a total stake of 1000 AZERO and a user attempts to unstake 200 AZERO, one agent's remaining stake may fall below the minimum required, leading to a function revert. The proposed solution is to adjust the unbond amount in `delegate_unbonding` to ensure that the remaining stake is either zero or at least `minimum_stake`.


  **Link**: [Issue #61](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/61)


- **Unbounded Staked Value Unsynchronized Causing DoS Risk During Permissionless Unbonding Operations**

  The unbounding process in the provided code allows unauthorized users to trigger the `unbond` operation when the pool is in a certain state, causing issues with the `staked` value. The `staked` value becomes unsynced when the unbond operation is called permissionlessly, especially if the pool is destroying and the caller is not the depositor. This discrepancy can affect functions that rely on the accurate `staked` value, such as `get_weight_imbalances` and `delegate_unbonding`, potentially leading to incorrect unbonding allocations or even denial-of-service scenarios. The recommendation is to fetch the `staked` data directly from the pool to ensure it remains updated. The root cause is identified as permissionless calling in specific situations, which also affects related issues.


  **Link**: [Issue #63](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/63)

## Low severity issues


- **Missing Event Emissions in Nomination Agent Functions for Transparency and Tracking**

  The `nomination_agent` contract lacks event emissions for functions like `deposit()`, `start_unbond()`, `withdraw_unbonded()`, and `compound()`, which alter staked token balances. Adding these events would enhance transparency and allow off-chain tools to reflect changes effectively. There's disagreement on this necessity, though event emission for `set_code()` is confirmed as needed.


  **Link**: [Issue #4](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/4)


- **"Function `add_agent` Limited by Packed Vec Storage Constraints in ink!"**

  The `add_agent` function adds new agents to a `Vec`, limited to the default size of one storage cell (~16MB), restricting the number of agents. Once the limit is reached, the function will panic. Moving to ink! 5's `StorageVec` or using `Mapping` structures could resolve this. However, adding too many agents can result in significant computation costs.


  **Link**: [Issue #9](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/9)


- **Adjust the `YEAR` Constant to Use 365.25 Days for Precision in Data.rs**

  The current calculation for a year multiplies the number of milliseconds in a day by 365, but for more precision, it should use 365.25 to account for leap years. AZERO staking rewards use this more precise value. The recommendation is to update the code to reflect this standard for accuracy in functions like `update_fees` and `get_virtual_shares_at_time`.


  **Link**: [Issue #11](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/11)


- **Enhance `remove_agent` function to check for bonded AZERO tokens before removal**

  The `remove_agent` function currently lacks a verification step to check if agents have bonded AZERO tokens, posing a risk of removing such agents unintentionally. This can negatively impact unstaking and compounding operations. It is recommended to enhance the function to ensure only agents without bonded tokens are removed.


  **Link**: [Issue #12](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/12)


- **Fee Accumulation Calculation Ignores Owner's Virtual Shares, Causing Revenue Loss**

  The `update_fees` function does not consider the owner's virtual shares (`total_shares_virtual`) in its fee accumulation calculation, leading to the owner losing some fees. The proposed code revision ensures that both `total_shares_minted` and `total_shares_virtual` are included, aligning with other share calculations and providing a more accurate representation of the owner's shares.


  **Link**: [Issue #18](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/18)


- **Implement Transfer Admin Functionality in NominationAgent for Improved Management**

  The `NominationAgent` contract has an `admin` variable set in its constructor, enabling the admin to upgrade the contract via the `set_code` function. Unlike the vault contract, `NominationAgent` lacks a function to transfer this admin ownership. Implementing such a function would simplify managing multiple nomination agents and prevent potential disruptions.


  **Link**: [Issue #22](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/22)


- **Incorrect Virtual Shares Emitted by `vault::compound` Function Requires Adjustment**

  The `vault::compound` function emits an event with outdated virtual share values. The current implementation directly references `self.data.total_shares_virtual` without recalculating at the emission time. This discrepancy can mislead users or systems depending on accurate event logs for virtual shares data. A proposed fix recalculates the virtual shares before emitting the event to ensure correctness.


  **Link**: [Issue #56](https://github.com/hats-finance/Kintsu-0x7d70f9442af3a9a0a734fa6a1b4857f25518e9d2/issues/56)



## Conclusion

The audit on Kintsu's sAZERO Liquid Staking Protocol, hosted by Hats.finance, identified significant vulnerabilities and areas of improvement. Key high-severity issues included a flaw in the `send_batch_unlock_requests` function that allowed duplicate batch IDs, leading to incorrect `total_pooled` values, which could hinder users from withdrawing their stakes. Medium-severity issues ranged from potential denial-of-service (DOS) vulnerabilities due to inconsistencies in nomination pools to issues with the `delegate_compound` function and improper unbond amount handling. Low-severity issues were mostly around missing event emissions, storage limitations, and precise time calculations, among others. The competition, which lasted two weeks, had a maximum reward of $52,000 and concluded with the total payout distributed among 16 participants. The proactive identification and resolution of these issues underscore Hats.finance's commitment to decentralized security solutions, ensuring robust and efficient protection for web3 projects.

## Disclaimer


This report does not assert that the audited contracts are completely secure. Continuous review and comprehensive testing are advised before deploying critical smart contracts.


The Kintsu audit competition illustrates the collaborative effort in identifying and rectifying potential vulnerabilities, enhancing the overall security and functionality of the platform.


Hats.finance does not provide any guarantee or warranty regarding the security of this project. Smart contract software should be used at the sole risk and responsibility of users.

