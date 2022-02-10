# Sienna Governance

## Concepts

* User stakes LP tokens into Reward contract and that represents his/her voting power.
* User always uses all staked tokens as voting power.
* User can vote on multiple polls using the same staked tokens.
* Votes are private
* Results are private (?)
* Poll creator needs to have minimum staked tokens, as defined in config (threshold)
* Quorum needs to be met, as defined in config and 33% by default, in order to have a valid poll
* Quorum needs to be preserved only during the voting period. 
* User can change or cancel the vote
* User can change ammount of staked tokens which will cause all his votes to be recalculated
* Changing the staked ammount after the poll has expired will have no consequence on quorum or result
* 

TDB:
* can creator unstake below the threshold while his poll is active
* is user permitted to unstake certain ammount after he placed votes (multiple polls)
* if no committee: results can't be private during the voting period; must send time on query (security issue)
* executing msg (proposed) - handle called by non-admin user. Preferably scheduler
* Low participation / users apathy
* 

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
    - UserPolls -> update all active polls for user

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