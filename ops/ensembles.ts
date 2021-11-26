import { Contract, Agent } from '@fadroma/ops'
import { BaseEnsemble, ScrtCLIAgent, ScrtAgentJS } from '@fadroma/scrt'
import {
  Commands, Command, Console, render, table,
  execFileSync, existsSync, JSONDirectory, randomHex,
  readFileSync, resolve, writeFileSync } from '@fadroma/tools'
import {
  SiennaSNIP20, MGMTContract, RPTContract,
  FactoryContract as AMMFactory, AMMContract as AMMExchange, AMMSNIP20,
  RewardsContract as RewardPool, LPToken, SNIP20Contract,
  IDOContract as IDO, SwapRouterContract, LaunchpadContract } from '@sienna/api'

import { abs, genConfig, getDefaultSchedule, ONE_SIENNA, projectRoot, stringify } from './index'
import { runDemo } from './tge.demo.js'
import { EnsemblesHelp as Help } from './help'
import { AmmFactoryContract, Pagination } from '../api/siennajs/lib/amm_factory'
import { CustomToken, get_token_type, TokenType, TypeOfToken } from '../api/siennajs/lib/core'
import { Snip20Contract } from '../api/siennajs/lib/snip20'
import { unlinkSync } from 'fs'
import { BroadcastMode } from 'secretjs'

const { debug, warn, info } = Console(import.meta.url)

type TGECommandArgs = { address?: string, chain?: any }

export class SiennaTGE extends BaseEnsemble {
  localCommands = (): Commands => [
    ['build',  Help.TGE.BUILD,  (_: any, sequential: boolean) => this.build(!sequential)],
    ['config', Help.TGE.CONFIG, (_: any, spreadsheet: any)    => genConfig(spreadsheet)]]
  remoteCommands = (): Commands => [
    ['deploy', Help.TGE.DEPLOY, async (_: any) => { await this.deploy(); process.exit(0) }],
    ['demo',   Help.TGE.DEMO,   runDemo],
    ['upload', Help.TGE.UPLOAD, (_: any) => this.upload()],
    ['init',   Help.TGE.INIT,   (_: any) => this.initialize()],
    ['launch', Help.TGE.LAUNCH, (ctx: any, a: any) => this.launch({...ctx, address: a})],
    ['claim',  Help.TGE.CLAIM,  (ctx: any, a: any, c: any) => this.claim({...ctx, address: a, claimant: c})],
    ['status', Help.TGE.STATUS, (ctx: any, a: any) => this.getStatus({...ctx, address: a})] ]
  contracts = {
    SIENNA: new SiennaSNIP20({ admin: this.agent }),
    MGMT:   new MGMTContract({ admin: this.agent }),
    RPT:    new RPTContract({ admin: this.agent })}
  schedule =
    getDefaultSchedule()
  async initialize () {
    await super.initialize()
    // Deploy SIENNA token /////////////////////////////////////////////////////////////////////////
    // mgmt could automatically instantiate token and rpt if it supported callbacks
    const {SIENNA} = this.contracts
    await this.task('Initialize main SIENNA token',
      async (report: Function) => {
        Object.assign(SIENNA.init.msg, { admin: this.agent.address })
        await SIENNA.instantiate(this.agent)
        report(SIENNA.initTx.transactionHash) })
    // Deploy MGMT vesting contract ////////////////////////////////////////////////////////////////
    // use placeholder RPT address in schedule
    // updated after RPT is deployed
    const initialRPTRecipient = this.agent.address
    const RPTAccount = this.schedule.pools
      .filter((x:any)=>x.name==='MintingPool')[0].accounts
      .filter((x:any)=>x.name==='RPT')[0]
    const {MGMT} = this.contracts
    await this.task('Initialize MGMT (TGE vesting contract)',
      async (report: Function) => {
        RPTAccount.address = initialRPTRecipient
        Object.assign(MGMT.init.msg, {
          admin:    this.agent.address,
          token:    SIENNA.linkPair,
          schedule: this.schedule })
        await MGMT.instantiate(this.agent)
        report(MGMT.initTx.transactionHash)})
    await this.task('Make MGMT the owner of the token', async (report: Function) => {
      const [tx1, tx2] = await MGMT.acquire(SIENNA)
      report(tx1.transactionHash)
      report(tx2.transactionHash) })
    // Deploy RPT splitter contract ////////////////////////////////////////////////////////////////
    const {RPT} = this.contracts
    await this.task('Initialize RPT (minting pool routing contract)',
      async (report: Function) => {
        Object.assign(RPT.init.msg, {
          token:   SIENNA.linkPair,
          mgmt:    MGMT.linkPair,
          portion: RPTAccount.portion_size, // TODO get this from schedule!!!
          config:  [[initialRPTRecipient, RPTAccount.portion_size]] })
        await RPT.instantiate(this.agent)
        report(RPT.initTx.transactionHash)})
    await this.task("Point RPT's account in MGMT's schedule to RPT's actual address",
      async (report: Function) => {
        RPTAccount.address = RPT.address
        const {transactionHash} = await MGMT.configure(this.schedule)
        report(transactionHash) })
    // And we're done //////////////////////////////////////////////////////////////////////////////
    return [
      ['SiennaSNIP20\nSienna SNIP20 token', `${SIENNA.address}\n${SIENNA.codeHash}`],
      ['MGMT\nVesting',                     `${MGMT.address}\n${MGMT.codeHash}`    ],
      ['RPT\nRemaining pool tokens',        `${RPT.address}\n${RPT.codeHash}`      ] ] }

