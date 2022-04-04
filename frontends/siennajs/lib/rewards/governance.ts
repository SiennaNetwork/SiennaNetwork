import { Address, Decimal, Uint128, ViewingKey } from "../core";
import { Permit } from "../permit";

/**
 * Supports any number of additions, saved as a string in the contract. 
 * Limits:
 *     min length: 5
 *     max length: 20
 */
export enum PollType {
    SiennaRewards = "sienna_rewards",
    SiennaSwapParameters = "sienna_swap_parameters",
    Other = "other"
}

export enum PollStatus {
    /**
     * The poll is not expired, voting is still possible
     */
    Active = "active",
    /**
     * The poll has expired, quorum has passed and the poll has passed
     */
    Passed = "passed",
    /**
     * Quorum has not been reached or poll has failed.
     */
    Failed = "failed"
}

/**
 * Possible vote options
 */
export enum VoteType {
    Yes = "yes",
    No = "no"
}


/**
 * Helper around poll expiration. Currently holds only @at_time
 */
export type Expiration = {
    at_time: number
}

export interface GovernanceConfig {
    /**
     * Minimum amount of staked tokens needed to create a poll
     */
    threshold: Uint128
    /**
     * The amount of time a poll lasts in seconds
     */
    deadline: number
    /**
     * Minimum percentage (0-1) which is needed for a poll to be valid
     */
    quorum: Decimal
}
export interface PollMetadata {
    /**
     * The title of the poll. 
     * Has a default min and max
     */
    title: String,
    /**
     * The description of the poll.
     * Has a default min and max
     */
    description: String,
    /**
     * Generic type of the poll, underlying type can be any string.
     */
    poll_type: PollType
}
export interface Poll {
    id: number,
    /**
     * Saved as the user who send the create poll transaction
     */
    creator: Address
    metadata: PollMetadata,
    expiration: Expiration,
    status: PollStatus,
    /**
     * Snapshot of the quorum taken from the configuration at the time of creation.
     * Used in calculating results until poll has expired
     */
    current_quorum: Decimal
}

export interface PollResult {
    poll_id: number,
    /** 
     * The total number of yes votes, equals the number of tokens staked.
     * As vote = stake power 
     */
    yes_votes: Uint128,
    no_votes: Uint128,
}
/**
 * Generic helper struct to wrap all poll information
 * @instance - The entire poll itself
 * @result - The up to date results of the poll. 
 */
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


