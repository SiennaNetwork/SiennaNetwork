import { Contract, ScrtEnsemble, EnsembleInit, ScrtCLIAgent, Agent,
         Commands, Command, Console, render, taskmaster, table,
         readFile, execFileSync, timestamp, decode } from '@hackbg/fadroma'
import { abs, genConfig, getDefaultSchedule } from './index'
import { runDemo } from './tge.demo.js'
import { EnsemblesHelp as Help } from './help'
import { SiennaSNIP20, MGMT as MGMTContract, RPT as RPTContract,
         AMMFactory, AMMExchange, AMMSNIP20,
         LPToken, RewardPool, rewardPools,
         IDO } from './contracts'

const { debug, warn, info } = Console(import.meta.url)

type TGECommandArgs = { address?: string, chain?: any }

export class SiennaTGE extends ScrtEnsemble {

  contracts = {
    SIENNA: new SiennaSNIP20(this.agent),
    MGMT:   new MGMTContract(this.agent),
    RPT:    new RPTContract(this.agent)}

  schedule = getDefaultSchedule()

  localCommands = (): Commands => [

    ['build',  Help.TGE.BUILD,  (_: any, sequential: boolean) =>
      this.build(!sequential)],

    ['config', Help.TGE.CONFIG, (_: any, spreadsheet: any) =>
      genConfig(spreadsheet)]]

  remoteCommands = (): Commands => [

    ['deploy', Help.TGE.DEPLOY, async (_: any) => {
      await this.deploy()
      process.exit(0) }],

    ['demo',   Help.TGE.DEMO,
      runDemo],

    ['upload', Help.TGE.UPLOAD, (_: any) =>
      this.upload()],

    ['init',   Help.TGE.INIT,   (_: any) => {
      return this.initialize() }],

    ['launch', Help.TGE.LAUNCH, (context: any, address: any) =>
      this.launch({...context, address})],

    ['claim',  Help.TGE.CLAIM,  (context: any, address: any, claimant: any) =>
      this.claim({...context, address, claimant})],

    ['status', Help.TGE.STATUS, (context: any, address: any) =>
      this.getStatus({...context, address})] ]

  async initialize () {
    await super.initialize()

    // Deploy SIENNA token /////////////////////////////////////////////////////////////////////////
    // mgmt could automatically instantiate token and rpt if it supported callbacks
    const {SIENNA} = this.contracts
    await this.task('initialize token',
      async (report: Function) => {
        await SIENNA.upload(this.agent)
        SIENNA.init.label = `${this.prefix}_${SIENNA.label}`
        Object.assign(SIENNA.init.msg, { admin: this.agent.address })
        await SIENNA.instantiate(this.agent)
        report(SIENNA.initTx.transactionHash) })

    // Deploy MGMT vesting contract ////////////////////////////////////////////////////////////////
    // use placeholder RPT address in schedule
    // updated after RPT is deployed
    const initialRPTRecipient = this.agent.address
    const setRPTAddress = (address: string) => {
      this.schedule.pools.filter((x:any)=>x.name==='MintingPool')[0]
                   .accounts.filter((x:any)=>x.name==='RPT')[0]
                   .address = address }
    const {MGMT} = this.contracts
    console.log(SIENNA.linkPair, SIENNA)
    await this.task('initialize mgmt',
      async (report: Function) => {
        console.log(this.schedule)
        setRPTAddress(initialRPTRecipient)
        await MGMT.upload(this.agent)
        MGMT.init.label = `${this.prefix}_${MGMT.init.label}`
        Object.assign(MGMT.init.msg, {
          admin:    this.agent.address,
          token:    SIENNA.linkPair,
          schedule: this.schedule })
        await MGMT.instantiate(this.agent)
        report(MGMT.initTx.transactionHash)})
    await this.task('make mgmt owner of token', async (report: Function) => {
      const [tx1, tx2] = await MGMT.acquire(SIENNA)
      report(tx1.transactionHash)
      report(tx2.transactionHash) })

    // Deploy RPT splitter contract ////////////////////////////////////////////////////////////////
    const {RPT} = this.contracts
    await this.task('initialize rpt',
      async (report: Function) => {
        await RPT.upload(this.agent)
        RPT.init.label = `${this.prefix}_${RPT.label}`
        Object.assign(RPT.init.msg, {
          token:   SIENNA.linkPair,
          mgmt:    MGMT.linkPair,
          portion: "2500000000000000000000", // TODO get this from schedule!!!
          config:  [[initialRPTRecipient, "2500000000000000000000"]] })
        await RPT.instantiate(this.agent)
        report(RPT.initTx.transactionHash)})
    await this.task('point rpt account in mgmt schedule to rpt contract',
      async (report: Function) => {
        setRPTAddress(RPT.address)
        const {transactionHash} = await MGMT.configure(this.schedule)
        report(transactionHash) })

    // And we're done //////////////////////////////////////////////////////////////////////////////
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
  contracts = {
    FACTORY:  new AMMFactory(this.agent),
    EXCHANGE: new AMMExchange(this.agent),
    AMMTOKEN: new AMMSNIP20(this.agent),
    LPTOKEN:  new LPToken(this.agent, `${this.prefix}_LPToken`),
    IDO:      new IDO(this.agent) }
  async initialize () { throw new Error('TODO!'); return {} } }

export class SiennaRewards extends ScrtEnsemble {