  async launch (options: TGECommandArgs = {}) {
    const address = options.address
    if (!address) {
      warn('TGE launch: needs address of deployed MGMT contract')
      // TODO add `error.user = true` flag to errors
      // to be able to discern between bugs and incorrect inputs
      process.exit(1) }
    info(`⏳ launching vesting of MGMT contract at ${address}...`)
    const { agent } = await options.chain.connect()
        , MGMT = options.chain.getContract(MGMTContract, address, agent)
    try {
      await MGMT.launch()
      info(`🟢 launch reported success; querying status...`)
      debug(await MGMT.status) }
    catch (e) {
      warn(e)
      info(`🔴 launch reported a failure`) } }

  async getStatus (options: TGECommandArgs = {}) {
    const address = options.address
    if (!address) {
      warn('TGE launch: needs address of deployed MGMT contract')
      // TODO add `error.user = true` flag to errors
      // to be able to discern between bugs and incorrect inputs
      process.exit(1) }
    info(`⏳ querying MGMT contract at ${address}...`)
    const { agent } = await options.chain.connect()
        , MGMT = options.chain.getContract(MGMTContract, address, agent)
        , [schedule, status] = await Promise.all([MGMT.schedule, MGMT.status])
    console.log('\n'+render(schedule))
    console.log('\n'+render(status)) }

  async reallocate ()  { throw new Error('TODO') }
  async addAccount ()  { throw new Error('TODO') }
  async claim (_: any) { throw new Error('TODO') } }

export class SiennaSwap extends BaseEnsemble {
  remoteCommands = (): Commands => [
    ['deploy', Help.Rewards.DEPLOY, null, [
      ['new-tge',  Help.Rewards.DEPLOY_ALL, this.deployAll.bind(this)  ],
      null,
      ...this.chain.instances.subdirs()
        .filter(this.canAttach.bind(this))
        .map((instance):Command=>
          [instance, Help.Rewards.ATTACH_TO, this.deployAttach.bind(this)])]]]

  private canAttach (prefix: string) {
    const dir = this.chain.instances.subdir(prefix, JSONDirectory)
    return (dir.has('SiennaSNIP20') && dir.has('SiennaMGMT') && dir.has('SiennaRPT')) }

  /** Deploy a single Sienna Rewards Pool + LP Token.
    * Use an existing SNIP20 token as the reward token. */
  async deployAttach (context: any) {
    console.log(context)
    await this.parseOptions(context.options)
    await this.deploy()
    process.exit() }

  /** Deploy a single Sienna Rewards Pool + LP Token + an instance of the TGE.
    * Use the TGE's token as the reward token. */
  async deployAll (context: any) {
    await this.parseOptions(context.options)
    let deployed = []
    deployed = [...deployed, ...await this.TGE.deploy()]
    deployed = [...deployed, ...await this.deploy()]
    console.log(table(deployed))
    process.exit() }

