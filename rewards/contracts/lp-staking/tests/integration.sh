#!/bin/bash

set -eu
set -o pipefail # If anything in a pipeline fails, the pipe's exit status is a failure
#set -x # Print all commands for debugging

declare -a KEY=(a b c d)

declare -A FROM=(
    [a]='-y --from a'
    [b]='-y --from b'
    [c]='-y --from c'
    [d]='-y --from d'
)

# This means we don't need to configure the cli since it uses the preconfigured cli in the docker.
# We define this as a function rather than as an alias because it has more flexible expansion behavior.
# In particular, it's not possible to dynamically expand aliases, but `tx_of` dynamically executes whatever
# we specify in its arguments.
function secretcli() {
    docker exec secretdev /usr/bin/secretcli "$@"
}

# Just like `echo`, but prints to stderr
function log() {
    echo "$@" >&2
}

# suppress all output to stdout and stderr for the command described in the arguments
function silent() {
    "$@" >/dev/null 2>&1
}

# Pad the string in the first argument to 256 bytes, using spaces
function pad_space() {
    printf '%-256s' "$1"
}

function assert_eq() {
    local left="$1"
    local right="$2"
    local message

    if [[ "$left" != "$right" ]]; then
        if [ -z ${3+x} ]; then
            local lineno="${BASH_LINENO[0]}"
            message="assertion failed on line $lineno - both sides differ. left: ${left@Q}, right: ${right@Q}"
        else
            message="$3"
        fi
        log "$message"
        return 1
    fi

    return 0
}

function assert_ne() {
    local left="$1"
    local right="$2"
    local message

    if [[ "$left" == "$right" ]]; then
        if [ -z ${3+x} ]; then
            local lineno="${BASH_LINENO[0]}"
            message="assertion failed on line $lineno - both sides are equal. left: ${left@Q}, right: ${right@Q}"
        else
            message="$3"
        fi

        log "$message"
        return 1
    fi

    return 0
}

declare -A ADDRESS=(
    [a]="$(secretcli keys show --address a)"
    [b]="$(secretcli keys show --address b)"
    [c]="$(secretcli keys show --address c)"
    [d]="$(secretcli keys show --address d)"
)

declare -A VK=([a]='' [b]='' [c]='' [d]='')

# Generate a label for a contract with a given code id
# This just adds "contract_" before the code id.
function label_by_init_msg() {
    local init_msg="$1"
    local code_id="$2"
    sha256sum <<< "$code_id $init_msg"
}

# Keep polling the blockchain until the tx completes.
# The first argument is the tx hash.
# The second argument is a message that will be logged after every failed attempt.
# The tx information will be returned.
function wait_for_tx() {
    local tx_hash="$1"
    local message="$2"

    local result

    log "waiting on tx: $tx_hash"
    # secretcli will only print to stdout when it succeeds
    until result="$(secretcli query tx "$tx_hash" 2>/dev/null)"; do
        log "$message"
        sleep 1
    done

    # log out-of-gas events
    if jq -e '.raw_log | startswith("execute contract failed: Out of gas: ") or startswith("out of gas:")' <<<"$result" >/dev/null; then
        log "$(jq -r '.raw_log' <<<"$result")"
    fi

    echo "$result"
}

# This is a wrapper around `wait_for_tx` that also decrypts the response,
# and returns a nonzero status code if the tx failed
function wait_for_compute_tx() {
    local tx_hash="$1"
    local message="$2"
    local return_value=0
    local result
    local decrypted

    result="$(wait_for_tx "$tx_hash" "$message")"
    # log "$result"
    if jq -e '.logs == null' <<<"$result" >/dev/null; then
        return_value=1
    fi
    decrypted="$(secretcli query compute tx "$tx_hash")" || return
    log "$decrypted"
    echo "$decrypted"

    return "$return_value"
}

# If the tx failed, return a nonzero status code.
# The decrypted error or message will be echoed
function check_tx() {
    local tx_hash="$1"
    local result
    local return_value=0

    result="$(secretcli query tx "$tx_hash")"
    if jq -e '.logs == null' <<<"$result" >/dev/null; then
        return_value=1
    fi
    decrypted="$(secretcli query compute tx "$tx_hash")" || return
    log "$decrypted"
    echo "$decrypted"

    return "$return_value"
}

