import { MigrationContext, Instance, buildAndUploadMany, randomHex } from '@hackbg/fadroma'
import * as API from '@sienna/api'
import getSettings from '@sienna/settings'
import { versions, contracts, source, sources } from '../Build'
import { linkStruct } from '../misc'
import { Schedule } from './SiennaTGE'

type VestingKind = 'tge' | 'vesting'

export interface VestingDeployContext extends MigrationContext {

  /** Which kind of vesting to deploy **/
  version:   VestingKind
  /** Address of the admin. */
  admin:     string
  /** The schedule for the new MGMT.
    * Defaults to production schedule. */
  settings?: { schedule?: Schedule }

  MGMTClient?:       API.MGMTClient
  RPTClient?:        API.RPTClient
  RewardsClient?:    API.RewardsClient

  tokens?:           unknown[]
  tokenClients?:     API.Snip20Client[]

  mgmtConfigs?:      unknown[]
  mgmtInstances?:    Instance[]
  mgmtClients?:      (typeof this.MGMTClient)[]

  rewardsConfigs?:   unknown[]
  rewardsInstances?: Instance[]
  rewardsClients?:   (typeof this.RewardsClient)[]

  rptConfigs?:       unknown[]
  rptInstances?:     Instance[]
  rptClients?:       (typeof this.RPTClient)[]

}

export interface VestingDeployResult {
  mgmtClients:    API.MGMTClient[],
  rptClients:     API.RPTClient[],
  tokenClients:   API.Snip20Client[],
  rewardsClients: API.RewardsClient[]
}

export async function deployVesting (context: VestingDeployContext): Promise<VestingDeployResult> {

  const {
    deployment,
    prefix,
    agent,
    agent: { chain: { isMainnet, isTestnet, isDevnet } },

    builder,
    uploader,

    templates: [
      mgmtTemplate,
      rptTemplate,
      rewardsTemplate,
      tokenTemplate
    ] = await buildAndUploadMany(
      builder,
      uploader,
      // build vesting contracts from working tree
      sources(versions.HEAD,       contracts.Vesting),
      // build rewards contract from release branch
      sources(versions.Rewards.v3, contracts.Rewards),
      // build a standard token contract for testing
      isMainnet ? [] : [source('amm-snip20')],
    ),

    MGMTClient    = API.MGMTClient['vested'],
    RPTClient     = API.RPTClient['vested'],
    RewardsClient = API.RewardsClient['v3'],

    admin = agent.address,
    settings: {
      schedule,
      vesting
    } = getSettings(agent.chain.mode),

    tokens           = isDevnet ? await initMockTokens(deployment, agent, tokenTemplate, vesting) : [],
    tokenClients     = tokens.map(instance => agent.getClient(API.Snip20Client, instance)),

    mgmtConfigs      = generateMgmtConfigs(vesting, admin, tokens),
    mgmtInstances    = await deployment.initMany(agent, mgmtTemplate, mgmtConfigs),
    mgmtClients      = mgmtInstances.map(instance => agent.getClient(MGMTClient, instance)),

    rewardsConfigs   = generateRewardsConfigs(admin, vesting, tokens),
    rewardsInstances = await deployment.initMany(agent, rewardsTemplate, rewardsConfigs),
    rewardsClients   = rewardsInstances.map(instance => agent.getClient(RewardsClient, instance)),

    rptConfigs       = generateRptConfigs(mgmtInstances, admin, vesting, rewardsInstances, tokens),
    rptInstances     = await deployment.initMany( agent, rptTemplate, rptConfigs),
    rptClients       = rptInstances.map(instance => agent.getClient(RPTClient, instance)),

  } = context

  await agent.bundle().wrap(async bundle => {
    const mgmtBundleClients = mgmtInstances.map(instance => bundle.getClient(MGMTClient, instance))
    await Promise.all(vesting.map(async ({ schedule, account }, i) => {
      account.address = rptInstances[i].address
      await mgmtBundleClients[i].add(schedule.pools[0].name, account)
    }))
  })

  return { mgmtClients, rptClients, tokenClients, rewardsClients }

}

async function initMockTokens (deployment, agent, tokenTemplate, vesting) {
  const mockTokenConfig = {
    decimals: 18,
    config: {
      public_total_supply: true,
      enable_deposit: true,
    },
    initial_balances: [{
      address: agent.address,
      amount: "9999999999999"
    }]
  }
  return await deployment.initMany(
    agent,
    tokenTemplate,
    vesting.map(({ name }) => [
      name, {
        ...mockTokenConfig,
        name:  `Mock_${name}`,
        symbol: name.toUpperCase(),
        prng_seed: randomHex(36),
      }
    ])
  )
}

export function generateMgmtConfigs (vesting, admin, tokens) {
  return vesting.map(({name, schedule, rewards, lp}, i) => [
    `${rewards.name}.MGMT[v3]`.replace(/\s/g, ''), {
      admin,
      schedule,
      prefund: true,
      token: linkStruct(tokens[i] || rewards)
    }
  ])
}

export function generateRptConfigs (mgmts, admin, vesting, pools, tokens) {
  return vesting.map(({ name, schedule, rewards, lp, account }, i) => [
    `${rewards.name}.RPT[v2]`.replace(/\s/g, ''), {
      mgmt:    linkStruct(mgmts[i]),
      token:   linkStruct(tokens[i] || rewards),
      portion: account.portion_size,
      distribution: [[pools[i].address, account.portion_size]],
    }
  ])
}

export function generateRewardsConfigs (admin, vesting, tokens) {
  return vesting.map(({name, schedule, rewards, lp}, i ) => [
    `${rewards.name}-${lp.name}.Rewards[v3]`.replace(/\s/g, ''), {
      admin,
      config: {
        lp_token:     linkStruct(tokens[i] || lp),
        reward_token: linkStruct(tokens[i] || rewards),
        ...(rewards.timekeeper && { timekeeper: rewards.timekeeper })
      }
    }
  ])
}
