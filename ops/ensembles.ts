import {
  Contract, ScrtEnsemble, EnsembleInit,
  Commands, Console, render, taskmaster, table,
  readFile, execFileSync } from '@hackbg/fadroma'

import {
  SNIP20Contract, MGMTContract, RPTContract, RewardsContract } from '@sienna/api'

import {
  abs, runDemo, genConfig, getDefaultSchedule } from './lib/index'

const { debug, warn, info } = Console(import.meta.url)

import {
  SIENNA_SNIP20, MGMT, RPT,
  AMM_FACTORY, AMM_EXCHANGE, AMM_SNIP20,
  LP_SNIP20, REWARD_POOL,
  IDO } from './contracts'

type TGESchedule    = string|Record<any, any>
type TGEInit        = EnsembleInit & { schedule?: TGESchedule, initialRPTRecipient?: string }
type TGECommandArgs = { address?: string, chain?: any }

export class SiennaTGE extends ScrtEnsemble {
  workspace = abs()
  contracts = { SIENNA_SNIP20, MGMT, RPT }

  static INFO = {
    BUILD:       '👷 Compile contracts from working tree',
    CONFIG:      '📅 Convert a spreadsheet into a JSON schedule',
    DEPLOY:      '🚀 Build, init, and deploy the TGE',
    DEMO:        '🐒 Run the TGE demo (long-running integration test)',
    UPLOAD:      '📦 Upload compiled contracts to chain',
    INIT:        '🚀 Init new instances of already uploaded contracts',
    LAUNCH:      '🚀 Launch deployed vesting contract',
    CLAIM:       '⚡ Claim funds from a deployed contract',
    STATUS:      '👀 Print the status and schedule of a contract.',
    //TRANSFER:    '⚡ Transfer ownership of contracts to another address',
    //CONFIGURE:   '⚡ Upload a new JSON config to an already initialized contract',
    //REALLOCATE:  '⚡ Update the allocations of the RPT tokens',
    //ADD_ACCOUNT: '⚡ Add a new account to a partial vesting pool'
  }

  localCommands = (): Commands => [
    ['build',  SiennaTGE.INFO.BUILD,  (_, sequential: boolean) =>
      this.build({parallel: !sequential})],
    ['config', SiennaTGE.INFO.CONFIG, (_, spreadsheet: any) =>
      genConfig(spreadsheet)]]

  remoteCommands = (): Commands => [
    ['deploy',   SiennaTGE.INFO.DEPLOY,   (context: any, schedule: any) =>
      this.deploy({...context, schedule}).then(()=>process.exit())],
    ['demo',     SiennaTGE.INFO.DEMO,
      runDemo],
    ['upload',   SiennaTGE.INFO.UPLOAD,   (context: any) =>
      this.upload(context)],
    ['init',     SiennaTGE.INFO.INIT,     (context: any, schedule: any) =>
      this.initialize({...context, schedule})],
    ['launch',   SiennaTGE.INFO.LAUNCH,   (context: any, address: any)  =>
      this.launch({...context, address})],
    ['claim',    SiennaTGE.INFO.CLAIM,    (context: any, address: any, claimant: any) =>
      this.claim({...context, address, claimant})],
    ['status',   SiennaTGE.INFO.STATUS,   (context: any, address: any) =>
      this.getStatus({...context, address})],
    //['transfer', SiennaTGE.INFO.TRANSFER, (context: any, address: any)  =>
      //this.transfer({...context, address})],
    //["configure",   SiennaTGE.INFO.CONFIGURE,
      //(context, deployment, schedule) => this.configure(deployment, schedule)],
    //['reallocate',  SiennaTGE.INFO.REALLOCATE,
      //(context, [deployment, allocations]) => this.reallocate(deployment, allocations)],
    //['add-account', SiennaTGE.INFO.ADD_ACCOUNT,
      //(context, [deployment, account]) => this.addAccount(deployment, account)],
  ]