# Extract the tx_hash from the output of the command
function tx_of() {
    "$@" | jq -r '.txhash'
}

# Extract the output_data_as_string from the output of the command
function data_of() {
    "$@" | jq -r '.output_data_as_string'
}

function get_generic_err() {
    jq -r '.output_error.generic_err.msg' <<<"$1"
}

# Send a compute transaction and return the tx hash.
# All arguments to this function are passed directly to `secretcli tx compute execute`.
function compute_execute() {
    tx_of secretcli tx compute execute "$@"
}

# Send a query to the contract.
# All arguments to this function are passed directly to `secretcli query compute query`.
function compute_query() {
    secretcli query compute query "$@"
}

function upload_code() {
    local directory="$1"
    local tx_hash
    local code_id

    tx_hash="$(tx_of secretcli tx compute store "code/$directory/contract.wasm.gz" ${FROM[a]} --gas 10000000)"
    code_id="$(
        wait_for_tx "$tx_hash" 'waiting for contract upload' |
            jq -r '.logs[0].events[0].attributes[] | select(.key == "code_id") | .value'
    )"

    log "uploaded contract #$code_id"

    echo "$code_id"
}

function instantiate() {
    local code_id="$1"
    local init_msg="$2"

    log 'sending init message:'
    log "${init_msg@Q}"

    local tx_hash
    tx_hash="$(tx_of secretcli tx compute instantiate "$code_id" "$init_msg" --label "$(label_by_init_msg "$init_msg" "$code_id")" ${FROM[a]} --gas 10000000)"
    wait_for_tx "$tx_hash" 'waiting for init to complete'
}

# This function uploads and instantiates a contract, and returns the new contract's address
function create_contract() {
    local dir="$1"
    local init_msg="$2"

    local code_id
    code_id="$(upload_code "$dir")"

    local init_result
    init_result="$(instantiate "$code_id" "$init_msg")"

    if jq -e '.logs == null' <<<"$init_result" >/dev/null; then
        log "$(secretcli query compute tx "$(jq -r '.txhash' <<<"$init_result")")"
        return 1
    fi

    jq -r '.logs[0].events[0].attributes[] | select(.key == "contract_address") | .value' <<<"$init_result"
}

# This function uploads and instantiates a contract, and returns the new contract's address
function init_contract() {
    local code_id="$1"
    local init_msg="$2"

    local init_result
    init_result="$(instantiate "$code_id" "$init_msg")"

    if jq -e '.logs == null' <<<"$init_result" >/dev/null; then
        log "$(secretcli query compute tx "$(jq -r '.txhash' <<<"$init_result")")"
        return 1
    fi

    jq -r '.logs[0].events[0].attributes[] | select(.key == "contract_address") | .value' <<<"$init_result"
}

function deposit() {
    local contract_addr="$1"
    local key="$2"
    local amount="$3"

    local deposit_message='{"deposit":{"padding":":::::::::::::::::"}}'
    local tx_hash
    local deposit_response
    tx_hash="$(compute_execute "$contract_addr" "$deposit_message" --amount "${amount}uscrt" ${FROM[$key]} --gas 150000)"
    deposit_response="$(data_of wait_for_compute_tx "$tx_hash" "waiting for deposit to \"$key\" to process")"
    assert_eq "$deposit_response" "$(pad_space '{"deposit":{"status":"success"}}')"
    log "deposited ${amount}uscrt to \"$key\" successfully"
}

function get_balance() {
    local contract_addr="$1"
    local key="$2"

    log "querying balance for \"$key\""
    local balance_query='{"balance":{"address":"'"${ADDRESS[$key]}"'","key":"'"${VK[$key]}"'"}}'
    local balance_response
    balance_response="$(compute_query "$contract_addr" "$balance_query")"
    log "balance response was: $balance_response"
    jq -r '.balance.amount' <<<"$balance_response"
}

function get_token_info() {
    local contract_addr="$1"

    local token_info_query='{"token_info":{}}'
    compute_query "$contract_addr" "$token_info_query"
}

