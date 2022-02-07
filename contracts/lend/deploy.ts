import {
  MigrationContext,
  printContracts,
  Deployment,
  Chain,
  Agent,
  bold,
  Console,
  randomHex,
  timestamp,
} from "@hackbg/fadroma";

import {
  InterestModelContract,
  LendOracleContract,
  LendMarketContract,
  LendOverseerContract,
  MockOracleContract,
  AMMSNIP20Contract,
} from "@sienna/api";

import { workspace } from "@sienna/settings";

export async function deployLend({
  chain,
  agent,
  deployment,
  prefix,
}: MigrationContext): Promise<{
  workspace: string;
  deployment: Deployment;
  prefix: string;
  OVERSEER: LendOverseerContract;
  MARKET: LendMarketContract;
  INTEREST_MODEL: InterestModelContract;
  ORACLE: LendOracleContract;
  MOCK_ORACLE: MockOracleContract;
  TOKEN1: AMMSNIP20Contract;
  TOKEN2: AMMSNIP20Contract;
}> {
  let [INTEREST_MODEL, ORACLE, MARKET, OVERSEER, MOCK_ORACLE, TOKEN1, TOKEN2] =
    await chain.buildAndUpload(agent, [
      new InterestModelContract({ workspace }),
      new LendOracleContract({ workspace }),
      new LendMarketContract({ workspace }),
      new LendOverseerContract({ workspace }),
      new MockOracleContract({ workspace }),
      new AMMSNIP20Contract({ workspace, name: "SLATOM" }),
      new AMMSNIP20Contract({ workspace, name: "SLSCRT" }),
    ]);

  await deployment.getOrInit(agent, TOKEN1, "SLATOM", {
    name: "slToken1",
    symbol: "SLATOM",
    decimals: 18,
    prng_seed: randomHex(36),
    config: {
      enable_burn: true,
      enable_deposit: true,
      enable_mint: true,
      enable_redeem: true,
      public_total_supply: true,
    },
  });

  await deployment.getOrInit(agent, TOKEN2, "SLSCRT", {
    name: "slToken2",
    symbol: "SLSCRT",
    decimals: 18,
    prng_seed: randomHex(36),
    config: {
      enable_burn: true,
      enable_deposit: true,
      enable_mint: true,
      enable_redeem: true,
      public_total_supply: true,
    },
  });

  await deployment.getOrInit(agent, INTEREST_MODEL, INTEREST_MODEL.label, {
    base_rate_year: "0",
    blocks_year: 6311520,
    jump_multiplier_year: "0",
    jump_threshold: "0",
    multiplier_year: "1",
  });

  let mock_oracle = await deployment.getOrInit(
    agent,
    MOCK_ORACLE,
    MOCK_ORACLE.label,
    {}
  );

  await deployment.getOrInit(agent, OVERSEER, OVERSEER.label, {
    close_factor: "0.5",
    entropy: randomHex(36),
    market_contract: {
      code_hash: MARKET.codeHash,
      id: MARKET.codeId,
    },
    oracle_contract: {
      code_hash: ORACLE.codeHash,
      id: ORACLE.codeId,
    },
    oracle_source: {
      address: mock_oracle.address,
      code_hash: mock_oracle.codeHash,
    },
    premium: "1",
    prng_seed: randomHex(36),
  });

  return {
    workspace,
    deployment,
    prefix,
    OVERSEER,
    MARKET,
    INTEREST_MODEL,
    ORACLE,
    MOCK_ORACLE,
    TOKEN1,
    TOKEN2,
  };
}