  private async parseOptions (options?: Record<string, any>) {
    if (!options) return
    if (options['agent'] === 'secretcli') this.agent = await ScrtCLIAgent.create(this.agent) }

  TGE = new SiennaTGE({chain: this.chain})

  contracts = {
    FACTORY:  new AMMFactory({  agent: this.agent }),
    EXCHANGE: new AMMExchange({ agent: this.agent }),
    AMMTOKEN: new AMMSNIP20({   agent: this.agent }),
    LPTOKEN:  new LPToken({     agent: this.agent }, `${this.prefix}_LPToken`),
    IDO:      new IDO({         agent: this.agent }),
    LAUNCHPAD: new LaunchpadContract({ agent: this.agent }),
    ROUTER:   new SwapRouterContract({ admin: this.agent }) }

  tokenContracts: Record<string, Contract> =
    this.chain ? this.getTokens() : {}

  private getTokens () {
    switch (this.chain.chainId) {
      case 'enigma-pub-testnet-3':
      case 'supernova-1-localnet':
        return this.getLocalnetTokens()
      case 'holodeck-2':
      case 'supernova-1':
        return this.getTestnetTokens()
      default:
        return {} } }

  private getLocalnetTokens () {
    return {
      sSCRT: new AMMSNIP20({
        prefix: this.prefix,
        label:  `placeholder_sSCRT`,
        initMsg: { name: 'SecretSCRT', symbol: 'SSCRT', decimals: 6, prng_seed: randomHex(36)}}),
      STEST: new AMMSNIP20({
        prefix: this.prefix,
        label:  `placeholder_STEST`,
        initMsg: { name: 'STEST', symbol: 'STEST', decimals: 9, prng_seed: randomHex(36)}}),
      SITOK: new AMMSNIP20({
        prefix: this.prefix,
        label:  `placeholder_SITOK`,
        initMsg: { name: 'SITOK', symbol: 'SITOK', decimals: 12, prng_seed: randomHex(36)}}),
      sETH: new AMMSNIP20({
        prefix: this.prefix,
        label:  `placeholder_sETH`,
        initMsg: { name: 'SecretETH', symbol: 'SETH', decimals: 15, prng_seed: randomHex(36)}})} }

  private getTestnetTokens () {
    return {
      SIENNA: this.TGE.contracts.SIENNA,
      sSCRT: new AMMSNIP20({
        address:  'secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx',
        codeHash: 'cd400fb73f5c99edbc6aab22c2593332b8c9f2ea806bf9b42e3a523f3ad06f62' }),
      STEST: new AMMSNIP20({
        address:  'secret1w9y0jala2yn4sh86dgwy3dcwg35s4qqjw932pc',
        codeHash: '78cb50a550d579eb671e05e868d26ba48f5201a2d23250c635269c889c7db829' }),
      SITOK: new AMMSNIP20({
        address:  'secret129nq840d05a0tvkranw5xesq9k0uwmn8mg7ft5',
        codeHash: '78cb50a550d579eb671e05e868d26ba48f5201a2d23250c635269c889c7db829' }),
      sETH: new AMMSNIP20({
        address:  'secret1ttg5cn3mv5n9qv8r53stt6cjx8qft8ut9d66ed',
        codeHash: '2da545ebc441be05c9fa6338f3353f35ac02ec4b02454bc49b1a66f4b9866aed' }) } }

  private loadConfig(): any {
    const path = resolve(projectRoot, 'settings', `amm-${this.chain.chainId}.json`)

    try {
      return JSON.parse(readFileSync(path, 'utf8'))
    }
    catch (e) {
      const config = {
        exchange_settings: {
          swap_fee: {
            nom: 28,
            denom: 1000
          },
          sienna_fee: {
            nom: 2,
            denom: 10000
          },
          sienna_burner: null
        },
        admin: null,
      }

      writeFileSync(path, stringify(config), 'utf8')

      info(`Created ${path}. Configure the file and re-run this command.`)
      process.exit(0)
    }
  }

