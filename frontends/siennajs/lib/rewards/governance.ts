import { Address, Decimal, Uint128 } from "../core";

export enum PollType {
    SiennaRewards = "sienna_rewards",
    SiennaSwapParameters = "sienna_swap_parameters",
    Other = "other"
}
export enum PollStatus {
    Active = "active",
    Passed = "passed",
    Failed = "failed"
}
export enum VoteType {
    Yes = "yes",
    No = "no"
}


export type Expiration = {
    at_time: number
}

export interface GovernanceConfig {
    threshold?: Uint128
    deadline?: number
    quorum?: Decimal
}
export interface PollMetadata {
    title: String,
    description: String,
    poll_type: PollType
}
export interface Poll {
    id: number,
    creator: Address
    metadata: PollMetadata,
    expiration: Expiration,
    status: PollStatus,
    current_quorum: Decimal
}

export interface PollResult {
    poll_id: number,
    yes_votes: Uint128,
    no_votes: Uint128,
}
export interface PollInfo {
    instance: Poll,
    result: PollResult
}

export interface VoteStatus {
    power: Uint128,
    choice: VoteType
}

export interface GetPollResponse {
    poll: PollInfo
}
export interface GetPollsResponse {
    polls: Array<Poll>,
    total: number,
    total_pages: number
}

export type PollsCollection = GetPollsResponse;
export interface GetVoteStatusResponse {
    vote_status: {
        power: Uint128,
        choice: VoteType
    }
}
export interface GetGovernanceConfigResponse {
    config: GovernanceConfig
}

export enum SortingDirection {
    Ascending = 1,
    Descending = 0
}