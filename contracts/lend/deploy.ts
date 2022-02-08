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
} from "@hackbg/fadroma";

import {
  InterestModelContract,
  LendOracleContract,
  LendMarketContract,
  LendOverseerContract,
  MockOracleContract,
} from "@sienna/api";

import settings, { workspace } from "@sienna/settings";

const console = Console("@sienna/amm/upgrade");

export async function deployLend({ agent, deployment, prefix }: MigrationContext): Promise<{
  OVERSEER: LendOverseerContract;
  INTEREST_MODEL: InterestModelContract;
}> {

  const { isLocalnet } = agent.chain
  let mock_oracle = { address: null, codeHash: null };

  const INTEREST_MODEL = new InterestModelContract();
  const ORACLE         = new LendOracleContract();
  const MARKET         = new LendMarketContract();
  const OVERSEER       = new LendOverseerContract();
  const MOCK_ORACLE    = new MockOracleContract();

  for (const contract of [INTEREST_MODEL, ORACLE, MARKET, OVERSEER, MOCK_ORACLE]) {
    await agent.buildAndUpload([contract]);
  }

  // paramters taken from Compound
  await deployment.getOrInit(agent, INTEREST_MODEL, INTEREST_MODEL.label, {
    base_rate_year: "20000000000000000",
    blocks_year: 6311520,
    jump_multiplier_year: "200000000000000000",
    jump_threshold: "900000000000000000",
    multiplier_year: "200000000000000000",
  });

  if (isLocalnet) {
    await deployment.instantiate(agent, [MOCK_ORACLE, {}]);
  }

  let overseer = await deployment.getOrInit(agent, OVERSEER, OVERSEER.label, {
    close_factor: "500000000000000000",
    entropy: randomHex(36),
    market_contract: {
      code_hash: MARKET.template.codeHash,
      id: MARKET.template.codeId,
    },
    oracle_contract: {
      code_hash: MARKET.template.codeHash,
      id: MARKET.template.codeId,
    },
    // TODO: add band oracle address and hash
    oracle_source: {
      address: isLocalnet ? mock_oracle.address : "",
      code_hash: isLocalnet ? mock_oracle.codeHash : "",
    },
    premium: "1080000000000000000",
    prng_seed: randomHex(36),
  });

  return {
    OVERSEER,
    INTEREST_MODEL,
  };
}
