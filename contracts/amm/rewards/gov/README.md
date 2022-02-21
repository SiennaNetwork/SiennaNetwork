# Sienna Governance

## Concepts

* User stakes LP tokens into Reward contract and that represents his/her voting power.
* User always uses all staked tokens as voting power.
* User can vote on multiple polls using the same staked tokens.
* Votes are private
* Vote status can be queried using a viewing key
* Results are public
* Poll creator needs to have minimum staked tokens, as defined in config (threshold)
* Quorum needs to be met, as defined in config and 33% by default, in order to have a valid poll
* Poll will use quorum requirements at the time of its creation
* User can change or cancel the vote
* User can increase (but not decrease) ammount of staked tokens which will cause all his votes to be recalculated
* User can't unstake if participated in active polls
* Changing the staked ammount after the poll has expired will have no consequence on quorum or result
* Creator can't unstake below the threshold while his poll is active


TDB:
* executing msg (proposed) - handle called by non-admin user. Preferably scheduler

## Implementation details

* Specific poll should take a snapshot value of quorum at the time being made, to shield against the global config changes while the poll is active

* After each user's stake change, we need to check:
    - if creator
        - threshold 
    - else 
        - is quorum met
        - recalculate all polls where user voted
        - if poll has expired, do not change voting power

* if no committee, all state changes must be done on Stake, UnStake, ChangeVote, Vote, UnVote. Query should check if poll has expired by sending time

* Handles:
    - Vote
    - UnVote


* Changing voting power recalculates:
    - Vote 
    - PollResult 
    - User -> update all active polls for user

### Storage

'#' is a placeholder for poll_id, which is usually added after the given namespace (/gov/poll/treshold # => /gov/poll/treshold/21)

- gov   - treshold (#)
        - quorum (#)
        - deadline (#)
        - votes (#)
        - committee (obsolete) (#)

        - polls  
                - total (-> count)
        - poll 
                - creator (#)
                - expiration (#)
                - status (#)
                - current_quorum (#)
                - reveal_approvals (obsolete) (#)
                - meta
                        - title (#)
                        - desc (#)
                        - type (#)
        - result (#)
        - vote (# + usr_addr)
        - user_polls (usr_addr) [poll_ids]