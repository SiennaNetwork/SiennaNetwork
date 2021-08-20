import { execFileSync } from 'child_process'

import { Scrt } from '@fadroma/agent'
import { Ensemble, InitArgs } from '@fadroma/ensemble'
import { Console, render, taskmaster, table } from '@fadroma/cli'
import { readFile } from '@fadroma/util-sys'

import { SNIP20Contract, MGMTContract, RPTContract, RewardsContract } from '@sienna/api'
import { abs, runDemo } from './lib/index'
import { genConfig, getDefaultSchedule } from './lib/gen'

const { debug, log, warn, error, info, table } = Console(import.meta.url)

import { SIENNA_SNIP20, MGMT, RPT,
         AMM_FACTORY, AMM_EXCHANGE, AMM_SNIP20,
         LP_SNIP20, REWARD_POOL,
         IDO } from './contracts'

export class SiennaTGE extends Ensemble {
  workspace = abs()
  contracts = { SIENNA_SNIP20, MGMT, RPT }

  async initialize (options: InitArgs = {}) {
    // idempotency support:
    // passing existing `contracts` to this makes it a no-op
    const { contracts = {} } = options
    if (Object.keys(contracts).length > 0) {
      return contracts }

    // these may belong in the super-method
    const network = Scrt.hydrate(options.network || this.network)
    const agent = options.agent || this.agent || await network.getAgent()

    // accept schedule as string or struct
    let { schedule = getDefaultSchedule() } = options
    if (typeof schedule === 'string') schedule = JSON.parse(await readFile(schedule, 'utf8'))
    //log(render(schedule))

    // unwrap remaining options
    const { task                = taskmaster()
          , receipts            = await this.upload({agent, network, task})
          , inits               = this.contracts
          , initialRPTRecipient = agent.address } = options

    // too many steps - mgmt could automatically instantiate token and rpt if it supported callbacks
    await task('initialize token', async report => {
      const {codeId} = receipts.SIENNA_SNIP20, {label, initMsg} = inits.SIENNA_SNIP20
      initMsg.admin = agent.address
      contracts.SIENNA_SNIP20 = await agent.instantiate(new SNIP20Contract({codeId, label, initMsg}))
      report(contracts.SIENNA_SNIP20.initTx.transactionHash) })

    await task('initialize mgmt', async report => {
      const {codeId} = receipts.MGMT, {label, initMsg} = inits.MGMT
      initMsg.token    = [contracts.SIENNA_SNIP20.address, contracts.SIENNA_SNIP20.codeHash]
      initMsg.schedule = schedule
      console.log({schedule})
      schedule.pools.filter(x=>x.name==='MintingPool')[0]
              .accounts.filter(x=>x.name==='RPT')[0]
              .address = agent.address
      contracts.MGMT = await agent.instantiate(new MGMTContract({codeId, label, initMsg}))
      report(contracts.MGMT.initTx.transactionHash) })

    await task('make mgmt owner of token', async report => {
      const {MGMT, SIENNA_SNIP20} = contracts
          , [tx1, tx2] = await MGMT.acquire(SIENNA_SNIP20)
      report(tx1.transactionHash)
      report(tx2.transactionHash) })

    await task('initialize rpt', async report => {
      const {codeId}              = receipts.RPT
          , {label, initMsg}      = inits.RPT
          , {MGMT, SIENNA_SNIP20} = contracts
      initMsg.token   = [SIENNA_SNIP20.address, SIENNA_SNIP20.codeHash]
      initMsg.mgmt    = [MGMT.address,  MGMT.codeHash ]
      initMsg.portion = "2500000000000000000000" // TODO get this from schedule!!!
      initMsg.config  = [[initialRPTRecipient, initMsg.portion]]
      contracts.RPT = await agent.instantiate(new RPTContract({ codeId, label, initMsg }))
      report(contracts.RPT.initTx.transactionHash) })

    await task('point rpt account in mgmt schedule to rpt contract', async report => {
      const {MGMT, RPT} = contracts
      schedule.pools.filter(x=>x.name==='MintingPool')[0]
              .accounts.filter(x=>x.name==='RPT')[0]
              .address = RPT.address
      const {transactionHash} = await MGMT.configure(schedule)
      report(transactionHash) })

    table([ [ 'Contract\nDescription'
            , 'Address\nCode hash' ]
          , [ 'SIENNA_SNIP20\nSienna SNIP20 token'
            , `${contracts.SIENNA_SNIP20.address}\n${contracts.SIENNA_SNIP20.codeHash}` ]
          , [ 'MGMT\nVesting'
            , `${contracts.MGMT.address}\n${contracts.MGMT.codeHash}`]
          , [ 'RPT\nRemaining pool tokens'
            , `${contracts.RPT.address}\n${contracts.RPT.codeHash}`] ])

    return {network, agent, contracts} }

  async launch (options = {}) {
    const address = options.address

    if (!address) {
      warn('TGE launch: needs address of deployed MGMT contract')
      // TODO add `error.user = true` flag to errors
      // to be able to discern between bugs and incorrect inputs
      process.exit(1) }

    info(`⏳ launching vesting MGMT contract at ${address}...`)
    const network = Scrt.hydrate(options.network || this.network)
    const { agent } = await network.connect()
    const MGMT = network.getContract(MGMTContract, address, agent)

    try {
      await MGMT.launch()
      info(`🟢 launch reported success`)
      info(`⏳ querying status...`)
      debug(await MGMT.status) }
    catch (e) {
      warn(e)
      info(`🔴 launch reported a failure`) } }

