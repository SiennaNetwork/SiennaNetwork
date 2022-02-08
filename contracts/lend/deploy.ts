import {
  MigrationContext,
  print,
  Deployment,
  Chain,
  Agent,
  bold,
  Console,
  randomHex,
  timestamp,
  Template,
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
  agent,
  deployment,
  prefix,
}: MigrationContext): Promise<{
  OVERSEER: LendOverseerContract;
  MARKET: LendMarketContract;
  INTEREST_MODEL: InterestModelContract;
  ORACLE: LendOracleContract;
  MOCK_ORACLE: MockOracleContract;
  TOKEN1: AMMSNIP20Contract;
  TOKEN2: AMMSNIP20Contract;
}> {
  const INTEREST_MODEL = new InterestModelContract();
  const ORACLE = new LendOracleContract();
  const MARKET = new LendMarketContract();
  const OVERSEER = new LendOverseerContract();
  const MOCK_ORACLE = new MockOracleContract();
  const TOKEN1 = new AMMSNIP20Contract({ name: "SLATOM" });
  const TOKEN2 = new AMMSNIP20Contract({ name: "SLSCRT" });

  for (const contract of [
    INTEREST_MODEL,
    ORACLE,
    MARKET,
    OVERSEER,
    MOCK_ORACLE,
    TOKEN1,
    TOKEN2,
  ]) {
    await agent.chain.buildAndUpload(agent, [contract]);
  }

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
      code_hash: MARKET.template.codeHash,
      id: Number(MARKET.template.codeId),
    },
    oracle_contract: {
      code_hash: ORACLE.template.codeHash,
      id: Number(ORACLE.template.codeId),
    },
    oracle_source: {
      address: mock_oracle.address,
      code_hash: mock_oracle.codeHash,
    },
    premium: "1",
    prng_seed: randomHex(36),
  });

  return {
    OVERSEER,
    MARKET,
    INTEREST_MODEL,
    ORACLE,
    MOCK_ORACLE,
    TOKEN1,
    TOKEN2,
  };
}