  pairs = [
    'SIENNA',
    'SIENNA_sSCRT',
    'SITOK_STEST',
    'SIENNA_STEST',
    'SIENNA_SITORK',
    'SIENNA_sETH',
    'sSCRT_SITEST']

  contracts = {
    SIENNA: new SiennaSNIP20(this.agent),
    ...rewardPools(this.agent, this.pairs)
  }

  localCommands = (): Commands => [...super.localCommands(),

    ["test",      Help.Rewards.TEST,      this.test.bind(this)     ],

    ["benchmark", Help.Rewards.BENCHMARK, this.benchmark.bind(this)]]

  remoteCommands = (): Commands => [

    ['deploy', Help.Rewards.DEPLOY, null, [

      ['all',  Help.Rewards.DEPLOY_ALL,  this.deployAll.bind(this)  ],

      ['this', Help.Rewards.DEPLOY_THIS, this.deployThis.bind(this) ],

      ...[].map((instance):Command =>
        [instance, Help.Rewards.ATTACH_TO, this.deployAttach.bind(this)])]]]

  private async parseOptions (options?: Record<string, any>) {
    if (!options) return
    if (options['agent'] === 'secretcli') this.agent = await ScrtCLIAgent.create(this.agent)
    if (options['premint.reward']) this.shouldPremintReward = true
    if (options['premint.admin'])  this.shouldPremintAdmin  = true }

  /** Deploy a single Sienna Rewards Pool + LP Token + an instance of the TGE.
    * Use the TGE's token as the reward token. */
  private async deployAll (context: any) {
    await this.parseOptions(context.options)
    const {chain} = context
    await chain.init()
    const TGE = await new SiennaTGE({chain}).deploy()
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
    await super.initialize()

    const deployed = [[ 'Contract\nDescription', 'Address\nCode hash' ]]

    // The reward token is pluggable: if rewardToken is not provided
    // (by deployAttach) a SNIP20 is automatically deployed
    if (this.contracts['SiennaSNIP20']) {
      deployed.push(await this.task('initialize reward token',
        this.initToken.bind(this, 'SiennaSNIP20'))) }

    // reward token is pluggable - existing token can be passed to the deployment
    for (const pair of this.pairs) {
      deployed.push(await this.task(`initialize LP token for ${pair}`,
        this.initToken.bind(this, pair)))
      deployed.push(await this.task(`initialize reward pool for ${pair}`,
        this.initRewards.bind(this, pair)))
      if (this.shouldPremintReward) {
        await this.task(`mint test balance to reward pool for ${pair}`,
          this.premintReward.bind(this)) } }

    if (this.instances['SiennaSNIP20']) {
      if (this.shouldPremintAdmin || this.shouldPremintReward) {
        await this.task('allow admin to mint tokens',
          this.allowMintingByAdmin.bind(this)) }
      if (this.shouldPremintAdmin) {
        await this.task('mint test balance to admin account',
          this.premintAdmin.bind(this)) } }

    console.log(table(deployed))

    return this.instances }

  private async initToken (pair: string, report: Function) {
    const LPTOKEN = this.contracts[`lp${pair}`]
    await LPTOKEN.upload(this.agent)
    LPTOKEN.init.label = `${this.prefix}_${LPTOKEN.init.label}`
    LPTOKEN.init.msg.admin = this.agent.address
    await LPTOKEN.init(this.agent)
    report(LPTOKEN.initReceipt.transactionHash)
    return [`${pair}\nLP Token`, `${LPTOKEN.address}\n${LPTOKEN.codeHash}`] }

  private async initRewards (pair: string, report: Function) {
    const REWARDS = this.contracts[`rp${pair}`]
    await REWARDS.upload(this.agent)
    REWARDS.init.label            = `${this.prefix}_${REWARDS.init.label}`
    REWARDS.init.msg.admin        = this.agent.address
    REWARDS.init.msg.reward_token = this.contracts.SIENNA.link
    REWARDS.init.msg.lp_token     = this.contracts[`lp${pair}`].link
    await REWARDS.init(this.agent)
    report(REWARDS.initReceipt.transactionHash)
    return [`${pair}\nReward pool`, `${REWARDS.address}\n${REWARDS.codeHash}`] }

  shouldPremintAdmin  = false
  shouldPremintReward = false
  private async allowMintingByAdmin (report: Function) {
    const result = await (this.instances.SIENNA as SiennaSNIP20).addMinters([this.agent.address], this.agent)
    //console.log(decode(Buffer.from(Object.values(result.data) as any)))
    report(result.transactionHash)
    return result }
  private async premintAdmin (report: Function) {
    const amount  = '540000000000000000000000'
        , address = this.agent.address
        , result  = await this.contracts.SIENNA.mint(amount, this.agent, address)
    report(result.transactionHash)
    return result }
  private async premintReward (pair: string, report: Function) {
    const amount  = '540000000000000000000000'
        , address = this.contracts[`rp${pair}`].address
        , result  = await this.contracts.SIENNA.mint(amount, this.agent, address)
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
  contracts = {/* SNIP20: { crate: 'snip20-lend' }
              , ATOKEN: { crate: 'atoken' }
              , CONFIG: { crate: 'configuration' } */} }
