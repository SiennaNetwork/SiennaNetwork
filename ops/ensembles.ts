import { ScrtEnsemble, ScrtCLIAgent,
         Commands, Command, Console, render, table,
         execFileSync, timestamp, JSONDirectory, randomHex } from '@hackbg/fadroma'
import { abs, genConfig, getDefaultSchedule, ONE_SIENNA } from './index'
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
    console.log(SIENNA.linkPair, SIENNA)
    await this.task('Initialize MGMT (TGE vesting contract)',
      async (report: Function) => {
        console.log(this.schedule)
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

export class SiennaRewards extends ScrtEnsemble {

  TGE = new SiennaTGE({chain: this.chain})

  pairs = {'SIENNA':       500,
           'SIENNA-sSCRT': 400,
           'SITOK-STEST':  500,
           'SIENNA-STEST': 300,
           'SIENNA-SITOK': 300,
           'SIENNA-sETH':  200,
           'sSCRT-SITEST': 300}

  contracts = rewardPools(this.agent, Object.keys(this.pairs))

  shouldPremintAdmin  = false
  shouldPremintReward = false

  localCommands = (): Commands => [...super.localCommands(),
    ["test",      Help.Rewards.TEST,      this.test.bind(this)     ],
    ["benchmark", Help.Rewards.BENCHMARK, this.benchmark.bind(this)]]

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
    return (dir.has('SiennaSNIP20') &&
            dir.has('SiennaMGMT')   &&
            dir.has('SiennaRPT')) }

  /** Deploy a single Sienna Rewards Pool + LP Token + an instance of the TGE.
    * Use the TGE's token as the reward token. */
  private async deployAll (context: any) {
    await this.parseOptions(context.options)
    await this.TGE.deploy()
    await this.deploy()
    process.exit() }

  /** Deploy a single Sienna Rewards Pool + LP Token.
    * Use an existing SNIP20 token as the reward token. */
  private async deployAttach (context: any) {
    console.log(context)
    await this.parseOptions(context.options)
    await this.deploy()
    process.exit() }

  private async parseOptions (options?: Record<string, any>) {
    if (!options) return
    if (options['agent'] === 'secretcli') this.agent = await ScrtCLIAgent.create(this.agent)
    if (options['premint.reward']) this.shouldPremintReward = true
    if (options['premint.admin'])  this.shouldPremintAdmin  = true }

  /** Deploys reward pairs (reward pool + LP token), as well as a reward pool for staking SIENNA.
    * Configures the RPT contract to route funds to the correct reward pools.
    * Can also premint SIENNA for testing. */
  async initialize () {
    await super.initialize()
    const SIENNA    = this?.TGE?.contracts?.SIENNA
        , RPT       = this?.TGE?.contracts?.RPT
        , deployed  = [[ 'Contract\nDescription', 'Address\nCode hash' ]]
        , rptConfig = []
    if (!SIENNA || !RPT) throw new Error("Unable to find SIENNA or RPT contract.")
    if (this.shouldPremintAdmin||this.shouldPremintReward) await this.premint()
    for (const [pair, amount] of Object.entries(this.pairs)) {
      const token = (pair === 'SIENNA') ? this.TGE.contracts.SIENNA :
        await this.task(`Initialize a liquidity provision token for ${pair}`, async (report: Function) => {
          const token = this.contracts[`LP_${pair}`]
          token.init.msg.admin = this.agent.address
          await token.instantiate(this.agent)
          report(token.initReceipt.transactionHash)
          deployed.push([`${pair}\nLP Token`, `${token.address}\n${token.codeHash}`])
          return token })
      await this.task(`Initialize a reward pool for ${pair}`, async (report: Function) => {
        const rewardPool = this.contracts[`RP_${pair}`]
        rewardPool.init.msg.admin        = this.agent.address
        rewardPool.init.msg.reward_token = SIENNA.link
        rewardPool.init.msg.lp_token     = token.link
        await rewardPool.instantiate(this.agent)
        report(rewardPool.initReceipt.transactionHash)
        deployed.push([`${pair}\nReward pool`, `${rewardPool.address}\n${rewardPool.codeHash}`])
        rptConfig.push([rewardPool.address, String(BigInt(amount) * ONE_SIENNA)]) }) }
    await this.task(`Configure RPT to route funds to reward pools`, async (report: Function) => {
      const result = RPT.configure(rptConfig)
      report(result.transactionHash) })
    console.log(table(deployed))
    return {chain:     this.chain,
            agent:     this.agent,
            contracts: this.contracts} }

  async premint () {
    const agent = this.agent
    const SIENNA = this.TGE.contracts.SIENNA as SiennaSNIP20
    await this.task('allow admin to mint reward tokens',
      async (report: Function) => {
        const result = await SIENNA.addMinters(
          [agent.address], agent)
        //console.log(decode(Buffer.from(Object.values(result.data) as any)))
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
    execFileSync(abs('node_modules/.bin/mocha'), args, { stdio: 'inherit' }) } }

export class SiennaSwap extends ScrtEnsemble {
  prefix: string = `${timestamp()}`
  workspace = abs()
  contracts = {
    FACTORY:  new AMMFactory(this.agent),
    EXCHANGE: new AMMExchange(this.agent),
    AMMTOKEN: new AMMSNIP20(this.agent),
    LPTOKEN:  new LPToken(this.agent, `${this.prefix}_LPToken`),
    IDO:      new IDO(this.agent)
  }

  //sienna_burner: string

  async initialize () {
    super.initialize()

    this.agent = await this.chain.getAgent()

    const instance = await this.task('instanitate AMM factory', async (report: Function) => {
      const {
        FACTORY,
        EXCHANGE,
        AMMTOKEN,
        LPTOKEN,
        IDO
      } = this.contracts;
  
      const initMsg = {
        snip20_contract: { code_hash: AMMTOKEN.codeHash, id: AMMTOKEN.codeId },
        pair_contract: { code_hash: EXCHANGE.codeHash, id: EXCHANGE.codeId },
        lp_token_contract: { code_hash: LPTOKEN.codeHash, id: LPTOKEN.codeId },
        ido_contract: { code_hash: IDO.codeHash, id: IDO.codeId },
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
        prng_seed: randomHex(36)
      }

      const result = await this.agent.instantiate(FACTORY.codeId, FACTORY.label, initMsg)
      report(result.transactionHash)

      return result
    })

    return {
      instance
    }
  }
}

export class SiennaLend extends ScrtEnsemble {
  contracts = {/* SNIP20: { crate: 'snip20-lend' }
              , ATOKEN: { crate: 'atoken' }
              , CONFIG: { crate: 'configuration' } */} }
