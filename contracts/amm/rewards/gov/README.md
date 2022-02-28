# Sienna Governance
  
## Conceptual overview
Sienna Governance serves the purpose of applying changes to the network based on the opinions of stake holders.  
Stake holders can create text based polls and other holders can vote on them.  
      
* The voting power of a user is dependent on the total amount of staked tokens in the rewards pool
* There are no consequences of polls failing
* User can vote in any number of polls  
* User can stake more tokens to update his voting power
* All the votes are private
  
As the polls are purely text based, any changes are left to the admin to change. There are currently no automatically executed messages after the poll is complete.  
   
Updating the polls is event based and has the following limitations: 
- User cannot unstake while their vote is active in an on-going poll
- A user(stake holder) can only create a poll if their staked balance is higher than the configured threshold
- If a user has not voted in any on going polls, but a poll they created is active, they can only unstake up to the threshold. Meaning his balance must never go below the threshold while the created poll is active

Updating of all active polls occurs when a stake holder stakes or unstakes (*according to the limitations*) tokens. All non expired polls will then be updated with the new voting power and the result will be recalculated. 
## Configuration
* threshold - the minimum amount of tokens needed in order to create a poll (defaults to 3500)
* quorum - minimum percentage of voting power that needs to be casted on a proposal for the result to be valid. (value between 0 and 1, defaults to 0.3)
* deadline - the amount of time, in seconds, a poll lasts. Expiration is then set as current_time + deadline (defaults to 7 days)

## Control flows
### User flow
### High level flow overview
![high level overview](../doc/gov_high_flow.png)  

When a user votes in a poll his balance is checked and his voting power is registered for the given poll
This causes the result to be (re)calculated according to this formula:
```
tally = yes votes + no votes
participation = tally / total staked in pool

result = participation > quorum
```