  async initialize () {
    await super.initialize()
    const config = this.loadConfig()
    const { FACTORY, EXCHANGE, AMMTOKEN, LPTOKEN, IDO, ROUTER, LAUNCHPAD } = this.contracts
    const results = []
    const factory = await this.task('instantiate AMM factory', async (report: Function) => {
      Object.assign(FACTORY.init.msg, {
        snip20_contract:   { code_hash: AMMTOKEN.codeHash, id: AMMTOKEN.codeId },
        pair_contract:     { code_hash: EXCHANGE.codeHash, id: EXCHANGE.codeId },
        lp_token_contract: { code_hash: LPTOKEN.codeHash, id: LPTOKEN.codeId },
        ido_contract:      { code_hash: IDO.codeHash, id: IDO.codeId },
        launchpad_contract:      { code_hash: LAUNCHPAD.codeHash, id: LAUNCHPAD.codeId },
        router_contract:   { code_hash: ROUTER.codeHash, id: ROUTER.codeId },
        exchange_settings: config.exchange_settings,
        admin: config.admin })
      const result = await FACTORY.instantiate(this.agent)
      report(result.transactionHash)
      results.push([ 'Sienna Swap\nFactory',  `${FACTORY.address}\n${FACTORY.codeHash}` ])
      return result })
    if (this.chain && this.chain.chainId === 'enigma-pub-testnet-3') {
      for (const [name, TOKEN] of Object.entries(this.tokenContracts)) {
        await this.task(`upload code for placeholder token: ${name}`, async (report: Function) => {
          const result = await TOKEN.upload(this.agent)
          report(result.transactionHash)
          return result })
        await this.task(`deploy placeholder token: ${name}`, async (report: Function) => {
          const result = await TOKEN.instantiate(this.agent)
          report(result.transactionHash)
          results.push([`${name}\nPlaceholder SNIP20 token`, `${TOKEN.address}\n${TOKEN.codeHash}`])
          return result }) } }
    return [] } }

type RewardPairs = Record<string, number>

export class SiennaRewards extends BaseEnsemble {
  localCommands = (): Commands => [...super.localCommands(),
    ["test",      Help.Rewards.TEST,      this.test.bind(this)     ],
    ["benchmark", Help.Rewards.BENCHMARK, this.benchmark.bind(this)]]
  remoteCommands = (): Commands => [
    ['deploy', Help.Rewards.DEPLOY, null, [
      ['new-tge',  Help.Rewards.DEPLOY_ALL, this.deployAll.bind(this) ],
      null,
      ...this.chain.instances.subdirs()
        .filter(this.canAttach.bind(this))
        .map((instance):Command=>
          [instance, Help.Rewards.ATTACH_TO, this.deployAttach.bind(this)])]]]
  private canAttach (prefix: string) {
    const dir = this.chain.instances.subdir(prefix, JSONDirectory)
    return (dir.has('SiennaSNIP20') && dir.has('SiennaMGMT') && dir.has('SiennaRPT')) }
  /** Deploy a single Sienna Rewards Pool + LP Token.
    * Use an existing SNIP20 token as the reward token. */
  async deployAttach (context: any) {
    await this.parseOptions(context.options)
    await this.initPairs()
    await this.deploy()
    process.exit() }
  /** Deploy a single Sienna Rewards Pool + LP Token + an instance of the TGE.
    * Use the TGE's token as the reward token. */
  async deployAll (context: any) {
    await this.parseOptions(context.options)
    await this.initPairs()
    let deployed = []
    deployed = [...deployed, ...await this.TGE.deploy()]
    if (!this.factoryAddress) {
      deployed = [...deployed, ...await this.Swap.deploy()]
      const agent = await this.chain.getAgent()
      for (const pair of Object.keys(this.pairs)) {
        const [tokenName1, tokenName2] = pair.split('-')
        if (!tokenName2) continue
        await this.task(`Create ${pair} exchange pair`, async () => {
          const token0 = tokenName1 === 'SIENNA' ? this.TGE.contracts.SIENNA : this.Swap.tokenContracts[tokenName1]
              , token1 = this.Swap.tokenContracts[tokenName2]
          //;(agent as any).API.restClient.broadcastMode = BroadcastMode.Block
          const result = await this.Swap.contracts.FACTORY.createExchange(
            { contract_addr: token0.address, token_code_hash: token0.codeHash },
            { contract_addr: token1.address, token_code_hash: token1.codeHash }, agent)
          const exchanges = await this.task(`List exchanges`, async () => {
            const result = await this.Swap.contracts.FACTORY.listExchanges()
            return result.list_exchanges.exchanges })
          const exchangeAddr = exchanges.filter(({pair})=>(
            pair.token_0.custom_token.contract_addr === token0.address &&
            pair.token_1.custom_token.contract_addr === token1.address))[0].address
          if (!exchangeAddr) {
            throw new Error(`could not retrieve address of exchange pair ${pair} from factory`) }
          const EXCHANGE = new AMMExchange(this.agent)
          EXCHANGE.init.address = exchangeAddr
          EXCHANGE.init.agent = agent
          deployed.push([`Exchange ${pair}\nSienna Swap Pair`, `${EXCHANGE.address}\n${EXCHANGE.codeHash}`])
          const exchangeInfo = await EXCHANGE.pairInfo()
          const LPTOKEN = this.lpTokenContracts[`LP_${pair}`] = new SNIP20Contract(agent)
          LPTOKEN.init.address = exchangeInfo.pair_info.liquidity_token.address
          LPTOKEN.blob.codeHash = this.Swap.contracts.LPTOKEN.codeHash
          LPTOKEN.init.agent = agent 
          deployed.push([`LP ${pair}\nLiquidity Provision Token`, `${LPTOKEN.address}\n${LPTOKEN.codeHash}`]) }) } }
    deployed = [...deployed, ...await this.deploy()]
    console.log(table(deployed))
    process.exit() }
  private async parseOptions (options?: Record<string, any>) {
    if (!options) return
    this.factoryAddress = options['factory']
    if (options['agent'] === 'secretcli') this.agent = await ScrtCLIAgent.create(this.agent)
    if (options['premint.reward']) this.shouldPremintReward = true
    if (options['premint.admin'])  this.shouldPremintAdmin  = true }

