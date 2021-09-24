import type { Chain, Agent } from '@fadroma/ops'
import { Scrt } from '@fadroma/scrt'
import { bold, symlinkDir } from '@fadroma/tools'
import process from 'process'
import { fileURLToPath } from 'url'

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(()=>process.exit(0))
}

export default async function main ([chainName, ...commands]: Array<string>) {
  const chains: Record<string, Function> = {
    'localnet-1.0': Scrt.localnet_1_0,
    'localnet-1.2': Scrt.localnet_1_2,
    'holodeck-2':   Scrt.holodeck_2,
    'supernova-1':  Scrt.supernova_1,
    'secret-2':     Scrt.secret_2,
    'secret-3':     Scrt.secret_3 }
  if (!chainName) {
    console.log(`Select target chain:`)
    for (const chain of Object.keys(chains)) console.log(`  ${bold(chain)}`)
    process.exit(1) }
  const chain = await chains[chainName]().init()
      , admin = await chain.getAgent()
  console.log(`Operating on ${bold(chainName)} as ${bold(admin.address)}`)
  const originalCommands = [...commands]
  let fragment: string|undefined
  let command: Record<string, any>|Function = {
    deploy: {
      vesting () {
        return deployVesting({ chain, admin }) },
      swap () {
        const MGMT = getSelectedVesting()
        return deploySwap({ chain, admin, MGMT }) } },
    select: {
      vesting (id?: string) {
        if (id) return selectVesting(chain, id)
        printVestingInstances(chain) },
      swap (id?: string) {
        if (id) return selectSwap(chain, id)
        printSwapInstances(chain) } } }
  while (true) {
    if (command instanceof Function) {
      return command(...commands) }
    else {
      fragment = commands.shift()
      if (fragment) {
        command = command[fragment] }
      else {
        console.log(`Available commands at this level are:`)
        for (const subcmd of Object.keys(command)) console.log(`  ${bold(subcmd)}`)
        process.exit(1) } } } }

/// ------------------------------------------------------------------------------------------------

import type { ScheduleFor_HumanAddr } from '@sienna/api/mgmt/handle'
import { SiennaSNIP20, MGMTContract, RPTContract } from '@sienna/api'
export type VestingOptions = {
  chain?:    Chain,
  admin?:    Agent,
  schedule?: ScheduleFor_HumanAddr,
}
export async function deployVesting (options: VestingOptions = {}): Promise<SwapOptions> {
  const { chain = await new Scrt().ready,
          admin = await chain.getAgent(),
          schedule } = options
  const SIENNA = new SiennaSNIP20(admin)
      , MGMT   = new MGMTContract(admin, schedule, SIENNA)
      , RPT    = new RPTContract(admin, MGMT)
  await Promise.all([SIENNA, MGMT, RPT].map(contract=>contract.build()))
  await Promise.all([SIENNA, MGMT, RPT].map(contract=>contract.upload()))
  await SIENNA.instantiate()
  await MGMT.instantiate()
  await RPT.instantiate()
  await MGMT.configure(replaceRPTAddress(schedule, RPT))
  await MGMT.launch()
  await RPT.vest()
  return { chain, admin, MGMT }
}
export function replaceRPTAddress (schedule: ScheduleFor_HumanAddr, RPT: RPT) {
  return schedule
}

export function getSelectedVesting (chain: Chain) {}
export function selectVesting (chain: Chain, id: string) {}
export function printVestingInstances (chain: Chain) {}

/// ------------------------------------------------------------------------------------------------

import { FactoryContract, AMMContract, AMMSNIP20, LPToken, RewardsContract, IDOContract } from '@sienna/api'
export type SwapOptions = {
  chain?: Chain,
  admin?: Agent,
  MGMT?:  MGMTContract,
}
export async function deploySwap (options: SwapOptions = {}) {
  const { chain = await new Scrt().ready,
          admin = await chain.getAgent(),
          MGMT,
          swapConfig = loadSwapConfig() } = options
  const EXCHANGE = new AMMContract({ admin })
      , AMMTOKEN = new AMMSNIP20({ admin })
      , LPTOKEN  = new LPToken({ admin })
      , IDO      = new IDOContract({ admin })
      , FACTORY  = new FactoryContract({ admin, swapConfig, EXCHANGE, AMMTOKEN, LPTOKEN, IDO })
      , REWARDS  = new RewardsContract({ admin })
  await Promise.all([EXCHANGE, AMMTOKEN, LPTOKEN, IDO, FACTORY, REWARDS].map(contract=>contract.build()))
  await Promise.all([EXCHANGE, AMMTOKEN, LPTOKEN, IDO, FACTORY, REWARDS].map(contract=>contract.upload()))
  await FACTORY.instantiate()
}

export async function loadSwapConfig () {}

export async function addRewardPool () {}

export async function replaceRewardPool () {}

export function getSelectedSwap (chain: Chain) {}
export function selectSwap (chain: Chain, id: string) {}
export function printSwapInstances (chain: Chain) {}
