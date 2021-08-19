import { execFileSync } from 'child_process'

import { taskmaster } from '@fadroma/cli'
import { Ensemble } from '@fadroma/ensemble'
import { randomBytes } from '@fadroma/util-sys'

import { SNIP20Contract, RewardsContract } from '@sienna/api'
import { abs } from './lib/index.js'

const prefix = `${new Date().toISOString()}`

const prng_seed = () => randomBytes(36).toString('hex')

export default class SiennaRewards extends Ensemble {

  workspace = abs()

  // TODO: make tokens "pluggable"
  // i.e. allow attaching the rewards to an existing deployment
  contracts = { TOKEN_LP, TOKEN_REWARD, REWARD_POOL, AMM_FACTORY, AMM_EXCHANGE }

  localCommands = [
    ...(console.log('lc',this,super.localCommands), []),
    ["test",      '🥒 Run unit tests',    this.test.bind(this)      ],
    ["benchmark", '⛽ Measure gas costs', this.benchmark.bind(this) ]]

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
    execFileSync(abs('node_modules/.bin/mocha'), args, { stdio: 'inherit' }) }

  async initialize ({ network, receipts, agent = network.agent }) {
    //throw new Error('todo!')
    const instances: Record<string, any> = {}
    const task = taskmaster()
    await task('initialize token',   initTokenTask)
    await task('initialize rewards', initRewardsTask)
    await task('mint some rewards',  mintRewardsTask)
    console.log(instances)
    console.table([
      ['Contract\nDescription',      'Address\nCode hash'],
      ['TOKEN\nSienna SNIP20 token', `${instances.TOKEN.address}\n${instances.TOKEN.codeHash}`],
      ['Rewards\n',                  `${instances.REWARDS.address}\n${instances.REWARDS.codeHash}`]])
    return instances

    async function initTokenTask (report: Function) {
      const {codeId} = receipts.TOKEN
      const {label, initMsg} = this.contracts.TOKEN
      Object.assign(initMsg, { admin: agent.address })
      instances.TOKEN = await agent.instantiate(new SNIP20Contract({codeId, label, initMsg}))
      report(instances.TOKEN.transactionHash) }

    async function initRewardsTask (report: Function) {
      const {codeId} = receipts.REWARDS
      const {label} = this.contracts.REWARDS
      const initMsg = {
        ...this.contracts.REWARDS.initMsg,
        admin:     agent.address,
        entropy:   randomBytes(36).toString('base64'),
        prng_seed: randomBytes(36).toString('base64'),
        reward_token: instances.TOKEN.reference }
      console.log(agent.address)
      instances.REWARDS = await agent.instantiate(new RewardsContract({codeId, label, initMsg}))
      report(instances.REWARDS.transactionHash) }

    async function mintRewardsTask (report: Function) {
      const amount = '540000000000000000000000'
      const result = await instances.TOKEN.mint(amount, agent, instances.REWARDS.address)
      report(result) } }

  /** Attach reward pool to existing deployment. */
  async augment () {
    throw new Error('todo!') } }

const TOKEN_LP = {
  crate: 'lp-token',
  label: `lp_token`,
  initMsg: {
    prng_seed,
    name:     "LPToken",
    symbol:   "LP",
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
    prng_seed,
    name:     "RewardToken",
    symbol:   "REWARD",
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
  initMsg: {}}

const AMM_FACTORY = {
  crate: 'factory',
  label: `amm_factory`,
  initMsg: {}}

const AMM_EXCHANGE = {
  crate: 'exchange',
  label: `amm_exchange`,
  initMsg: {}}