function increase_allowance() {
    local contract_addr="$1"
    local owner_key="$2"
    local spender_key="$3"
    local amount="$4"

    local owner_address="${ADDRESS[$owner_key]}"
    local spender_address="${ADDRESS[$spender_key]}"
    local allowance_message='{"increase_allowance":{"spender":"'"$spender_address"'","amount":"'"$amount"'"}}'
    local allowance_response

    tx_hash="$(compute_execute "$contract_addr" "$allowance_message" ${FROM[$owner_key]} --gas 150000)"
    allowance_response="$(data_of wait_for_compute_tx "$tx_hash" "waiting for the increase of \"$spender_key\"'s allowance to \"$owner_key\"'s funds to process")"
    assert_eq "$(jq -r '.increase_allowance.spender' <<<"$allowance_response")" "$spender_address"
    assert_eq "$(jq -r '.increase_allowance.owner' <<<"$allowance_response")" "$owner_address"
    jq -r '.increase_allowance.allowance' <<<"$allowance_response"
    log "Increased allowance given to \"$spender_key\" from \"$owner_key\" by ${amount}uscrt successfully"
}

function decrease_allowance() {
    local contract_addr="$1"
    local owner_key="$2"
    local spender_key="$3"
    local amount="$4"

    local owner_address="${ADDRESS[$owner_key]}"
    local spender_address="${ADDRESS[$spender_key]}"
    local allowance_message='{"decrease_allowance":{"spender":"'"$spender_address"'","amount":"'"$amount"'"}}'
    local allowance_response

    tx_hash="$(compute_execute "$contract_addr" "$allowance_message" ${FROM[$owner_key]} --gas 150000)"
    allowance_response="$(data_of wait_for_compute_tx "$tx_hash" "waiting for the decrease of \"$spender_key\"'s allowance to \"$owner_key\"'s funds to process")"
    assert_eq "$(jq -r '.decrease_allowance.spender' <<<"$allowance_response")" "$spender_address"
    assert_eq "$(jq -r '.decrease_allowance.owner' <<<"$allowance_response")" "$owner_address"
    jq -r '.decrease_allowance.allowance' <<<"$allowance_response"
    log "Decreased allowance given to \"$spender_key\" from \"$owner_key\" by ${amount}uscrt successfully"
}

function get_allowance() {
    local contract_addr="$1"
    local owner_key="$2"
    local spender_key="$3"

    log "querying allowance given to \"$spender_key\" by \"$owner_key\""
    local owner_address="${ADDRESS[$owner_key]}"
    local spender_address="${ADDRESS[$spender_key]}"
    local allowance_query='{"allowance":{"spender":"'"$spender_address"'","owner":"'"$owner_address"'","key":"'"${VK[$owner_key]}"'"}}'
    local allowance_response
    allowance_response="$(compute_query "$contract_addr" "$allowance_query")"
    log "allowance response was: $allowance_response"
    assert_eq "$(jq -r '.allowance.spender' <<<"$allowance_response")" "$spender_address"
    assert_eq "$(jq -r '.allowance.owner' <<<"$allowance_response")" "$owner_address"
    jq -r '.allowance.allowance' <<<"$allowance_response"
}

function set_viewing_keys() {
    local contract_addr="$1"

    for key in "${KEY[@]}"; do
        log 'setting the viewing key for "'$key'"'
        local set_viewing_key_message='{"set_viewing_key":{"key":"'"${VK[$key]}"'"}}'
        tx_hash="$(compute_execute "$contract_addr" "$set_viewing_key_message" ${FROM[$key]} --gas 1400000)"
        viewing_key_response="$(data_of wait_for_compute_tx "$tx_hash" 'waiting for viewing key for "'$key'" to be set')"
        assert_eq "$viewing_key_response" "$(pad_space '{"set_viewing_key":{"status":"success"}}')"
    done
}

function query_height() {
    local status=$(secretcli status)
    jq -r '.sync_info.latest_block_height' <<<"$status"
}

function log_test_header() {
    log " # Starting ${FUNCNAME[1]}"
}

##### lockup help functions