  TGE  = new SiennaTGE({chain: this.chain})
  Swap = new SiennaSwap({chain: this.chain})

  pairs: RewardPairs = { }
  factoryAddress = ''

  contracts = { }
  lpTokenContracts: Record<string, Contract> = {}

  shouldPremintAdmin  = false
  shouldPremintReward = false

  /** Deploys reward pairs (reward pool + LP token), as well as a reward pool for staking SIENNA.
    * Configures the RPT contract to route funds to the correct reward pools.
    * Can also premint SIENNA for testing. */
  async initialize () {
    await super.initialize()
    const SIENNA    = this?.TGE?.contracts?.SIENNA
        , RPT       = this?.TGE?.contracts?.RPT
        , deployed  = []
        , rptConfig = []
    if (!SIENNA || !RPT) throw new Error("Unable to find SIENNA or RPT contract.")
    if (this.shouldPremintAdmin||this.shouldPremintReward) await this.premint()
    for (const [pair, amount] of Object.entries(this.pairs)) {
      await this.task(`Initialize a reward pool for ${pair}`, async (report: Function) => {
        const rewardPool = this.contracts[`RP_${pair}`]
        rewardPool.init.msg.admin = this.agent.address
        rewardPool.init.msg.reward_token = SIENNA.link
        rewardPool.init.msg.lp_token = (pair === 'SIENNA')
          ? this.TGE.contracts.SIENNA.link
          : this.lpTokenContracts[`LP_${pair}`].link
        await rewardPool.instantiate(this.agent)
        report(rewardPool.initReceipt.transactionHash)
        deployed.push([`${pair}\nReward pool`, `${rewardPool.address}\n${rewardPool.codeHash}`])
        rptConfig.push([rewardPool.address, String(BigInt(amount) * ONE_SIENNA)]) }) }
    await this.task(`Configure RPT to route funds to reward pools`, async (report: Function) => {
      const result = RPT.configure(rptConfig)
      report(result.transactionHash) })
    return deployed }
  async premint () {
    const agent = this.agent
    const SIENNA = this.TGE.contracts.SIENNA as SiennaSNIP20
    await this.task('allow admin to mint reward tokens',
      async (report: Function) => {
        const result = await SIENNA.addMinters(
          [agent.address], agent)
        report(result.transactionHash)
        return result })
    if (this.shouldPremintAdmin) {
      await this.task('mint test balance to admin account',
        async (report: Function) => {
          const amount  = '540000000000000000000000'
              , address = agent.address
              , result  = await SIENNA.mint(amount, agent, address)
          report(result.transactionHash)
          return result }) }
    if (this.shouldPremintReward) {
      for (const [pair, amount] of Object.entries(this.pairs)) {
        await this.task(`mint test balance to reward pool for ${pair}`,
          async (report: Function) => {
            const amount  = '540000000000000000000000'
                , address = this.contracts[`rp${pair}`].address
                , result  = await SIENNA.mint(amount, agent, address)
            report(result.transactionHash)
            return result }) } } }
  test (_: any, ...args: any) {
    args = ['test', '-p', 'sienna-rewards', ...args]
    execFileSync('cargo', args, {
      stdio: 'inherit',
      env: { ...process.env, RUST_BACKTRACE: 'full' } }) }
  benchmark () {
    /* stupid esmodule import issue when running mocha programmatically
     * their CLI works fine though...
    const mocha = new Mocha()
    mocha.addFile(abs('api/Rewards.spec.js'))
    mocha.run(fail => process.exit(fail ? 1 : 0))*/
    const args = ['-p', 'false', 'api/Rewards.spec.js']
    execFileSync(abs('node_modules/.bin/mocha'), args, { stdio: 'inherit' }) }