  async initialize (options: TGEInit = {}) {
    const chain   = options.chain
        , agent   = options.agent   || this.agent || await chain.getAgent()
        , task    = options.task    || taskmaster()
        , uploads = options.uploads || await this.upload({agent, chain, task})
        , initialRPTRecipient = options.initialRPTRecipient || agent.address

    const instances: Record<string, Contract> = {}

    // too many steps - mgmt could automatically instantiate token and rpt if it supported callbacks
    await task('initialize token', async (report: Function) => {
      const codeId  = uploads.SIENNA_SNIP20.codeId
          , label   = this.contracts.SIENNA_SNIP20.label
          , initMsg = {
              ...this.contracts.SIENNA_SNIP20.initMsg,
              admin: agent.address }
      instances.SIENNA_SNIP20 = await agent.instantiate(new SNIP20Contract({codeId, label, initMsg}))
      report(instances.SIENNA_SNIP20.initTx.transactionHash) })

    // accept schedule as string or struct
    let { schedule = getDefaultSchedule() } = options
    if (typeof schedule === 'string') schedule = JSON.parse(await readFile(schedule, 'utf8'))

    await task('initialize mgmt', async (report: Function) => {
      const codeId  = uploads.MGMT.codeId
          , label   = this.contracts.MGMT.label
          , initMsg = {
              ...this.contracts.MGMT.initMsg,
              token: [instances.SIENNA_SNIP20.address, instances.SIENNA_SNIP20.codeHash],
              schedule }
      // set placeholder RPT address in schedule - updated after RPT is deployed
      schedule.pools.filter((x:any)=>x.name==='MintingPool')[0]
              .accounts.filter((x:any)=>x.name==='RPT')[0]
              .address = initialRPTRecipient
      instances.MGMT = await agent.instantiate(new MGMTContract({codeId, label, initMsg}))
      report(instances.MGMT.initTx.transactionHash) })

    await task('make mgmt owner of token', async (report: Function) => {
      const {MGMT, SIENNA_SNIP20} = instances
          , [tx1, tx2] = await (MGMT as MGMTContract).acquire(SIENNA_SNIP20)
      report(tx1.transactionHash)
      report(tx2.transactionHash) })

    await task('initialize rpt', async (report: Function) => {
      const {MGMT, SIENNA_SNIP20} = instances
          , codeId  = uploads.MGMT.codeId
          , label   = this.contracts.RPT.label
          , initMsg = {
              ...this.contracts.RPT.initMsg,
              token:   [SIENNA_SNIP20.address, SIENNA_SNIP20.codeHash],
              mgmt:    [MGMT.address,  MGMT.codeHash ],
              portion: "2500000000000000000000", // TODO get this from schedule!!!
              config:  [[initialRPTRecipient, (this.contracts.RPT.initMsg as any).portion]] }
      instances.RPT = await agent.instantiate(new RPTContract({ codeId, label, initMsg }))
      report(instances.RPT.initTx.transactionHash) })

    await task('point rpt account in mgmt schedule to rpt contract', async (report: Function) => {
      // set real RPT address in schedule
      const {MGMT, RPT} = instances
      schedule.pools.filter((x:any)=>x.name==='MintingPool')[0]
              .accounts.filter((x:any)=>x.name==='RPT')[0]
              .address = RPT.address
      const {transactionHash} = await (MGMT as MGMTContract).configure(schedule)
      report(transactionHash) })

    const { SIENNA_SNIP20, MGMT, RPT } = instances
    console.log(table(
      [ [ 'Contract\nDescription',              'Address\nCode hash' ]
      , [ 'SIENNA_SNIP20\nSienna SNIP20 token', `${SIENNA_SNIP20.address}\n${SIENNA_SNIP20.codeHash}` ]
      , [ 'MGMT\nVesting',                      `${MGMT.address}\n${MGMT.codeHash}`]
      , [ 'RPT\nRemaining pool tokens',         `${RPT.address}\n${RPT.codeHash}`] ]))

    return {chain, agent, contracts: instances} }

  async launch (options: TGECommandArgs = {}) {
    const address = options.address

    if (!address) {
      warn('TGE launch: needs address of deployed MGMT contract')
      // TODO add `error.user = true` flag to errors
      // to be able to discern between bugs and incorrect inputs
      process.exit(1) }

    info(`⏳ launching vesting MGMT contract at ${address}...`)
    const { agent } = await options.chain.connect()
        , MGMT = options.chain.getContract(MGMTContract, address, agent)

    try {
      await MGMT.launch()
      info(`🟢 launch reported success`)
      info(`⏳ querying status...`)
      debug(await MGMT.status) }
    catch (e) {
      warn(e)
      info(`🔴 launch reported a failure`) } }