# Redeem locked tokens from an account
# As you can see, verifying this is happening correctly requires a lot of code
# so I separated it to its own function, because it's used several times.
function redeem() {
    local contract_addr="$1"
    local key="$2"
    local amount="$3"
    local token_addr="$4"

    local redeem_message
    local tx_hash
    local redeem_tx
    local transfer_attributes
    local redeem_response
    local redeem_error

    log "redeeming \"$key\""
    redeem_message='{"redeem":{"amount":"'"$amount"'"}}'
    old_balance=$(get_balance "$token_addr" "$key")

    tx_hash="$(compute_execute "$contract_addr" "$redeem_message" ${FROM[$key]} --gas 350000)"
#    redeem_tx="$(wait_for_compute_tx "$tx_hash" "waiting for redeem from \"$key\" to process")"

    if redeem_tx="$(wait_for_compute_tx "$tx_hash" "waiting for redeem from \"$key\" to process")">/dev/null; then
        redeem_response="$(data_of wait_for_compute_tx "$tx_hash" "waiting for redeem from \"$key\" to process")"
        assert_eq "$redeem_response" "$(pad_space '{"redeem":{"status":"success"}}')"

        new_balance=$(get_balance "$token_addr" "$key")
        assert_eq "$amount" $(bc <<<"$new_balance - $old_balance")

        log "successfully redeemed ${amount} for \"$key\""
    elif ! redeem_tx="$(wait_for_compute_tx "$tx_hash" "waiting for redeem from \"$key\" to process")">/dev/null; then
        redeem_error="$(get_generic_err "$redeem_tx")"
        if ! jq -Re 'startswith("insufficient funds to redeem")' <<< "$redeem_error">/dev/null; then
            log "$redeem_error"
            return 1
        fi
#        assert_eq "$redeem_error" "$(pad_space "insufficient funds to redeem: balance=$old_balance, required=$amount")"
    fi

#    assert_eq "$redeem_response" "$(pad_space '{"redeem":{"status":"success"}}')"

}

function get_deposit() {
    local contract_addr="$1"
    local key="$2"

    log "querying deposit for \"$key\""
    local deposit_query='{"deposit":{"address":"'"${ADDRESS[$key]}"'","key":"'"${VK[$key]}"'"}}'
    local deposit_response
    deposit_response="$(compute_query "$contract_addr" "$deposit_query")"
    log "deposit response was: $deposit_response"
    jq -r '.deposit.deposit' <<<"$deposit_response"
}

function stake() {
    local contract_addr="$1"
    local key="$2"
    local amount="$3"
    local token_addr="$4"

    local deposit_message='{"deposit":{}}'
    local tx_hash
    local deposit_response
    local deposit_binary="$(base64 <<< "$deposit_message")"
    local send_message='{"send":{"recipient":"'"$contract_addr"'","amount":"'"$amount"'","msg":"'"$deposit_binary"'"}}'
    tx_hash="$(compute_execute "$token_addr" "$send_message" ${FROM[$key]} --gas 350000)"
    deposit_response="$(data_of wait_for_compute_tx "$tx_hash" "waiting for stake to \"$key\" to process")"
    assert_eq "$deposit_response" "$(pad_space '{"send":{"status":"success"}}')"
    log "deposited ${amount} to \"$key\" successfully"
}

##### actual test functions

function test_deposit() {
    local contract_addr="$1"
    local token_addr="$2"

    log_test_header

    local tx_hash

    local -A deposits=([a]=10000000000000000000 [b]=20000000000000000000 [c]=30000000000000000000 [d]=40000000000000000000)
    for key in "${KEY[@]}"; do
        stake "$contract_addr" "$key" "${deposits[$key]}" "$token_addr"
    done

    # Query the balances of the accounts and make sure they have the right balances.
    for key in "${KEY[@]}"; do
        assert_eq "$(get_deposit "$contract_addr" "$key")" "${deposits[$key]}"
    done

    # Try to overdraft
    local redeem_message
    local overdraft
    local redeem_response
    for key in "${KEY[@]}"; do
        overdraft="${deposits[$key]}0"
        redeem_message='{"redeem":{"amount":"'"$overdraft"'"}}'
        tx_hash="$(compute_execute "$contract_addr" "$redeem_message" ${FROM[$key]} --gas 150000)"
        # Notice the `!` before the command - it is EXPECTED to fail.
        ! redeem_response="$(wait_for_compute_tx "$tx_hash" "waiting for overdraft from \"$key\" to process")"
        log "trying to overdraft from \"$key\" was rejected"
        assert_eq \
            "$(get_generic_err "$redeem_response")" \
            "insufficient funds to redeem: balance=${deposits[$key]}, required=$overdraft"
    done

    # Withdraw Everything
    for key in "${KEY[@]}"; do
        redeem "$contract_addr" "$key" "${deposits[$key]}" "$token_addr"
    done

    # Check the balances again. They should all be empty
    for key in "${KEY[@]}"; do
        assert_eq "$(get_deposit "$contract_addr" "$key")" 0
    done
}

