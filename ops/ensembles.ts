import { Contract, ScrtEnsemble, EnsembleInit, Agent,
         Commands, Command, Console, render, taskmaster, table,
         readFile, execFileSync, timestamp, decode } from '@hackbg/fadroma'

import { SNIP20Contract, MGMTContract, RPTContract, RewardsContract } from '@sienna/api'

import { abs, genConfig, getDefaultSchedule } from './index'

import { runDemo } from './tge.demo.js'

import { EnsemblesHelp as Help } from './help'

const { debug, warn, info } = Console(import.meta.url)

import { SIENNA_SNIP20, MGMT, RPT,
         AMM_FACTORY, AMM_EXCHANGE, AMM_SNIP20,
         LP_SNIP20, REWARD_POOL,
         IDO } from './contracts'

type TGESchedule = string|Record<any, any>

type TGEInit = EnsembleInit & { schedule?: TGESchedule, initialRPTRecipient?: string }

type TGECommandArgs = { address?: string, chain?: any }

export class SiennaTGE extends ScrtEnsemble {

  prefix: string = `${timestamp()}`

  workspace = abs()

  contracts = { SIENNA_SNIP20, MGMT, RPT }

  localCommands = (): Commands => [
    ['build',  Help.TGE.BUILD,  (_, sequential: boolean) => this.build({parallel: !sequential})],
    ['config', Help.TGE.CONFIG, (_, spreadsheet: any)    => genConfig(spreadsheet)]]