  private async initPairs() {
    if (!this.factoryAddress) {
      this.pairs = {
        'SIENNA':       500,
        'SIENNA-sSCRT': 400,
        'SITOK-STEST':  500,
        'SIENNA-STEST': 300,
        'SIENNA-SITOK': 300,
        'SIENNA-sETH':  200,
        'sSCRT-STEST':  300 }
    } else {
      const path = resolve(projectRoot, 'settings', `rewards-${this.factoryAddress}.json`)

      if (existsSync(path)) {
        this.pairs = JSON.parse(readFileSync(path, 'utf8'))
        unlinkSync(path) // delete because we don't need to persist this as it might not be up-to-date later
      } else {
        info(`Querying pairs from factory(${this.factoryAddress})...`)
        const pairs = await this.queryPairs()

        writeFileSync(path, stringify(pairs), 'utf8')

        info(`Created ${path}. Configure the file and re-run this command.`)
        process.exit(0)
      }
    }

    this.contracts = rewardPools(this.agent, Object.keys(this.pairs))
  }

  private async queryPairs(): Promise<RewardPairs> {
    if (!this.agent) {
      this.agent = await this.chain.getAgent()
    }

    const factory = new AmmFactoryContract(this.factoryAddress, (this.agent as ScrtAgentJS).API)
    const pairs: RewardPairs = { }

    let index = 0
    const limit = 30

    while(true) {
      const resp = await factory.query().list_exchanges(new Pagination(index, limit))

      if (resp.length === 0)
        break

      index += limit

      for(const pair of resp) {
        const name_0 = await this.getTokenName(pair.pair.token_0)
        const name_1 = await this.getTokenName(pair.pair.token_1)

        const key = `${name_0}-${name_1}`
        pairs[key] = 100
      }
    }

    return pairs
  }

  private async getTokenName(token: TokenType): Promise<string> {
    if (get_token_type(token) === TypeOfToken.Native) {
      return 'scrt'
    } else {
      const snip20 = (token as CustomToken).custom_token
      const contract = new Snip20Contract(snip20.contract_addr, (this.agent as ScrtAgentJS).API)

      const info = await contract.query().get_token_info()

      return info.name
    }
  }
}

export function rewardPools (agent: Agent, pairs: Array<string>) {
  const pools = {}
  for (const pair of pairs) {
    pools[`LP_${pair}`] = new LPToken(agent, pair)
    pools[`RP_${pair}`] = new RewardPool(agent, pair) }
  return pools }

export class SiennaLend extends BaseEnsemble {
  contracts = {/* SNIP20: { crate: 'snip20-lend' }
              , ATOKEN: { crate: 'atoken' }
              , CONFIG: { crate: 'configuration' } */} }