function test_viewing_key() {
    local contract_addr="$1"

    log_test_header

    # common variables
    local result
    local tx_hash

    # query balance. Should fail.
    local wrong_key
    wrong_key="$(xxd -ps <<<'wrong-key')"
    local deposit_query
    local expected_error="$(pad_space '{"query_error":{"msg":"Wrong viewing key for this address or viewing key not set"}}')"
    for key in "${KEY[@]}"; do
        log "querying deposit for \"$key\" with wrong viewing key"
        deposit_query='{"deposit":{"address":"'"${ADDRESS[$key]}"'","key":"'"$wrong_key"'"}}'
        result="$(compute_query "$contract_addr" "$deposit_query")"
        assert_eq "$result" "$expected_error"
    done

    # Create viewing keys
    local create_viewing_key_message='{"create_viewing_key":{"entropy":"MyPassword123"}}'
    local viewing_key_response
    for key in "${KEY[@]}"; do
        log "creating viewing key for \"$key\""
        tx_hash="$(compute_execute "$contract_addr" "$create_viewing_key_message" ${FROM[$key]} --gas 1400000)"
        viewing_key_response="$(data_of wait_for_compute_tx "$tx_hash" "waiting for viewing key for \"$key\" to be created")"
        VK[$key]="$(jq -er '.create_viewing_key.key' <<<"$viewing_key_response")"
        log "viewing key for \"$key\" set to ${VK[$key]}"
        if [[ "${VK[$key]}" =~ ^api_key_ ]]; then
            log "viewing key \"$key\" seems valid"
        else
            log 'viewing key is invalid'
            return 1
        fi
    done

    # Check that all viewing keys are different despite using the same entropy
    assert_ne "${VK[a]}" "${VK[b]}"
    assert_ne "${VK[b]}" "${VK[c]}"
    assert_ne "${VK[c]}" "${VK[d]}"

    # query balance. Should succeed.
    local deposit_query
    for key in "${KEY[@]}"; do
        deposit_query='{"deposit":{"address":"'"${ADDRESS[$key]}"'","key":"'"${VK[$key]}"'"}}'
        log "querying deposit for \"$key\" with correct viewing key"
        result="$(compute_query "$contract_addr" "$deposit_query")"
        if ! silent jq -e '.deposit.deposit | tonumber' <<<"$result"; then
            log "Deposit query returned unexpected response: ${result@Q}"
            return 1
        fi
    done

    # Change viewing keys
    local vk2_a

    log 'creating new viewing key for "a"'
    tx_hash="$(compute_execute "$contract_addr" "$create_viewing_key_message" ${FROM[a]} --gas 1400000)"
    viewing_key_response="$(data_of wait_for_compute_tx "$tx_hash" 'waiting for viewing key for "a" to be created')"
    vk2_a="$(jq -er '.create_viewing_key.key' <<<"$viewing_key_response")"
    log "viewing key for \"a\" set to $vk2_a"
    assert_ne "${VK[a]}" "$vk2_a"

    # query deposit with old keys. Should fail.
    log 'querying deposit for "a" with old viewing key'
    local deposit_query_a='{"deposit":{"address":"'"${ADDRESS[a]}"'","key":"'"${VK[a]}"'"}}'
    result="$(compute_query "$contract_addr" "$deposit_query_a")"
    assert_eq "$result" "$expected_error"

    # query deposit with new keys. Should succeed.
    log 'querying deposit for "a" with new viewing key'
    deposit_query_a='{"deposit":{"address":"'"${ADDRESS[a]}"'","key":"'"$vk2_a"'"}}'
    result="$(compute_query "$contract_addr" "$deposit_query_a")"
    if ! silent jq -e '.deposit.deposit | tonumber' <<<"$result"; then
        log "Deposit query returned unexpected response: ${result@Q}"
        return 1
    fi

    # Set the vk for "a" to the original vk
    log 'setting the viewing key for "a" back to the first one'
    local set_viewing_key_message='{"set_viewing_key":{"key":"'"${VK[a]}"'"}}'
    tx_hash="$(compute_execute "$contract_addr" "$set_viewing_key_message" ${FROM[a]} --gas 1400000)"
    viewing_key_response="$(data_of wait_for_compute_tx "$tx_hash" 'waiting for viewing key for "a" to be set')"
    assert_eq "$viewing_key_response" "$(pad_space '{"set_viewing_key":{"status":"success"}}')"

    # try to use the new key - should fail
    log 'querying deposit for "a" with new viewing key'
    deposit_query_a='{"deposit":{"address":"'"${ADDRESS[a]}"'","key":"'"$vk2_a"'"}}'
    result="$(compute_query "$contract_addr" "$deposit_query_a")"
    assert_eq "$result" "$expected_error"

    # try to use the old key - should succeed
    log 'querying deposit for "a" with old viewing key'
    deposit_query_a='{"deposit":{"address":"'"${ADDRESS[a]}"'","key":"'"${VK[a]}"'"}}'
    result="$(compute_query "$contract_addr" "$deposit_query_a")"
    if ! silent jq -e '.deposit.deposit | tonumber' <<<"$result"; then
        log "Deposit query returned unexpected response: ${result@Q}"
        return 1
    fi
}