  remoteCommands = (): Commands => [
    ['deploy', Help.TGE.DEPLOY, (context: any, schedule: any)=>this.deploy({...context, schedule}).then(()=>process.exit())],
    ['demo',   Help.TGE.DEMO,   runDemo],
    ['upload', Help.TGE.UPLOAD, (context: any)=>this.upload(context)],
    ['init',   Help.TGE.INIT,   (context: any, schedule: any)=>this.initialize({...context, schedule})],
    ['launch', Help.TGE.LAUNCH, (context: any, address: any)=>this.launch({...context, address})],
    ['claim',  Help.TGE.CLAIM,  (context: any, address: any, claimant: any)=>this.claim({...context, address, claimant})],
    ['status', Help.TGE.STATUS, (context: any, address: any)=>this.getStatus({...context, address})],
    /*['transfer', Help.TGE.TRANSFER, (context: any, address: any)=>//this.transfer({...context, address})],
      //["configure",   Help.TGE.CONFIGURE,//(context, deployment, schedule) => this.configure(deployment, schedule)],
      //['reallocate',  Help.TGE.REALLOCATE,//(context, [deployment, allocations]) => this.reallocate(deployment, allocations)],
      //['add-account', Help.TGE.ADD_ACCOUNT,//(context, [deployment, account]) => this.addAccount(deployment, account)]*/ ]

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
          , label   = `${this.prefix}_${this.contracts.SIENNA_SNIP20.label}`
          , initMsg = {
              ...this.contracts.SIENNA_SNIP20.initMsg,
              admin: agent.address }
      instances.SIENNA_SNIP20 = await agent.instantiate(new SNIP20Contract({codeId, label, initMsg}))
      report(instances.SIENNA_SNIP20.initTx.transactionHash) })

    // accept schedule as string or struct
    let { schedule = getDefaultSchedule() } = options
    if (typeof schedule === 'string') schedule = JSON.parse(await readFile(schedule, 'utf8'))

    // use placeholder RPT address in schedule - updated after RPT is deployed
    const setRPTAddress = (address: string) => {
      schedule.pools.filter((x:any)=>x.name==='MintingPool')[0]
              .accounts.filter((x:any)=>x.name==='RPT')[0]
              .address = address }

    await task('initialize mgmt', async (report: Function) => {
      setRPTAddress(initialRPTRecipient)
      instances.MGMT = await agent.instantiate(new MGMTContract({
        codeId:  uploads.MGMT.codeId,
        label:   `${this.prefix}_${this.contracts.MGMT.label}`,
        initMsg: {
          ...this.contracts.MGMT.initMsg,
          token: instances.SIENNA_SNIP20.referencePair,
          schedule }}))
      report(instances.MGMT.initTx.transactionHash) })

    await task('make mgmt owner of token', async (report: Function) => {
      const {MGMT, SIENNA_SNIP20} = instances
          , [tx1, tx2] = await (MGMT as MGMTContract).acquire(SIENNA_SNIP20)
      report(tx1.transactionHash)
      report(tx2.transactionHash) })

    await task('initialize rpt', async (report: Function) => {
      instances.RPT = await agent.instantiate(new RPTContract({
        codeId: uploads.RPT.codeId,
        label:  `${this.prefix}_${this.contracts.RPT.label}`,
        initMsg: {
          ...this.contracts.RPT.initMsg,
          token:   instances.SIENNA_SNIP20.referencePair,
          mgmt:    instances.MGMT.referencePair,
          portion: "2500000000000000000000", // TODO get this from schedule!!!
          config:  [[initialRPTRecipient, "2500000000000000000000"]] } }))
      report(instances.RPT.initTx.transactionHash) })

    await task('point rpt account in mgmt schedule to rpt contract', async (report: Function) => {
      setRPTAddress(instances.RPT.address)
      const {transactionHash} = await (instances.MGMT as MGMTContract).configure(schedule)
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

export class SiennaSwap extends ScrtEnsemble {
  prefix: string = `${timestamp()}`
  workspace = abs()
  contracts = { AMM_FACTORY, AMM_EXCHANGE, AMM_SNIP20, LP_SNIP20, IDO }
  async initialize (_: EnsembleInit) { throw new Error('TODO!'); return {} } }

type RewardsInit = EnsembleInit & { rewardToken?: SNIP20Contract }

export class SiennaRewards extends ScrtEnsemble {

  prefix: string = `${timestamp()}`

  workspace = abs()

  contracts = { LP_SNIP20, REWARD_POOL }

  instances: Record<string, any> = {}

  localCommands = (): Commands => [...super.localCommands(),
    ["test",      Help.Rewards.TEST,      this.test.bind(this)     ],
    ["benchmark", Help.Rewards.BENCHMARK, this.benchmark.bind(this)]]

  remoteCommands = (): Commands => [
    ['deploy', Help.Rewards.DEPLOY, null, [
      ['all',  Help.Rewards.DEPLOY_ALL,  this.deployAll.bind(this)  ],
      ['this', Help.Rewards.DEPLOY_THIS, this.deployThis.bind(this) ],
      ...[].map((instance):Command=>
        [instance, Help.Rewards.ATTACH_TO, this.deployAttach.bind(this)])]]]

  /** Deploy a single Sienna Rewards Pool + LP Token + an instance of the TGE.
    * Use the TGE's token as the reward token. */
  private async deployAll (context: any, schedule: any) {
    if (context.options['premint.reward']) this.shouldPremintReward = true
    if (context.options['premint.admin'])  this.shouldPremintAdmin  = true
    const {chain} = context
    await chain.init()
    const TGE = await new SiennaTGE({chain}).deploy({...context, schedule})
    this.instances.SIENNA_SNIP20 = TGE.contracts.SIENNA_SNIP20
    await this.deploy(context)
    process.exit() }

  /** Deploy a single Sienna Rewards Pool + LP Token + Reward Token. */
  private async deployThis (context: any) {
    if (context.options['premint.reward']) this.shouldPremintReward = true
    if (context.options['premint.admin'])  this.shouldPremintAdmin  = true
    Object.assign(this.contracts, { SIENNA_SNIP20 })
    await this.deploy(context)
    process.exit() }

  /** Deploy a single Sienna Rewards Pool + LP Token.
    * Use an existing SNIP20 token as the reward token. */
  private async deployAttach (context: any) {
    if (context.options['premint.reward']) this.shouldPremintReward = true
    if (context.options['premint.admin'])  this.shouldPremintAdmin  = true
    await this.deploy(context)
    process.exit() }

  async initialize (options: RewardsInit) {
    const deployed = [[ 'Contract\nDescription', 'Address\nCode hash' ]]
    const { chain,
            agent   = this.agent || await chain.getAgent(),
            task    = taskmaster(),
            uploads = await this.upload({agent, chain, task}) } = options
    // The reward token is pluggable: if rewardToken is not provided
    // (by deployAttach) a SNIP20 is automatically deployed
    if (this.contracts['SIENNA_SNIP20']) {
      deployed.push(await task(
        'initialize reward token',
        this.initToken.bind(this, uploads, 'SIENNA_SNIP20', agent))) }
    // reward token is pluggable - existing token can be passed to the deployment
    deployed.push(await task('initialize LP token',
      this.initToken.bind(this, uploads, 'LP_SNIP20', agent)))
    deployed.push(await task(
      'initialize rewards',
      this.initRewards.bind(this, uploads, agent)))
    if (this.instances['SIENNA_SNIP20']) {
      if (this.shouldPremintAdmin || this.shouldPremintReward) {
        await task('allow admin to mint tokens',
          this.allowMintingByAdmin.bind(this, agent)) }
      this.shouldPremintAdmin && await task('mint test balance to admin account',
        this.premintAdmin.bind(this, agent))
      this.shouldPremintReward && await task('mint test balance to rewards contract',
        this.premintReward.bind(this, agent)) }
    console.log(table(deployed))
    return this.instances }

  private async initToken (
    uploads: Record<string, any>,
    key:     string,
    agent:   Agent,
    report:  Function
  ) {
    const {address, codeHash} = this.instances[key] =
      await agent.instantiate(new SNIP20Contract({
        codeId:  uploads[key].codeId,
        label:   `${this.prefix}_${this.contracts[key].label}`,
        initMsg: { ...this.contracts[key].initMsg,
                   admin: agent.address }}))
    report(this.instances[key].transactionHash)
    return [`${key}\nToken`, `${address}\n${codeHash}`]}

  private async initRewards (
    uploads: Record<string, any>,
    agent:   Agent,
    report:  Function
  ) {
    const {address, codeHash} = this.instances.REWARD_POOL =
      await agent.instantiate(new RewardsContract({
        codeId: uploads.REWARD_POOL.codeId,
        label: `${this.prefix}_${this.contracts.REWARD_POOL.label}`,
        initMsg: { ...this.contracts.REWARD_POOL.initMsg,
                   admin:        agent.address,
                   reward_token: this.instances.SIENNA_SNIP20.reference }}))
    report(this.instances.REWARD_POOL.transactionHash)
    return ['REWARD_POOL\nReward pool', `${address}\n${codeHash}`]}

  shouldPremintAdmin  = false
  shouldPremintReward = false
  private async allowMintingByAdmin (agent: Agent, report: Function) {
    const address = this.instances.REWARD_POOL.address
        , result  = await this.instances.SIENNA_SNIP20.addMinters([address], agent)
    //console.log(decode(Buffer.from(Object.values(result.data) as any)))
    report(result.transactionHash)
    return result }
  private async premintAdmin (agent: Agent, report: Function) {
    const amount  = '540000000000000000000000'
        , address = agent.address
        , result  = await this.instances.SIENNA_SNIP20.mint(amount, agent, address)
    report(result.transactionHash)
    return result }
  private async premintReward (agent: Agent, report: Function) {
    const amount  = '540000000000000000000000'
        , address = this.instances.REWARD_POOL.address
        , result  = await this.instances.SIENNA_SNIP20.mint(amount, agent, address)
    report(result.transactionHash)
    return result }

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
