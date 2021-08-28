import { Contract, ScrtEnsemble, EnsembleInit, ScrtCLIAgent, Agent,
         Commands, Command, Console, render, taskmaster, table,
         readFile, execFileSync, timestamp, decode } from '@hackbg/fadroma'
import { abs, genConfig, getDefaultSchedule } from './index'
import { runDemo } from './tge.demo.js'
import { EnsemblesHelp as Help } from './help'
import { SiennaSNIP20, MGMT as MGMTConfig, RPT as RPTConfig,
         AMMFactory, AMMExchange, AMMSNIP20,
         RewardPool, rewardPools,
         IDO } from './contracts'

const { debug, warn, info } = Console(import.meta.url)

type TGESchedule = string|Record<any, any>

type TGECommandArgs = { address?: string, chain?: any }

export class SiennaTGE extends ScrtEnsemble {

  contracts = {
    SIENNA: new SiennaSNIP20(this.agent),
    MGMT:   new MGMTConfig(this.agent),
    RPT:    new RPTConfig(this.agent)
  }

  schedule: TGESchedule

  localCommands = (): Commands => [

    ['build',  Help.TGE.BUILD,  (_: any, sequential: boolean) =>
      this.build(!sequential)],

    ['config', Help.TGE.CONFIG, (_: any, spreadsheet: any) =>
      genConfig(spreadsheet)]]

  remoteCommands = (): Commands => [

    ['deploy', Help.TGE.DEPLOY, async (_: any, schedule: any) => {
      this.schedule = schedule
      await this.deploy()
      process.exit(0) }],

    ['demo',   Help.TGE.DEMO,
      runDemo],

    ['upload', Help.TGE.UPLOAD, (_: any) =>
      this.upload()],

    ['init',   Help.TGE.INIT,   (_: any, schedule: any) => {
      this.schedule = schedule
      return this.initialize() }],

    ['launch', Help.TGE.LAUNCH, (context: any, address: any) =>
      this.launch({...context, address})],

    ['claim',  Help.TGE.CLAIM,  (context: any, address: any, claimant: any) =>
      this.claim({...context, address, claimant})],

    ['status', Help.TGE.STATUS, (context: any, address: any) =>
      this.getStatus({...context, address})] ]

  async initialize () {
    await this.chain.init()
    this.agent = await this.chain.getAgent()

    // mgmt could automatically instantiate token and rpt if it supported callbacks
    const {SIENNA} = this.contracts
    await this.task('initialize token',
      async (report: Function) => {
        SIENNA.init.label = `${this.prefix}_${SIENNA.label}`
        Object.assign(SIENNA.init.msg, { admin: this.agent.address })
        await SIENNA.upload(this.agent)
        await SIENNA.instantiate(this.agent)
        report(SIENNA.initTx.transactionHash) })

    // accept schedule as string or struct
    const schedule = getDefaultSchedule()

    // use placeholder RPT address in schedule
    // updated after RPT is deployed
    const initialRPTRecipient = this.agent.address
    const setRPTAddress = (address: string) => {
      schedule.pools.filter((x:any)=>x.name==='MintingPool')[0]
              .accounts.filter((x:any)=>x.name==='RPT')[0]
              .address = address }
    const {MGMT} = this.contracts
    console.log(SIENNA.linkPair, SIENNA)
    await this.task('initialize mgmt',
      async (report: Function) => {
        setRPTAddress(initialRPTRecipient)
        MGMT.init.label = `${this.prefix}_${MGMT.init.label}`
        Object.assign(MGMT.init.msg, {
          admin: this.agent.address,
          token: SIENNA.linkPair,
          schedule })
        await MGMT.upload(this.agent)
        await MGMT.instantiate(this.agent)
        report(MGMT.initTx.transactionHash)})

    await this.task('make mgmt owner of token', async (report: Function) => {
      const [tx1, tx2] = await MGMT.acquire(SIENNA)
      report(tx1.transactionHash)
      report(tx2.transactionHash) })

    const {RPT} = this.contracts
    await this.task('initialize rpt',
      async (report: Function) => {
      RPT.init.label = `${this.prefix}_${RPT.label}`
      Object.assign(RPT.init.msg, {
        token:   SIENNA.linkPair,
        mgmt:    MGMT.linkPair,
        portion: "2500000000000000000000", // TODO get this from schedule!!!
        config:  [[initialRPTRecipient, "2500000000000000000000"]] })
      await RPT.upload(this.agent)
      await RPT.instantiate(this.agent)
      report(RPT.initTx.transactionHash)})

    await this.task('point rpt account in mgmt schedule to rpt contract',
      async (report: Function) => {
        setRPTAddress(RPT.address)
        const {transactionHash} = await MGMT.configure(schedule)
        report(transactionHash) })

    console.log(table(
      [ [ 'Contract\nDescription',             'Address\nCode hash'                    ]
      , [ 'SiennaSNIP20\nSienna SNIP20 token', `${SIENNA.address}\n${SIENNA.codeHash}` ]
      , [ 'MGMT\nVesting',                     `${MGMT.address}\n${MGMT.codeHash}`     ]
      , [ 'RPT\nRemaining pool tokens',        `${RPT.address}\n${RPT.codeHash}`       ] ]))

    return {
      chain:     this.chain,
      agent:     this.agent,
      contracts: this.contracts } }

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
  contracts = { /*AMMFactory, AMMExchange, AMMSNIP20, LPSNIP20, IDO*/ }
  async initialize (_: EnsembleInit) { throw new Error('TODO!'); return {} } }