function test_simulation() {
    local lockup_contract_addr="$1"
    local eth_contract_addr="$2"
    local scrt_contract_addr="$3"
    local deadline="$4"

    log_test_header

    local max_eth_amount=100
    local height=$(query_height)

    while [ $height -le $(($deadline + 10)) ]; do
        log 'Current height: '$height
        local amount=$(($RANDOM % $max_eth_amount + 1))
        amount=$(bc <<<"$amount * 10^18")

        local action=$(($RANDOM % 2))
        local user_idx=$(($RANDOM % 4))

        # Deposit
        if [ $action == 0 ]; then
            log 'Depositing with amount: ' $amount
            stake "$lockup_contract_addr" "${KEY[$user_idx]}" "$amount" "$eth_contract_addr"
        # Redeem
        elif [ $action == 1 ]; then
            log 'Redeeming with amount: ' $amount
            redeem "$lockup_contract_addr" "${KEY[$user_idx]}" "$amount" "$eth_contract_addr"
        fi

        sleep 5

        height=$(query_height)
    done

    # Withdraw Everything
    for key in "${KEY[@]}"; do
        local deposit="$(get_deposit "$lockup_contract_addr" "$key")"
        redeem "$lockup_contract_addr" "$key" "$deposit" "$eth_contract_addr"
    done

    local pool_query='{"reward_pool_balance":{}}'
    local reward_pool_balance="$(compute_query "$lockup_contract_addr" "$pool_query")"

    local reward_received=0
    for key in "${KEY[@]}"; do
        local balance=$(get_balance "$scrt_contract_addr" "$key")
        reward_received=$(bc <<< "$reward_received + $balance")
    done

    log ''
    log 'Simulation ended'
    log 'Stats:'

    log 'reward pool balance: ' $reward_pool_balance
    log 'total rewards collected: ' $reward_received
}