  async getStatus (options = {}) {
    const address = options.address
    if (!address) {
      warn('TGE launch: needs address of deployed MGMT contract')
      process.exit(1)
      // TODO add `error.user = true` flag to errors
      // to be able to discern between bugs and incorrect inputs
    }
    info(`⏳ querying MGMT contract at ${address}...`)
    const network = Scrt.hydrate(options.network || this.network)
    const { agent } = await network.connect()
    const MGMT = network.getContract(MGMTContract, address, agent)
    const [schedule, status] = await Promise.all([MGMT.schedule, MGMT.status])
    console.log('\n'+render(schedule))
    console.log('\n'+render(status)) }

  async reallocate () { throw new Error('not implemented') }

  async addAccount () { throw new Error('not implemented') }

  async claim (options = {}) { throw new Error('not implemented') }

  localCommands = () => [
    ["build",       '👷 Compile contracts from working tree',
      (_, sequential) => this.build(sequential)],
    ['config',      '📅 Convert a spreadsheet into a JSON schedule',
      (_, spreadsheet) => genConfig(spreadsheet)]]

  remoteCommands = () => [
    ["deploy",       '🚀 Build, init, and deploy the TGE',
      (context, schedule) => this.deploy({...context, schedule}).then(process.exit)],
    ["demo",         '🐒 Run the TGE demo (long-running integration test)',
      runDemo],
    ["upload",       '📦 Upload compiled contracts to network',
      (context)           => this.upload(context)],
    ["init",         '🚀 Init new instances of already uploaded contracts',
      (context, schedule) => this.initialize({...context, schedule})],
    ["launch",       '🚀 Launch deployed vesting contract',
      (context, address)  => this.launch({...context, address})],
    ["transfer",     '⚡ Transfer ownership of contracts to another address',
      (context, address)  => this.transfer({...context, address})],
    ['claim',        '⚡ Claim funds from a deployed contract',
      (context, address, claimant) => this.claim({...context, address, claimant})],
    ['status',       '👀 Print the status and schedule of a contract.',
      (context, address) => this.getStatus({...context, address})],
    /*["configure",   '⚡ Upload a new JSON config to an already initialized contract',
      //(context, deployment, schedule) => this.configure(deployment, schedule)],
    //['reallocate',  '⚡ Update the allocations of the RPT tokens',
      //(context, [deployment, allocations]) => this.reallocate(deployment, allocations)],
    //['add-account', '⚡ Add a new account to a partial vesting pool',
      //(context, [deployment, account]) => this.addAccount(deployment, account)],*/] }

export class SiennaSwap extends Ensemble {
  workspace = abs()
  contracts = { AMM_FACTORY, AMM_EXCHANGE, AMM_SNIP20, LP_SNIP20, IDO } }

export class SiennaRewards extends Ensemble {
  workspace = abs()
  contracts = { SIENNA_SNIP20, LP_SNIP20, REWARD_POOL }
  async initialize (args: InitArgs) {
    const { network, receipts, agent = network.agent } = args
    //throw new Error('todo!')
    const instances: Record<string, any> = {}
    const task = taskmaster()
    await task('initialize LP token',     initTokenTask.bind(this, 'LP_SNIP20'))
    await task('initialize reward token', initTokenTask.bind(this, 'SIENNA_SNIP20'))
    await task('initialize rewards',      initRewardsTask.bind(this))
    await task('mint some rewards',       mintRewardsTask.bind(this))
    console.log(instances)
    console.log(table([
      [ 'Contract\nDescription',
        'Address\nCode hash' ],
      [ 'LP_SNIP20\nLiquidity provision',
        `${instances.SIENNA_SNIP20.address}\n${instances.SIENNA_SNIP20.codeHash}`],
      [ 'SIENNA_SNIP20\nSienna SNIP20 token',
        `${instances.SIENNA_SNIP20.address}\n${instances.SIENNA_SNIP20.codeHash}`],
      [ 'Rewards\n',
        `${instances.REWARD_POOL.address}\n${instances.REWARD_POOL.codeHash}`]]))
    return instances

    async function initTokenTask (key: string, report: Function) {
      const {codeId} = receipts[key]
          , {label}  = this.contracts[key]
          , initMsg  = { ...this.contracts[key].initMsg
                       , admin: agent.address }
      instances[key] = await agent.instantiate(
        new SNIP20Contract({codeId, label: `${this.prefix}_${label}`, initMsg}))
      report(instances.LP_SNIP20.transactionHash) }

    async function initRewardsTask (report: Function) {
      const {codeId} = receipts.REWARD_POOL
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
  async augment (tge: TGEContracts) {
    throw new Error('todo!') }

  localCommands () {
    return [
      ...super.localCommands(),
      ["test",      '🥒 Run unit tests',    this.test.bind(this)     ],
      ["benchmark", '⛽ Measure gas costs', this.benchmark.bind(this)]] }

  test (context: object, ...args: any) {
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

export class SiennaLend extends Ensemble {
  workspace = abs()
  contracts = { SNIP20: { crate: 'snip20-lend' }
              , ATOKEN: { crate: 'atoken' }
              , CONFIG: { crate: 'configuration' } } }
