import { get_token_type, TypeOfToken } from './types.js';
function create_coin(amount) {
    return {
        denom: 'uscrt',
        amount
    };
}
export function create_fee(amount, gas) {
    if (gas === undefined) {
        gas = amount;
    }
    return {
        amount: [{ amount, denom: "uscrt" }],
        gas,
    };
}
export class SmartContract {
    constructor(address, signing_client, client) {
        this.address = address;
        this.signing_client = signing_client;
        this.client = client;
    }
    query_client() {
        if (this.client !== undefined) {
            return this.client;
        }
        return this.signing_client;
    }
}
export class FactoryContract extends SmartContract {
    constructor(address, signing_client, client) {
        super(address, signing_client, client);
        this.address = address;
        this.signing_client = signing_client;
        this.client = client;
    }
    async create_exchange(pair, fee) {
        const msg = {
            create_exchange: {
                pair
            }
        };
        if (fee === undefined) {
            fee = create_fee('700000');
        }
        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee);
    }
    async create_ido(info, fee) {
        const msg = {
            create_ido: {
                info
            }
        };
        if (fee === undefined) {
            fee = create_fee('200000');
        }
        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee);
    }
    async set_config(snip20_contract, lp_token_contract, pair_contract, ido_contract, exchange_settings, fee) {
        const msg = {
            set_config: {
                snip20_contract,
                lp_token_contract,
                pair_contract,
                ido_contract,
                exchange_settings
            }
        };
        if (fee === undefined) {
            fee = create_fee('150000');
        }
        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee);
    }
    async get_exchange_address(pair) {
        const msg = {
            get_exchange_address: {
                pair
            }
        };
        const result = await this.query_client().queryContractSmart(this.address, msg);
        return result.get_exchange_address.address;
    }
    async list_idos(pagination) {
        const msg = {
            list_idos: {
                pagination
            }
        };
        const result = await this.query_client().queryContractSmart(this.address, msg);
        return result.list_idos.idos;
    }
    async list_exchanges(pagination) {
        const msg = {
            list_exchanges: {
                pagination
            }
        };
        const result = await this.query_client().queryContractSmart(this.address, msg);
        return result.list_exchanges.exchanges;
    }
}
export class ExchangeContract extends SmartContract {
    constructor(address, signing_client, client) {
        super(address, signing_client, client);
        this.address = address;
        this.signing_client = signing_client;
        this.client = client;
    }
    async provide_liquidity(amount, tolerance, fee) {
        const msg = {
            add_liquidity: {
                deposit: amount,
                slippage_tolerance: tolerance
            }
        };
        if (fee === undefined) {
            fee = create_fee('3000000');
        }
        const transfer = add_native_balance_pair(amount);
        return await this.signing_client.execute(this.address, msg, undefined, transfer, fee);
    }
    async withdraw_liquidity(amount, recipient, fee) {
        const msg = {
            remove_liquidity: {
                amount,
                recipient
            }
        };
        if (fee === undefined) {
            fee = create_fee('2500000');
        }
        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee);
    }
    async swap(amount, expected_return, fee) {
        const msg = {
            swap: {
                offer: amount,
                expected_return
            }
        };
        if (fee === undefined) {
            fee = create_fee('2400000');
        }
        const transfer = add_native_balance(amount);
        return await this.signing_client.execute(this.address, msg, undefined, transfer, fee);
    }
    async get_pair_info() {
        const msg = 'pair_info'; //yeah...
        const result = await this.query_client().queryContractSmart(this.address, msg);
        return result.pair_info;
    }
    async simulate_swap(amount) {
        const msg = {
            swap_simulation: {
                offer: amount
            }
        };
        return await this.query_client().queryContractSmart(this.address, msg);
    }
}
export class Snip20Contract extends SmartContract {
    constructor(address, signing_client, client) {
        super(address, signing_client, client);
        this.address = address;
        this.signing_client = signing_client;
        this.client = client;
    }
    async increase_allowance(spender, amount, expiration, padding, fee) {
        const msg = {
            increase_allowance: {
                spender,
                amount,
                expiration,
                padding
            }
        };
        if (fee === undefined) {
            fee = create_fee('200000');
        }
        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee);
    }
    async get_allowance(owner, spender, key) {
        const msg = {
            allowance: {
                owner,
                spender,
                key
            }
        };
        const result = await this.query_client().queryContractSmart(this.address, msg);
        return result.allowance;
    }
    async get_balance(address, key) {
        const msg = {
            balance: {
                address,
                key
            }
        };
        const result = await this.query_client().queryContractSmart(this.address, msg);
        return result.balance.amount;
    }
    async get_token_info() {
        const msg = {
            token_info: {}
        };
        const result = await this.query_client().queryContractSmart(this.address, msg);
        return result;
    }
    get_exchange_rate() {
        /*
        const msg = {
            exchange_rate: { }
        }

        const result = await this.client.queryContractSmart(this.address, msg)
        return result as GetExchangeRateResponse
        */
        // This is hardcoded in the contract
        return {
            rate: "1",
            denom: "uscrt"
        };
    }
    async set_viewing_key(key, padding, fee) {
        const msg = {
            set_viewing_key: {
                key,
                padding
            }
        };
        if (fee === undefined) {
            fee = create_fee('200000');
        }
        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee);
    }
    async deposit(amount, padding, fee) {
        const msg = {
            deposit: {
                padding
            }
        };
        if (fee === undefined) {
            fee = create_fee('200000');
        }
        const transfer = [create_coin(amount)];
        return await this.signing_client.execute(this.address, msg, undefined, transfer, fee);
    }
    async transfer(recipient, amount, padding, fee) {
        const msg = {
            transfer: {
                recipient,
                amount,
                padding
            }
        };
        if (fee === undefined) {
            fee = create_fee('200000');
        }
        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee);
    }
    async mint(recipient, amount, padding, fee) {
        const msg = {
            mint: {
                recipient,
                amount,
                padding
            }
        };
        if (fee === undefined) {
            fee = create_fee('200000');
        }
        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee);
    }
}
export class RewardsContract extends SmartContract {
    constructor(address, signing_client, client) {
        super(address, signing_client, client);
        this.address = address;
        this.signing_client = signing_client;
        this.client = client;
    }
    async claim(lp_tokens, fee) {
        const msg = {
            claim: {
                lp_tokens
            }
        };
        if (fee === undefined) {
            fee = create_fee('200000');
        }
        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee);
    }
    async claim_simulation(address, viewing_key, current_time_secs, lp_tokens) {
        let msg = {
            claim_simulation: {
                address,
                current_time: current_time_secs,
                lp_tokens,
                viewing_key
            }
        };
        let result = await this.query_client().queryContractSmart(this.address, msg);
        return result.claim_simulation;
    }
    async lock_tokens(amount, lp_token, fee) {
        let msg = {
            lock_tokens: {
                amount,
                lp_token
            }
        };
        if (fee === undefined) {
            fee = create_fee('200000');
        }
        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee);
    }
    async retrieve_tokens(amount, lp_token, fee) {
        let msg = {
            retrieve_tokens: {
                amount,
                lp_token
            }
        };
        if (fee === undefined) {
            fee = create_fee('200000');
        }
        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee);
    }
    async get_pools() {
        const msg = 'pools';
        let result = await this.query_client().queryContractSmart(this.address, msg);
        return result.pools;
    }
    async get_accounts(address, lp_tokens, viewing_key) {
        let msg = {
            accounts: {
                address,
                lp_tokens,
                viewing_key
            }
        };
        let result = await this.query_client().queryContractSmart(this.address, msg);
        return result.accounts;
    }
}
function add_native_balance_pair(amount) {
    let result = [];
    if (get_token_type(amount.pair.token_0) == TypeOfToken.Native) {
        result.push({
            denom: 'uscrt',
            amount: amount.amount_0
        });
    }
    else if (get_token_type(amount.pair.token_1) == TypeOfToken.Native) {
        result.push({
            denom: 'uscrt',
            amount: amount.amount_1
        });
    }
    else {
        result = undefined;
    }
    return result;
}
function add_native_balance(amount) {
    let result = [];
    if (get_token_type(amount.token) == TypeOfToken.Native) {
        result.push({
            denom: 'uscrt',
            amount: amount.amount
        });
    }
    else {
        result = undefined;
    }
    return result;
}
//# sourceMappingURL=contract.js.map