function main() {
    log '              <####> Starting integration tests <####>'
    log "secretcli version in the docker image is: $(secretcli version)"

    local prng_seed
    prng_seed="$(base64 <<<'enigma-rocks')"
    local init_msg

    # Store snip20 code
    local code_id
    code_id="$(upload_code '../secret-secret')"

    # secretSCRT init
    init_msg='{"name":"secret-secret","admin":"'"${ADDRESS[a]}"'","symbol":"SSCRT","decimals":6,"initial_balances":[],"prng_seed":"'"$prng_seed"'","config":{"public_total_supply":true}}'
    scrt_contract_addr="$(init_contract "$code_id" "$init_msg")"
    scrt_contract_hash="$(secretcli q compute contract-hash "$scrt_contract_addr")"
    scrt_contract_hash="${scrt_contract_hash:2}"

    # secretETH init
    init_msg='{"name":"secret-eth","admin":"'"${ADDRESS[a]}"'","symbol":"SETH","decimals":18,"initial_balances":[{"address":"'"${ADDRESS[a]}"'", "amount":"1000000000000000000000000"}, {"address":"'"${ADDRESS[b]}"'", "amount":"1000000000000000000000000"},{"address":"'"${ADDRESS[c]}"'", "amount":"1000000000000000000000000"},{"address":"'"${ADDRESS[d]}"'", "amount":"1000000000000000000000000"}],"prng_seed":"'"$prng_seed"'","config":{"public_total_supply":true}}'
    eth_contract_addr="$(init_contract "$code_id" "$init_msg")"
    eth_contract_hash="$(secretcli q compute contract-hash "$eth_contract_addr")"
    eth_contract_hash="${eth_contract_hash:2}"

    # Rewards init
    deadline=$(query_height)
    deadline=$(($deadline + 100)) # Will run for approximately ~10 minutes
    init_msg='{"reward_token":{"address":"'"$scrt_contract_addr"'", "contract_hash":"'"$scrt_contract_hash"'"}, "inc_token":{"address":"'"$eth_contract_addr"'", "contract_hash":"'"$eth_contract_hash"'"}, "deadline":'"$deadline"', "pool_claim_block":'"$deadline"', "viewing_key": "123", "prng_seed": "'"$prng_seed"'"}'
    lockup_contract_addr="$(create_contract '.' "$init_msg")"
    lockup_contract_hash="$(secretcli q compute contract-hash "$lockup_contract_addr")"
    lockup_contract_hash="${lockup_contract_hash:2}"
    log 'Deadline is: ' $deadline

    # To make testing faster, check the logs and try to reuse the deployed contract and VKs from previous runs.
    # Remember to comment out the contract deployment and `test_viewing_key` if you do.
    #    local scrt_contract_addr='secret1zyhdfsw23p6ldlqahg9daa7remw3jwyyhwq8as'
    #    local eth_contract_addr='secret1c59jeww5g7advma6vpanzxkveyndupu8w3chkd'
    #    local lockup_contract_addr='secret1smz53pmnf7jslu834qn6l4j90xk05d8cgm7qsa'
    #    VK[a]='api_key_8zWXxxJ5vd9tHfSHvX0A3d66USmyBVFh3EOisDiGmfI='
    #    VK[b]='api_key_GdJ2B3Eo/mgbDLBsyCdhfZmT3dEzPq0pGAlia2SVlK4='
    #    VK[c]='api_key_s+uYm1vMSEeq7EwRQVsApit2KcobA1OfAbF+AOGTZO4='
    #    VK[d]='api_key_kWRCXp/kFKPjvH97bU7/zLz6ygFDo9UPykTWSbnZw8E='

    # Deposit prize money and transfer to contract
    log 'depositing rewards to secretSCRT and transfer to the lockup contract'
    local rewards='500000000000'
    deposit "$scrt_contract_addr" 'a' "$rewards"
    local receiver_msg='{"deposit_rewards":{}}'
    receiver_msg="$(base64 <<<"$receiver_msg")"
    local send_message='{"send":{"recipient":"'"$lockup_contract_addr"'","amount":"'"$rewards"'","msg":"'"$receiver_msg"'"}}'
    local send_response
    tx_hash="$(compute_execute "$scrt_contract_addr" "$send_message" ${FROM[a]} --gas 500000)"
    send_response="$(wait_for_compute_tx "$tx_hash" 'waiting for deposit rewards to complete')"
    log "$send_response"

    #    balance="$(get_balance "$scrt_contract_addr" "$lockup_contract_addr")"
    #    log 'lockup contracts reward balance is: '"$balance"
    #    local receiver_state_query='{"reward_pool_balance":{}}'
    #    rewards_result="$(compute_query "$lockup_contract_addr" "$receiver_state_query")"
    #    rewards="$(jq -r '.reward_pool_balance.balance' <<<"$rewards_result")"
    #    log 'lockup contracts rewards pool is: '"$rewards"

    log '###### Contracts Details ######'
    log 'code id is: ' "$code_id"
    log ''
    log 'secret addr is: ' "$scrt_contract_addr"
    log 'secret hash is: ' "$scrt_contract_hash"
    log ''
    log 'eth addr is: ' "$eth_contract_addr"
    log 'eth hash is: ' "$eth_contract_hash"
    log ''
    log 'lockup addr is: ' "$lockup_contract_addr"
    log 'lockup hash is: ' "$lockup_contract_hash"
    log ''

    test_viewing_key "$lockup_contract_addr"
    set_viewing_keys "$eth_contract_addr"
    test_deposit "$lockup_contract_addr" "$eth_contract_addr"
    set_viewing_keys "$scrt_contract_addr"
    test_simulation "$lockup_contract_addr" "$eth_contract_addr" "$scrt_contract_addr" "$deadline"

    log 'Tests completed successfully'

    # If everything else worked, return successful status
    return 0
}

main "$@"
