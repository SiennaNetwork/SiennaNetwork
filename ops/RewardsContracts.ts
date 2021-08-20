import { execFileSync } from 'child_process'
import { taskmaster, table } from '@fadroma/cli'
import { Ensemble, InitArgs } from '@fadroma/ensemble'
import { randomHex } from '@fadroma/util-sys'
import { SNIP20Contract, RewardsContract } from '@sienna/api'
import { abs } from './lib/index.js'
import { TGEContracts } from './TGEContracts.js'

const prefix = `${new Date().toISOString()}`

const prng_seed = () => randomHex(36)

const TOKEN_LP = {
  crate: 'lp-token',
  label: `lp_token`,
  initMsg: {
    get prng_seed () { return randomHex(36) },
    name: "Liquidity Provision Token",
    symbol: "LPTOKE",
    decimals: 18,
    config: {
      public_total_supply: true,
      enable_deposit:      true,
      enable_redeem:       true,
      enable_mint:         true,
      enable_burn:         true } } }

const TOKEN_REWARD = {
  crate: 'amm-snip20',
  label: `reward_token`,
  initMsg: {
    get prng_seed () { return randomHex(36) },
    name: "RewardToken",
    symbol: "REWARD",
    decimals: 18,
    config: {
      public_total_supply: true,
      enable_deposit:      true,
      enable_redeem:       true,
      enable_mint:         true,
      enable_burn:         true } } }

const REWARD_POOL = {
  crate: 'sienna-rewards',
  label: `reward_pool`,
  initMsg: {
    get entropy     () { return randomHex(36) },
    get prng_seed   () { return randomHex(36) },
    get viewing_key () { return randomHex(36) } } }

const AMM_FACTORY = {
  crate: 'factory',
  label: `amm_factory`,
  initMsg: {} }

const AMM_EXCHANGE = {
  crate: 'exchange',
  label: `amm_exchange`,
  initMsg: {} }

export default class SiennaRewards extends Ensemble {

  workspace = abs()

  // TODO: make tokens "pluggable"
  // i.e. allow attaching the rewards to an existing deployment
  contracts = {
    TOKEN_LP, TOKEN_REWARD, REWARD_POOL, AMM_FACTORY, AMM_EXCHANGE }

  async initialize (args: InitArgs) {
    const { network, receipts, agent = network.agent } = args
    //throw new Error('todo!')
    const instances: Record<string, any> = {}
    const task = taskmaster()
    await task('initialize LP token',     initTokenTask.bind(this, 'TOKEN_LP'))
    await task('initialize reward token', initTokenTask.bind(this, 'TOKEN_REWARD'))
    await task('initialize rewards',      initRewardsTask.bind(this))
    await task('mint some rewards',       mintRewardsTask.bind(this))
    console.log(instances)
    console.log(table([
      [ 'Contract\nDescription',
        'Address\nCode hash' ],
      [ 'TOKEN_LP\nLiquidity provision',
        `${instances.TOKEN_REWARD.address}\n${instances.TOKEN_REWARD.codeHash}`],
      [ 'TOKEN_REWARD\nSienna SNIP20 token',
        `${instances.TOKEN_REWARD.address}\n${instances.TOKEN_REWARD.codeHash}`],
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
      report(instances.TOKEN_LP.transactionHash) }

    async function initRewardsTask (report: Function) {
      const {codeId} = receipts.REWARD_POOL
          , {label}  = this.contracts.REWARD_POOL
          , initMsg  = { ...this.contracts.REWARD_POOL.initMsg
                       , admin:        agent.address
                       , reward_token: instances.TOKEN_REWARD.reference }
      instances.REWARD_POOL = await agent.instantiate(
        new RewardsContract({codeId, label: `${this.prefix}_${label}`, initMsg}))
      report(instances.REWARD_POOL.transactionHash) }

    async function mintRewardsTask (report: Function) {
      const amount = '540000000000000000000000'
      const result = await instances.TOKEN_REWARD.mint(amount, agent, instances.REWARD_POOL.address)
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
