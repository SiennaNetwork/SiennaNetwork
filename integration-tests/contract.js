import SecretNetworkContract from '@hackbg/fadroma/js/contract.js'

export class Callback {
    constructor(msg, contract_addr, contract_code_hash) {
        this.msg = msg
        this.contract_addr = contract_addr
        this.contract_code_hash = contract_code_hash
    }
}

export class ContractInstantiationInfo {
    constructor(code_hash, id) {
        this.code_hash = code_hash
        this.id = id
    }
}

export class ContractInfo {
    constructor(code_hash, address) {
        this.code_hash = code_hash
        this.address = address
    }
}

export class TokenPair {
    constructor(token_0, token_1) {
        this.token_0 = token_0
        this.token_1 = token_1
    }
}

export class TokenPairAmount {
    constructor(pair, amount_0, amount_1) {
        this.amount_0 = amount_0
        this.amount_1 = amount_1
        this.pair = pair
    }
}

export class TokenTypeAmount {
    constructor(token, amount) {
        this.token = token
        this.amount = amount
    }
}

export class FactoryContract extends SecretNetworkContract { 
    static async instantiate (say, commit, snip20_contract, pair_contract, ido_contract, sienna_token) {
        const name = 'amm-factory'
        const binary = `${commit}-${name}.wasm`

        let args = {}
        args.name = name
        args.binary = binary
        args.data = { 
            snip20_contract,
            pair_contract,
            ido_contract,
            sienna_token
        }

        say = say.tag(`#${this.name}`)

        const label = `${name} (${new Date().toISOString()})`

        return super.deploy({say, binary, label, name, ...args})
    }

    async create_exchange(pair) {
        return await this.execute('create_exchange', {pair})
    }

    async register_exchange(pair) {
        return await this.execute('register_exchange', {pair})
    }

    async get_exchange_address(pair) {
        const {address} = await this.query('get_exchange_address', {pair});
        return address
    }

    async get_exchange_pair(address) {
        const {pair} = await this.query('get_exchange_pair', {exchange_addr: address});
        return pair
    }
}

export class ExchangeContract { 
    constructor(name, address, agent) {
        this.info = { name, address };
        this.agent = agent;
    }

    async add_liquidity(tokens_amount) {
        return await this.agent.execute(this.info, 'add_liquidity', {
            deposit: tokens_amount
        })
    }

    async remove_liquidity(amount, recipient) {
        return await this.agent.execute(this.info, 'remove_liquidity', {
            amount,
            recipient 
        })
    }

    async swap(token_amount) {
        return await this.agent.execute(this.info, 'add_liquidity', {
            offer: token_amount
        })
    }

    async query_pair_info() {
        const {pair_info} = await this.agent.execute(this.info, "pair_info")
        return pair_info
    }

    async query_factory_info() {
        const {factory_info} = await this.agent.execute(this.info, "factory_info")
        return factory_info
    }

    async query_pool() {
        const {pool} = await this.agent.execute(this.info, "pool")
        return pool
    }

    async swap_preview() {
        return await this.agent.execute(this.info, "swap_simulation")
    }
}

export class SNIP20Contract extends SecretNetworkContract {

    static async deployNewToken (name, agent, binary, init_msg) {
        let args = {}
        args.name   = `TOKEN{${name}}`
        args.binary = binary
        args.data = init_msg
        args.agent = agent
        args.label = `${name} (${new Date().toISOString()})`

        const result = super.deploy(args)

        return this(result.name, result.address, result.agent)
    }

    constructor(name, address, agent) {
        this.info = { name, address };
        this.agent = agent;
    }
  
    async createViewingKey (agent, address, entropy = "minimal") {
      const response = await agent.execute(this, 'create_viewing_key', { entropy })
      const {create_viewing_key:{key}} = JSON.parse(response.data)
      this.say.tag(` #new-viewing-key`)({'for': address, key})
      return key
    }
  
    async balance ({ agent, viewkey, address }) {
      const {balance:{amount}} = await this.query('balance', {key: viewkey, address}, agent)
      return amount
    }
  
    async setMinters (minters = []) {
      return await this.execute('set_minters', {minters})
    }
  
    async changeAdmin (address) {
      return await this.execute('change_admin', {address})
    } 
}