export class SiennaRewards extends ScrtEnsemble {

  contracts = rewardPools(this.agent, [
    'SIENNA',
    'SIENNA_sSCRT',
    'SITOK_STEST',
    'SIENNA_STEST',
    'SIENNA_SITORK',
    'SIENNA_sETH',
    'sSCRT_SITEST' ])

  localCommands = (): Commands => [...super.localCommands(),
    ["test",      Help.Rewards.TEST,      this.test.bind(this)     ],
    ["benchmark", Help.Rewards.BENCHMARK, this.benchmark.bind(this)]]

  remoteCommands = (): Commands => [
    ['deploy', Help.Rewards.DEPLOY, null, [
      ['all',  Help.Rewards.DEPLOY_ALL,  this.deployAll.bind(this)  ],
      ['this', Help.Rewards.DEPLOY_THIS, this.deployThis.bind(this) ],
      ...[].map((instance):Command=>
        [instance, Help.Rewards.ATTACH_TO, this.deployAttach.bind(this)])]]]

  private async parseOptions (options?: Record<string, any>) {
    if (!options) return
    if (options['agent'] === 'secretcli') this.agent = await ScrtCLIAgent.create(this.agent)
    if (options['premint.reward']) this.shouldPremintReward = true
    if (options['premint.admin'])  this.shouldPremintAdmin  = true }

  /** Deploy a single Sienna Rewards Pool + LP Token + an instance of the TGE.
    * Use the TGE's token as the reward token. */
  private async deployAll (context: any, schedule: any) {
    await this.parseOptions(context.options)
    const {chain} = context
    await chain.init()
    const TGE = await new SiennaTGE({chain}).deploy({...context, schedule})
    this.instances.SiennaSNIP20 = TGE.contracts.SiennaSNIP20
    await this.deploy()
    process.exit() }

  /** Deploy a single Sienna Rewards Pool + LP Token + Reward Token. */
  private async deployThis (context: any) {
    await this.parseOptions(context.options)
    Object.assign(this.contracts, { SiennaSNIP20 })
    await this.deploy()
    process.exit() }

  /** Deploy a single Sienna Rewards Pool + LP Token.
    * Use an existing SNIP20 token as the reward token. */
  private async deployAttach (context: any) {
    await this.parseOptions(context.options)
    await this.deploy()
    process.exit() }

  async initialize () {
    const deployed = [[ 'Contract\nDescription', 'Address\nCode hash' ]]
    const { chain,
            agent   = this.agent || await chain.getAgent(),
            task    = taskmaster(),
            uploads = await this.upload({agent, chain, task}) } = options
    // The reward token is pluggable: if rewardToken is not provided
    // (by deployAttach) a SNIP20 is automatically deployed
    if (this.contracts['SiennaSNIP20']) {
      deployed.push(await this.task(
        'initialize reward token',
        this.initToken.bind(this, uploads, 'SiennaSNIP20', agent))) }
    // reward token is pluggable - existing token can be passed to the deployment
    deployed.push(await this.task('initialize LP token',
      this.initToken.bind(this, uploads, 'LPSNIP20', agent)))
    deployed.push(await this.task(
      'initialize rewards',
      this.initRewards.bind(this, uploads, agent)))
    if (this.instances['SiennaSNIP20']) {
      if (this.shouldPremintAdmin || this.shouldPremintReward) {
        await this.task('allow admin to mint tokens',
          this.allowMintingByAdmin.bind(this, agent)) }
      this.shouldPremintAdmin && await this.task('mint test balance to admin account',
        this.premintAdmin.bind(this, agent))
      this.shouldPremintReward && await this.task('mint test balance to rewards contract',
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
    const {address, codeHash} = this.instances.RewardPool =
      await agent.instantiate(new RewardsContract({
        codeId: uploads.RewardPool.codeId,
        label: `${this.prefix}_${this.contracts.RewardPool.label}`,
        initMsg: { ...this.contracts.RewardPool.initMsg,
                   admin:        agent.address,
                   reward_token: this.instances.SiennaSNIP20.reference }}))
    report(this.instances.RewardPool.transactionHash)
    return ['RewardPool\nReward pool', `${address}\n${codeHash}`]}

  shouldPremintAdmin  = false
  shouldPremintReward = false
  private async allowMintingByAdmin (agent: Agent, report: Function) {
    const address = this.instances.RewardPool.address
        , result  = await this.instances.SiennaSNIP20.addMinters([address], agent)
    //console.log(decode(Buffer.from(Object.values(result.data) as any)))
    report(result.transactionHash)
    return result }
  private async premintAdmin (agent: Agent, report: Function) {
    const amount  = '540000000000000000000000'
        , address = agent.address
        , result  = await this.instances.SiennaSNIP20.mint(amount, agent, address)
    report(result.transactionHash)
    return result }
  private async premintReward (agent: Agent, report: Function) {
    const amount  = '540000000000000000000000'
        , address = this.instances.RewardPool.address
        , result  = await this.instances.SiennaSNIP20.mint(amount, agent, address)
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
  contracts = { SNIP20: { crate: 'snip20-lend' }
              , ATOKEN: { crate: 'atoken' }
              , CONFIG: { crate: 'configuration' } } }
