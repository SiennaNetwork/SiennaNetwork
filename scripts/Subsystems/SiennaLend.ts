import { MigrationContext, randomHex, buildAndUploadMany } from '@hackbg/fadroma'
import * as API from '@sienna/api'
import { versions, contracts, sources } from '../Build'
import { templateStruct } from '../misc'

export interface LendInterestModelOptions {
  base_rate_year:       string
  blocks_year:          number
  jump_multiplier_year: string
  jump_threshold:       string
  multiplier_year:      string
}

export interface LendOverseerOptions {
  entropy:      string
  prng_seed:    string
  close_factor: string
  premium:      string
}

export interface LendContracts {
  OVERSEER:       API.LendOverseerClient
  INTEREST_MODEL: API.InterestModelClient
  MARKET?:        API.LendMarketClient
  ORACLE?:        API.LendOracleClient
  MOCK_ORACLE?:   API.MockOracleClient
  TOKEN1?:        API.AMMSnip20Client
  TOKEN2?:        API.AMMSnip20Client
}

export type LendDeployContext =
  MigrationContext & LendInterestModelOptions & LendOverseerOptions

export async function deployLend (context: LendDeployContext): Promise<LendContracts> {

  // 1. Expand dependencies and settings from context
  const { ref = versions.HEAD
        , src = sources(ref, contracts.Lend)
        , builder
        , uploader
        , templates = await buildAndUploadMany(builder, uploader, src)
        , deployAgent, deployment, prefix
        , agent

        // Interest model settings:
        , base_rate_year       =      "0"
        , blocks_year          = 6311520
        , jump_multiplier_year =      "0"
        , jump_threshold       =      "0"
        , multiplier_year      =      "1"

        // Overseer settings:
        , entropy      =  randomHex(36)
        , prng_seed    =  randomHex(36)
        , close_factor =  "0.5"
        , premium      =  "1"
        } = context

  const { isDevnet } = agent.chain

  const [
    interestModelTemplate,
    oracleTemplate,
    marketTemplate,
    overseerTemplate,
    mockOracleTemplate,
    tokenTemplate,
  ] = templates

  // Define names for deployed contracts
  const v = 'v1'
  const names = {
    interestModel: `Lend[${v}].InterestModel`,
    oracle:        `Lend[${v}].Oracle`,
    mockOracle:    `Lend[${v}].MockOracle`,
    overseer:      `Lend[${v}].Overseer`,
    token1:        `Lend[${v}].Placeholder.slATOM`,
    token2:        `Lend[${v}].Placeholder.slSCRT`
  }

  // Deploy placeholder tokens
  const tokenConfig = {
    enable_burn: true,
    enable_deposit: true,
    enable_mint: true,
    enable_redeem: true,
    public_total_supply: true,
  }
  const token1 = await deployment.init(
    deployAgent, tokenTemplate, names.token1, {
      name:     "slToken1",
      symbol:   "SLATOM",
      decimals:  18,
      prng_seed: randomHex(36),
      config:    tokenConfig,
    })
  const token2 = await deployment.init(
    deployAgent, tokenTemplate, names.token2, {
      name:     "slToken2",
      symbol:   "SLSCRT",
      decimals:  18,
      prng_seed: randomHex(36),
      config:    tokenConfig,
    })

  // Create the interest model
  await deployment.init(
    deployAgent, interestModelTemplate, names.interestModel, {
      base_rate_year,
      blocks_year,
      jump_multiplier_year,
      jump_threshold,
      multiplier_year
    })

  // Create the mock oracle
  const mockOracle = await deployment.init(
    deployAgent, mockOracleTemplate, names.mockOracle, {})

  // Create the overseer
  await deployment.init(
    deployAgent, overseerTemplate, names.overseer, {
      entropy, prng_seed, close_factor, premium,
      market_contract: templateStruct(marketTemplate),
      oracle_contract: templateStruct(oracleTemplate),
      oracle_source:   templateStruct(mockOracle)
    })

  // Return clients to the instantiated contracts

  const client = (Class, name) => new Class({...deployment.get(name), agent})
  return {
    OVERSEER:       client(API.LendOverseerClient,  names.overseer),
    INTEREST_MODEL: client(API.InterestModelClient, names.interestModel),
    // TODO: get oracle by querying overseer (once this query exists)
    // ORACLE:         new API.LendOracleClient({
    //   ...deployment.get(names.oracle),        agent
    // }),
    MOCK_ORACLE:    client(API.MockOracleClient, names.mockOracle),
    TOKEN1:         client(API.AMMSnip20Client,  names.token1),
    TOKEN2:         client(API.AMMSnip20Client,  names.token2),
  }

}
