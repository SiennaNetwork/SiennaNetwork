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
    ['build',  Help.TGE.BUILD, (_: any, sequential: boolean) =>
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
    ['init',   Help.TGE.INIT,   (_: any) =>
      this.initialize()],
    ['launch', Help.TGE.LAUNCH, (_: any, address: any) =>
      this.launch({...context, address})],
    ['claim',  Help.TGE.CLAIM,  (_: any, address: any, claimant: any) =>
      this.claim({...context, address, claimant})],
    ['status', Help.TGE.STATUS, (_: any, address: any) =>
      this.getStatus({...context, address})] ]

  async initialize () {
    await super.initialize()

    // Deploy SIENNA token /////////////////////////////////////////////////////////////////////////
    // mgmt could automatically instantiate token and rpt if it supported callbacks
    const {SIENNA} = this.contracts
    await this.task('initialize token',
      async (report: Function) => {
        SIENNA.init.label = `${this.prefix}_${SIENNA.label}`
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
    console.log(SIENNA.linkPair, SIENNA)
    await this.task('initialize mgmt',
      async (report: Function) => {
        console.log(this.schedule)
        RPTAccount.address = initialRPTRecipient
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
        RPT.init.label = `${this.prefix}_${RPT.label}`
        Object.assign(RPT.init.msg, {
          token:   SIENNA.linkPair,
          mgmt:    MGMT.linkPair,
          portion: RPTAccount.portion_size, // TODO get this from schedule!!!
          config:  [[initialRPTRecipient, RPTAccount.portion_size]] })
        await RPT.instantiate(this.agent)
        report(RPT.initTx.transactionHash)})
    await this.task('point rpt account in mgmt schedule to rpt contract',
      async (report: Function) => {
        RPTAccount.address = RPT.address
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

export class SiennaRewards extends SiennaTGE {

  pairs = {'SIENNA':       500,
           'SIENNA-sSCRT': 400,
           'SITOK-STEST':  500,
           'SIENNA-STEST': 300,
           'SIENNA-SITOK': 300,
           'SIENNA-sETH':  200,
           'sSCRT-SITEST': 300}

  shouldPremintAdmin  = false
  shouldPremintReward = false

  constructor (...args: Array<any>) {
    super(...args)
    Object.assign(this.contracts, rewardPools(this.agent, Object.keys(this.pairs))) }

  localCommands = (): Commands => [...super.localCommands(),
    ["test",      Help.Rewards.TEST,      this.test.bind(this)     ],
    ["benchmark", Help.Rewards.BENCHMARK, this.benchmark.bind(this)]]

  remoteCommands = (): Commands => {console.log(this.chain.instances.list());return [
    ['deploy', Help.Rewards.DEPLOY, null, [
      ['all',  Help.Rewards.DEPLOY_ALL,  this.deployAll.bind(this)  ],
      ['this', Help.Rewards.DEPLOY_THIS, this.deployThis.bind(this) ],
      ...[].map((instance):Command=>[instance, Help.Rewards.ATTACH_TO, this.deployAttach.bind(this)])]]]}

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
    this.contracts.SiennaSNIP20 = TGE.contracts.SiennaSNIP20
    await this.deploy()
    process.exit() }

  /** Deploy a single Sienna Rewards Pool + LP Token + Reward Token. */
  private async deployThis (context: any) {
    await this.parseOptions(context.options)
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

    if (this.contracts.SIENNA) {
      if (this.shouldPremintAdmin || this.shouldPremintReward) {
        await this.task('allow admin to mint tokens',
          async (report: Function) => {
            const result = await (this.contracts.SIENNA as SiennaSNIP20).addMinters([this.agent.address], this.agent)
            //console.log(decode(Buffer.from(Object.values(result.data) as any)))
            report(result.transactionHash)
            return result }) }
      if (this.shouldPremintAdmin) {
        await this.task('mint test balance to admin account',
          async (report: Function) => {
            const amount  = '540000000000000000000000'
                , address = this.agent.address
                , result  = await this.contracts.SIENNA.mint(amount, this.agent, address)
            report(result.transactionHash)
            return result }) } }

    const deployed = [[ 'Contract\nDescription', 'Address\nCode hash' ]]

    // reward token is pluggable - existing token can be passed to the deployment
    for (const [pair, amount] of Object.entries(this.pairs)) {

      const token = (pair === 'SIENNA') ? this.contracts.SIENNA :
        await this.task(`initialize LP token for ${pair}`, async (report: Function) => {
          const token = this.contracts[`LP_${pair}`]
          token.init.label = `${this.prefix}_${token.init.label}`
          token.init.msg.admin = this.agent.address
          await token.instantiate(this.agent)
          report(token.initReceipt.transactionHash)
          deployed.push([`${pair}\nLP Token`, `${token.address}\n${token.codeHash}`])
          return token })

      await this.task(`initialize reward pool for ${pair}`, async (report: Function) => {
        const rewardPool = this.contracts[`RP_${pair}`]
        rewardPool.init.label = `${this.prefix}_${rewardPool.init.label}`
        rewardPool.init.msg.admin = this.agent.address
        rewardPool.init.msg.reward_token = this.contracts.SIENNA.link
        rewardPool.init.msg.lp_token = token.link
        await rewardPool.instantiate(this.agent)
        report(rewardPool.initReceipt.transactionHash)
        deployed.push([`${pair}\nReward pool`, `${rewardPool.address}\n${rewardPool.codeHash}`]) })

      if (this.shouldPremintReward) {
        await this.task(`mint test balance to reward pool for ${pair}`,
          async (report: Function) => {
            const amount  = '540000000000000000000000'
                , address = this.contracts[`rp${pair}`].address
                , result  = await this.contracts.SIENNA.mint(amount, this.agent, address)
            report(result.transactionHash)
            return result }) } }

    console.log(table(deployed))

    return {
      chain:     this.chain,
      agent:     this.agent,
      contracts: this.contracts } }

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

export class SiennaLend extends ScrtEnsemble {
  contracts = {/* SNIP20: { crate: 'snip20-lend' }
              , ATOKEN: { crate: 'atoken' }
              , CONFIG: { crate: 'configuration' } */} }
