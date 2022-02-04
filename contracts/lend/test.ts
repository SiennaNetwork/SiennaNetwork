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

import { Snip20 } from "../../frontends/siennajs/lib/snip20";

import {
  InterestModelContract,
  LendOracleContract,
  LendMarketContract,
  LendOverseerContract,
  MockOracleContract,
  AMMSNIP20Contract,
} from "@sienna/api";

import settings, { workspace } from "@sienna/settings";

export async function testLend({
  chain,
  agent,
  deployment,
  prefix,
}: MigrationContext) {
  // console.log(chain.identities.load("ALICE"))
  async function withGasReport(agent: Agent, contract: any, msg: any) {
    let op = Object.keys(msg)[0];
    let res = await agent.execute(contract, msg);
    gasTable.push({ op, gas_wanted: res.gas_wanted, gas_used: res.gas_used });
  }

  const ALICE = await chain.getAgent("ALICE");
  const BOB = await chain.getAgent("BOB");

  const INTEREST_MODEL = new InterestModelContract({ workspace });
  const ORACLE = new LendOracleContract({ workspace });
  const MARKET = new LendMarketContract({ workspace });
  const OVERSEER = new LendOverseerContract({ workspace });
  const MOCK_ORACLE = new MockOracleContract({ workspace });

  const TOKEN1 = new AMMSNIP20Contract({ workspace, name: "SLATOM" });
  const TOKEN2 = new AMMSNIP20Contract({ workspace, name: "SLSCRT" });

  await chain.buildAndUpload(agent, [TOKEN1, TOKEN2]);

  const gasTable = [];

  const deployedInterestModel = await deployment.getOrInit(
    agent,
    INTEREST_MODEL,
    INTEREST_MODEL.label
  );
  const deployedOracle = await deployment.getOrInit(
    agent,
    ORACLE,
    ORACLE.label
  );
  const deployedMarket = await deployment.getOrInit(
    agent,
    MARKET,
    MARKET.label
  );
  const deployedOverseer = await deployment.getOrInit(
    agent,
    OVERSEER,
    OVERSEER.label
  );
  const deployedMockOracle = await deployment.getOrInit(
    agent,
    MOCK_ORACLE,
    MOCK_ORACLE.label
  );

  const token1 = await deployment.getOrInit(agent, TOKEN1, "SLATOM", {
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

  const token2 = await deployment.getOrInit(agent, TOKEN2, "SLSCRT", {
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

  await withGasReport(agent, token1, {
    mint: { recipient: BOB.address, amount: "100" },
  });
  await withGasReport(agent, token2, {
    mint: { recipient: ALICE.address, amount: "100" },
  });

  await withGasReport(agent, deployedOverseer, {
    whitelist: {
      config: {
        config: {
          initial_exchange_rate: "1",
          reserve_factor: "1",
          seize_factor: "0.9",
        },
        entropy: randomHex(36),
        interest_model_contract: {
          address: deployedInterestModel.address,
          code_hash: deployedInterestModel.codeHash,
        },
        ltv_ratio: "0.9",
        prng_seed: randomHex(36),
        token_symbol: "SLTOKEN1",
        underlying_asset: {
          address: token1.address,
          code_hash: token1.codeHash,
        },
      },
    },
  });

  await withGasReport(agent, deployedOverseer, {
    whitelist: {
      config: {
        config: {
          initial_exchange_rate: "1",
          reserve_factor: "1",
          seize_factor: "0.9",
        },
        entropy: randomHex(36),
        interest_model_contract: {
          address: deployedInterestModel.address,
          code_hash: deployedInterestModel.codeHash,
        },
        ltv_ratio: "0.9",
        prng_seed: randomHex(36),
        token_symbol: "SLTOKEN1",
        underlying_asset: {
          address: token2.address,
          code_hash: token2.codeHash,
        },
      },
    },
  });

  let markets = await agent.query(deployedOverseer, {markets: {start: 0, limit: 10}})

  console.table(gasTable, ["op", "gas_wanted", "gas_used"]);
}
