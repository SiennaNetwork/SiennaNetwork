import {
  Address,
  Uint128,
  Uint256,
  Fee,
  ContractInfo,
  ViewingKey,
} from "../core";
import { SmartContract, Querier } from "../contract";
import { ViewingKeyExecutor } from "../executors/viewing_key_executor";

import { ExecuteResult } from "secretjs";
import {
  GetGovernanceConfigResponse,
  GetPollResponse,
  GetPollsResponse,
  GetVoteStatusResponse,
  GovernanceConfig,
  Poll,
  PollInfo,
  PollMetadata,
  PollsCollection,
  SortingDirection,
  VoteStatus,
  VoteType,
} from "./governance";

export type Moment = number;
export type Duration = number;

/**
 * Reward pool configuration
 */
export interface RewardsConfig {
  lp_token?: ContractInfo;
  reward_token?: ContractInfo;
  reward_vk?: string;
  bonding?: number;
  timekeeper?: Address;
}

export interface RewardsClock {
  /** "For what point in time do the reported values hold true?" */
  now: Moment;
  /** "What is the current reward epoch?" */
  number: number;
  /** "When did the epoch last increment?" */
  started: Moment;
  /** "What was the total pool liquidity at the epoch start?" */
  volume: Uint256;
}

export interface RewardsTotal {
  /** What is the current time and epoch? */
  clock: RewardsClock;
  /** When was the last time someone staked or unstaked tokens?" */
  updated: Moment;
  /** What liquidity is there in the whole pool right now? */
  staked: Uint128;
  /** What liquidity has this pool contained up to this point? */
  volume: Uint256;
  /** What amount of rewards is currently available for users? */
  budget: Uint128;
  /** What rewards has everyone received so far? */
  distributed: Uint128;
  /** What rewards were unlocked for this pool so far? */
  unlocked: Uint128;
  /** How long must the user wait between claims? */
  bonding: Duration;
  /** Is this pool closed, and if so, when and why? */
  closed?: [Moment, string];
}

/** Account status */
export interface RewardsAccount {
  /** What is the overall state of the pool? */
  total: RewardsTotal;
  /** "When did this user's liquidity amount last change?" Set to current time on update. */
  updated: Moment;
  /** How much time has passed since the user updated their stake? */
  elapsed: Duration;
  /** How much liquidity does this user currently provide? */
  staked: Uint128;
  /** What portion of the pool is currently owned by this user? */
  pool_share: [Uint128, Uint128];
  /** How much liquidity has this user provided since they first appeared? */
  volume: Uint256;
  /** What was the volume of the pool when the user entered? */
  starting_pool_volume: Uint256;
  /** How much liquidity has accumulated in the pool since this user entered? */
  accumulated_pool_volume: Uint256;
  /** What portion of all liquidity accumulated since this user's entry is from this user?  */
  reward_share: [Uint256, Uint256];
  /** How much rewards were already unlocked when the user entered? */
  starting_pool_rewards: Uint128;
  /** How much rewards have been unlocked since this user entered? */
  accumulated_pool_rewards: Uint128;
  /** How much rewards has this user earned? */
  earned: Uint128;
  /** How many units of time (seconds) remain until the user can claim? */
  bonding: Duration;
}

export class RewardsV3Contract extends SmartContract<
  RewardsV3Executor,
  RewardsV3Querier
> {
  exec(fee?: Fee, memo?: string): RewardsV3Executor {
    return new RewardsV3Executor(this.address, this.execute_client, fee, memo);
  }

  query(): RewardsV3Querier {
    return new RewardsV3Querier(this.address, this.query_client);
  }
}

class RewardsV3Executor extends ViewingKeyExecutor {
  async claim(): Promise<ExecuteResult> {
    const msg = { rewards: { claim: {} } };
    return this.run(msg, "80000");
  }

  async deposit_tokens(amount: Uint128): Promise<ExecuteResult> {
    const msg = { rewards: { deposit: { amount } } };

    return this.run(msg, "75000");
  }

  async withdraw_tokens(amount: Uint128): Promise<ExecuteResult> {
    const msg = { rewards: { withdraw: { amount } } };

    return this.run(msg, "75000");
  }

  async create_poll(meta: PollMetadata) {
    const msg = { governance: { create_poll: { meta } } };

    return this.run(msg, "80000");
  }
  async vote(choice: VoteType, poll_id: number) {
    const msg = { governance: { vote: { choice, poll_id } } };

    return this.run(msg, "75000");
  }
  async unvote(poll_id: number) {
    const msg = { governance: { poll_id } };

    return this.run(msg, "75000");
  }
  async change_vote_choice(choice: VoteType, poll_id: number) {
    const msg = { governance: { change_vote_choice: { choice, poll_id } } };

    return this.run(msg, "75000");
  }
}

class RewardsV3Querier extends Querier {
  async get_pool(at: number): Promise<RewardsTotal> {
    const msg = { rewards: { pool_info: { at } } };

    const result = (await this.run(msg)) as GetPoolResponse;
    return result.rewards.pool_info;
  }

  async get_account(
    address: Address,
    key: ViewingKey,
    at: number
  ): Promise<RewardsAccount> {
    const msg = { rewards: { user_info: { address, key, at } } };

    const result = (await this.run(msg)) as GetAccountResponse;
    return result.rewards.user_info;
  }

  async get_poll(poll_id: number, now: number): Promise<PollInfo> {
    const msg = { governance: { poll: { poll_id, now } } };
    const result = (await this.run(msg)) as GetPollResponse;
    return result.poll;
  }

  async get_polls(
    now: number,
    page: number,
    take: number,
    sort: SortingDirection
  ): Promise<PollsCollection> {
    const msg = { governance: { polls: { now, page, take, asc: !!sort } } };
    return this.run(msg);
  }
  async get_vote_status(
    address: Address,
    key: string,
    poll_id: number
  ): Promise<VoteStatus> {
    const msg = { governance: { vote_status: { address, key, poll_id } } };

    const result = (await this.run(msg)) as GetVoteStatusResponse;
    return result.vote_status;
  }
  async get_governance_config(): Promise<GovernanceConfig> {
    const msg = { governance: { config: {} } };
    const result = (await this.run(msg)) as GetGovernanceConfigResponse;

    return result.config;
  }
}

interface GetAccountResponse {
  rewards: { user_info: RewardsAccount };
}

interface GetPoolResponse {
  rewards: { pool_info: RewardsTotal };
}