  async getStatus (options: TGECommandArgs = {}) {
    const address = options.address
    if (!address) {
      warn('TGE launch: needs address of deployed MGMT contract')
      process.exit(1)
      // TODO add `error.user = true` flag to errors
      // to be able to discern between bugs and incorrect inputs
    }
    info(`⏳ querying MGMT contract at ${address}...`)
    const { agent } = await options.chain.connect()
        , MGMT = options.chain.getContract(MGMTContract, address, agent)
        , [schedule, status] = await Promise.all([MGMT.schedule, MGMT.status])
    console.log('\n'+render(schedule))
    console.log('\n'+render(status)) }

  async reallocate () { throw new Error('not implemented') }

  async addAccount () { throw new Error('not implemented') }

  async claim (_: any) { throw new Error('not implemented') } }

export class SiennaSwap extends ScrtEnsemble {
  workspace = abs()
  contracts = { AMM_FACTORY, AMM_EXCHANGE, AMM_SNIP20, LP_SNIP20, IDO }

  async initialize (args: EnsembleInit) {
    return {}
  }

  async attachTo (tge: SiennaTGE) {
  }

}

export class SiennaRewards extends ScrtEnsemble {
  workspace = abs()
  contracts = { SIENNA_SNIP20, LP_SNIP20, REWARD_POOL }

  localCommands (): Commands {
    return [...super.localCommands(),
      ["test",      '🥒 Run unit tests',    this.test.bind(this)     ],
      ["benchmark", '⛽ Measure gas costs', this.benchmark.bind(this)]] }

  async initialize (options: EnsembleInit) {
    const chain   = options.chain
        , agent   = options.agent   || this.agent || await chain.getAgent()
        , task    = options.task    || taskmaster()
        , uploads = options.uploads || await this.upload({agent, chain, task})
        , instances: Record<string, any> = {}

    await task('initialize LP token',     initTokenTask.bind(this, 'LP_SNIP20'))
    await task('initialize reward token', initTokenTask.bind(this, 'SIENNA_SNIP20'))
    await task('initialize rewards',      initRewardsTask.bind(this))
    await task('mint some rewards',       mintRewardsTask.bind(this))

    console.log(table([
      [ 'Contract\nDescription', 'Address\nCode hash' ],
      [ 'LP_SNIP20\nLiquidity provision',
        `${instances.SIENNA_SNIP20.address}\n${instances.SIENNA_SNIP20.codeHash}`],
      [ 'SIENNA_SNIP20\nSienna SNIP20 token',
        `${instances.SIENNA_SNIP20.address}\n${instances.SIENNA_SNIP20.codeHash}`],
      [ 'Rewards\n',
        `${instances.REWARD_POOL.address}\n${instances.REWARD_POOL.codeHash}`]]))

    return instances

    async function initTokenTask (key: string, report: Function) {
      const {codeId} = uploads[key]
          , {label}  = this.contracts[key]
          , initMsg  = { ...this.contracts[key].initMsg
                       , admin: agent.address }
      instances[key] = await agent.instantiate(
        new SNIP20Contract({codeId, label: `${this.prefix}_${label}`, initMsg}))
      report(instances.LP_SNIP20.transactionHash) }

    async function initRewardsTask (report: Function) {
      const {codeId} = uploads.REWARD_POOL
          , {label}  = this.contracts.REWARD_POOL
          , initMsg  = { ...this.contracts.REWARD_POOL.initMsg
                       , admin:        agent.address
                       , reward_token: instances.SIENNA_SNIP20.reference }
      instances.REWARD_POOL = await agent.instantiate(
        new RewardsContract({codeId, label: `${this.prefix}_${label}`, initMsg}))
      report(instances.REWARD_POOL.transactionHash) }

    async function mintRewardsTask (report: Function) {
      const amount = '540000000000000000000000'
      const result = await instances.SIENNA_SNIP20.mint(amount, agent, instances.REWARD_POOL.address)
      report(result) } }

  /** Attach reward pool to existing deployment. */
  async attachTo (amm: SiennaSwap) {
    throw new Error('todo!') }

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
    execFileSync(abs('node_modules/.bin/mocha'), args, { stdio: 'inherit' }) } }

export class SiennaLend extends ScrtEnsemble {
  workspace = abs()
  contracts = { SNIP20: { crate: 'snip20-lend' }
              , ATOKEN: { crate: 'atoken' }
              , CONFIG: { crate: 'configuration' } } }
