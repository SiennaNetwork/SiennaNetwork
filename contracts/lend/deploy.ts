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
} from "@sienna/api";

import settings, { workspace } from "@sienna/settings";
import { random } from "../../frontends/dashboard/helpers";

const console = Console("@sienna/amm/upgrade");
import { InitMsg as MarketInitMsg } from "./market/schema/init_msg";

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
}> {
  let mock_oracle = { address: null, codeHash: null };
  let [INTEREST_MODEL, ORACLE, MARKET, OVERSEER, MOCK_ORACLE] =
    await chain.buildAndUpload(agent, [
      new InterestModelContract({ workspace }),
      new LendOracleContract({ workspace }),
      new LendMarketContract({ workspace }),
      new LendOverseerContract({ workspace }),
      new MockOracleContract({ workspace }),
    ]);

  // paramters taken from Compound
  await deployment.getOrInit(agent, INTEREST_MODEL, INTEREST_MODEL.label, {
    base_rate_year: "20000000000000000",
    blocks_year: 6311520,
    jump_multiplier_year: "200000000000000000",
    jump_threshold: "900000000000000000",
    multiplier_year: "200000000000000000",
  });

  if (chain.isLocalnet) {
    let contract = await deployment.init(agent, MOCK_ORACLE, {});

    mock_oracle.address = contract.initTx.contractAddress;
    mock_oracle.codeHash = contract.codeHash;
  }

  await deployment.getOrInit(agent, OVERSEER, OVERSEER.label, {
    close_factor: "500000000000000000",
    entropy: randomHex(36),
    market_contract: {
      code_hash: MARKET.codeHash,
      id: MARKET.codeId,
    },
    oracle_contract: {
      code_hash: MARKET.codeHash,
      id: MARKET.codeId,
    },
    // TODO: add band oracle address and hash
    oracle_source: {
      address: chain.isLocalnet ? mock_oracle.address : "",
      code_hash: chain.isLocalnet ? mock_oracle.codeHash : "",
    },
    premium: "1080000000000000000",
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
  };
}
