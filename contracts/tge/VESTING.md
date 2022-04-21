# Vesting

* [Contents](#contents) of this directory
* [MGMT](#mgmt) 
* [RPT](#rpt)
* [Compile for production](#compile-for-production)
* [Run tests](#run-tests)
* [Configuration](#configuration)
* [Use](#use)

* [Emergency mode](#Emergency-mode)
  * Pausing contract
  * Resuming normal operation
  * Deploying an updated version of the contract
  * Pausing all transactions with the token
* [Launch vesting](#Launch-Vesting)
  * [Adding a new account to vesting](#Adding-a-new-account-to-vesting)

## Contents

* `snip20-sienna` - Core SIENNA token for Secret Network
* `wrapped` - SIENNA token wrapped as ERC20 (Ethereum)
* `mgmt` - Main vesting management contract
* `rpt` - Remaining pool token distribution contract

## MGMT
The main contract managing and vesting the tokens. It's main configuration is as follows: 
```rust
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub schedule: Schedule<HumanAddr>,
    pub token: ContractLink<HumanAddr>,
    pub prefund: bool,
}
```
* admin
    * if the admin is not set, the sender will be made the admin
* schedule
    * the main configuration of the contract
* token
    * which token will be vested
* prefund
    * if enabled, upon launch the contract will check if it has enough balance of the token
    * if disabled, the contract will expect to be a minter of the token and it will mint tokens according to the schedule. Then it will remove all minters so that nobody can mint anymore tokens. 


#### Schedule
This is the configuration dictating when,where and how many of the tokens will be sent. 
It's format is as follows: 
```jsonc
{
    "schedule":{
        "total":"10000000000000000000000000",
        "pools":[
        {
            "partial":true,
            "name":"Investors",
            "total":"2000000000000000000000000",
            "accounts":[
                {
                    "name":"978",
                    "address":"secret1...",
                    "amount":"155000000000000000",
                    "cliff":"0",
                    "start_at":7776000,
                    "interval":86400,
                    "duration":41472000
                }
                // ...
            ]
        }
        // ...
        ]
    }
}
```
* schedule.total
    * how many tokens across all pools are vested
    * the total from each pool *must* add up to the total in the schedule, else the
    configuration is invalid
* pools
    * category which holds a list of accounts that belong to it. 
    * the amount from each account in the pool summed up must add up to the total in its pool
    * if partial is enabled, the validation that the accounts add up to the total in the pool will
    be disabled. This allows adding more accounts as long as it's not more than the allowed total in the pool
* account
    * amount - how many tokens this account recieves in total
    * cliff - if `> 0`, releases this much money the first time, pushing back the regular portions
    * start_at - how many seconds after launch to begin vesting
    * interval - how many seconds to wait between portions
    * duration - if `> 0`, vesting stops after this many seconds regardless of how much is left of `total`

##### Claiming
After launching, when the vesting has begun, any account in the schedule can call the `Claim` method to recieve the gains they have accumulated so far. This method can be call in the regular intervals or at any point in time, even after the vesting period has finished. 

This method can be called as follows via `secretcli`:
```bash
secretcli tx compute execute <address> '{"claim": {}}' --from <your_key> -y 
```
##### Progress
To check how much progress any account has made for a given moment in time, you can call the `Progress` query.

This can be done via `secretcli` as follows:
```bash
secretcli q compute query <address> '{"progress": { "address": <account_addr>, "time": <unix_timestamp> }}'
```
##### Increasing Allocation
Since the schedule cannot be configured again after launching, to avoid spinning up a new instance there is a method to increase the total amount of tokens vested and add a pool which will use the newly added tokens immediately.
* The total can only be incremented
* Accounts cannot be changed
* Pools cannot be removed or changed
* The contract **must** have enough balance to increase the total, else the schedule will not be updated

Steps: 
1. Send desired amount of tokens to `MGMT`
2. Send `IncreaseAllocation` message
3. Update RPT distribution accordingly

After sending over the required tokens, call the `IncreaseAllocation` method as follows: 
```bash
secretcli tx compute execute <address> '{"increase_allocation": { "total_increment": "<amount>", "pool": <new_pool> }}`
```
> The pool can be made partial with no accounts since its possible to add accounts to a pool later. You can also add 

Pitfalls:
* To avoid any potential issues, the allocation should be updated before vest is called
* Distribution in `RPT` also has to be updated right away

In the scenario that the `RPT` is not updated at all or in time, it will have leftover funds which will remain on the contract. The only possible way to take out these tokens then would be to update the distribution accordingly so that the tokens that werent sent, would be sent. This is however prone to error and should be avoided (if the admin makes a mistake in the calculation). 

## RPT
Handles the main distribution of the tokens to the relevant pools or users. 

It's main configuration is as follows:
```rust
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub distribution: Distribution<HumanAddr>,
    pub portion: Portion,
    pub token: ContractLink<HumanAddr>,
    pub mgmt: ContractLink<HumanAddr>,
}
```
* portion
    * how many tokens the contract recieves when `Vest` is called
* distribution
    * a list of addresses along with how many tokens they recieve.
    * the portions in the distribution must add up to the main portion configured.



## Compile for production

```sh
pnpm build vesting
```

## Run tests

```sh
cargo test -p tge-tests
```
### Deploy the TGE contracts
Before getting started, make sure you have prepared the vesting schedule. You can either place this in `settings/vesting.json`, or prepare a custom file and set it via env as follows:
```bash
export VESTING_CONFIG= 'path'
```

<table>
<tr><td>

* Make sure you're running in a secure environment
  with a reliable Internet connection.

* Make sure you have access to the following tools in your
  environment:
  - Bash
  - Git
  - jq
  - Docker
  - Node.JS + pnpm
  - secretcli

</td><td>

```bash
# 1. clone the repo
git clone --recursive https://github.com/SiennaNetwork/SiennaNetwork

# 2. Install dependencies
pnpm i
git submodule update --init --recursive

# 3. Configure environment. Either using a .env file or exporting
export SCRT_AGENT_ADDRESS='your address'
export SCRT_AGENT_MNEMONIC='your mnemonic'
export SCRT_AGENT_NAME='your name'
export FADROMA_CHAIN='' # Scrt_1_2_(Devnet/Testnet/Mainnet)
# 5. Run sienna deploy

pnpm deploy vesting

# 5. Remove your mnemonic from the environment immediately afterwards!
export SCRT_AGENT_MNEMONIC=
```

> ℹ️  The agent configuration is only needed for `Mainnet` and `Testnet` deploy.

</td></tr>
<tr><!--spacer--></tr>
</td></tr>
</table>


# Configuration


<table>

Vesting configuratin: 

```jsonc
[
    {
        "name": "",
        "rewards": {
            "name": "",
            "timekeeper": "" // optional, will be ignored if undefined or empty. Do not pass invalid address here!
            "decimals": 18, // based on how the token is configured
            "address": "",
            "code_hash": ""
        },
        "lp": {
            "name": "",
            "address": "",
            "code_hash": ""
        },
        "schedule": {
            "total": "",
            "pools": [
                {
                    "name": "",
                    "total": "",
                    "partial": true,
                    "accounts": []
                }
            ]
        },
        "account": {
            "name": "",
            "amount": "",
            "address": "LEAVE_BLANK", // do not put anything here, it will be set automatically
            "start_at": 0,
            "interval": 0,
            "duration": 0,
            "cliff": "0",
            "portion_size": ""       
        }
    },
]
```
The deploy procedure needs the links to the lp and rewards tokens that are to be used. 
The Reward token will be used in `MGMT` and `RPT`, while the LP token will only be used for instantiating the Reward Pool. 

The schedule should have at least one pool, wheter its partial is up to you.
The account property has to be configured correctly as it will be added after both the `MGMT` and `RPT` contracts are deployed. The address of the account can be left blank as the procedure will automatically fill in the address of the `RPT` contract. 


>  On Devnet, mock tokens will be instantiated instead of using the configured ones. 
>  However, the same schedule will still be used.

</td></tr>
<tr><!--separator--></tr>

<tr><td valign="top">

### Add a new account to vesting

Send this message in a transaction from the admin address to the MGMT contract to unlock 1 SIENNA immediately for address `secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a`:

```json
{"add_account":{"pool":"Investors","account":{"name":"someone","amount":"1000000000000000000","address":"secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a","start_at":0,"interval":0,"duration":0,"cliff":"1000000000000000000"}}
```g

</td><td>

Example:
```bash
# Sample transaction to add 1 SIENNA with immediate vesting to sample account
secretcli tx compute execute MGMT_CONTRACT_ADDRESS '{"add_account":{"pool_name":"Investors","account":{"name":"someone","amount":"1000000000000000000","address":"secret1ngfu3dkawmswrpct4r6wvx223f5pxfsryffc7a","start_at":0,"interval":0,"duration":0,"cliff":"1000000000000000000"}}}' --from ADMIN_KEY_ALIAS --chain-id NETWORK_ID --gas 450000
```

>ℹ️ `start_at`, `interval`, `duration` are in seconds (1 day = 86400 seconds). For immediate vesting, set them all to 0 and `cliff` = `amount`.

>ℹ️ `amount` and `cliff` are in attoSIENNA (multiply by `1000000000000000000` - 1 with 18 zeros - to get SIENNA), and must be in double quotes (`"`) - because JSON doesn't support numbers that big.

>⚠️ Be careful - errors here are permanent and can't be remedied without a full migration (untested procedure!)

</td></tr>

<tr><!--spacer--></tr>

<tr><td valign="top">

### Launch the vesting

Launch the vesting with this message.

Transaction message:

```bash
{"launch":{}}
```

</td><td>

Example:
```bash
# Start vesting
secretcli tx compute execute MGMT_CONTRACT_ADDRESS '{"launch":{}}' --from ADMIN_KEY_ALIAS --chain-id NETWORK_ID --gas 450000
```

> ℹ️ NETWORK_ID is holodeck-2 for testnet & secret-2 for mainnet
```
# Confirm the transaction succeeded
secretcli q compute tx TRANSACTION_HASH | jq '.'
```

</td></tr>
</table>

## Use

* To claim funds from MGMT, send it `{"claim":{}}`.
* To make RPT send funds to the reward pools, send it `{"vest":{}}`

## Killswitch

In case of unexpected events,
the admin of the contracts
(normally, the multisig wallet)
can send one of the following transactions
to allow for manual recovery.

### Pausing contract

Transaction message:

```json
{"killswitch": { "set_status": { "level": "Paused", "reason": "This contract is paused because someone did a silly thing", "new_address": "<Optional address>" }}}
```

Sending this message to MGMT or RPT would pause that contract.
Any transactions will return an error message containing the specified reason.

>ℹ️ Note that you can do this separately for each of the two.
>If you pause MGMT, RPT will be unable to claim any additional funds from it.
>If you pause RPT, MGMT continues operating normally.

### Resuming normal operation

Transaction message:

```json
{"killswitch": { "set_status":{ "level":"Operational","reason": "" }}}
```

This returns a paused contract to its `Operational` state,
allowing transactions to proceed normally.

>ℹ️ When MGMT or RPT is paused, time continues to pass for the vesting.
>When it is resumed, users will be able claim the funds accumulated in the meantime.

### Deploying an updated version of the contract

Transaction message:

```json
{"killswitch": { "set_status":{"level":"Migrating","reason":"He's dead, Jim"}}}
```

This pauses the contract **permanently**. Once you've deployed an updated version,
you can send another `set_status` to provide the new contract address
that will be presented to users in the error message.

Transaction message:

```json
{"killswitch": { "set_status":{"level":"Migrating","reason":"Fixed!","new_address":"secret1....."}}}
```

>ℹ️ Actual migrations are manual.
>Other than permanently stopping the contract and telling users the new address,
>the `Migrating` state performs no other operations. Future versions may implement
>automated migration logic as needed.

