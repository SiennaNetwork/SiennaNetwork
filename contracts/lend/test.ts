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
import { b64encode } from "@waiting/base64";

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
  async function withGasReport(agent: Agent, contract: any, msg: any) {
    let op = Object.keys(msg)[0];
    let res = await agent.execute(contract, msg);
    gasTable.push({ op, gas_wanted: res.gas_wanted, gas_used: res.gas_used });
  }

  const ALICE = await chain.getAgent("ALICE");
  const BOB = await chain.getAgent("BOB");
  const MALLORY = await chain.getAgent("MALLORY");

  const INTEREST_MODEL = new InterestModelContract({ workspace });
  const ORACLE = new LendOracleContract({ workspace });
  const MARKET = new LendMarketContract({ workspace });
  const OVERSEER = new LendOverseerContract({ workspace });
  const MOCK_ORACLE = new MockOracleContract({ workspace });

  const TOKEN1 = new AMMSNIP20Contract({ workspace, name: "SLATOM" });
  const TOKEN2 = new AMMSNIP20Contract({ workspace, name: "SLSCRT" });

  await chain.buildAndUpload(agent, [TOKEN1, TOKEN2]);

  const gasTable = [];

  const deployedInterestModel = await deployment.get(INTEREST_MODEL.name);
  const deployedOverseer = await deployment.get(OVERSEER.name);
  const deployedMockOracle = await deployment.get(MOCK_ORACLE.name);

  // set prices
  await agent.execute(deployedMockOracle, {
    set_price: {
      symbol: "SLATOM",
      price: "1",
    },
  });
  await agent.execute(deployedMockOracle, {
    set_price: {
      symbol: "SLSCRT",
      price: "1",
    },
  });
  const token1 = await deployment.get(TOKEN1.name, "SLATOM");

  const token2 = await deployment.get(TOKEN2.name, "SLSCRT");

  console.info("minting tokens...");
  await withGasReport(agent, token1, {
    mint: { recipient: BOB.address, amount: "100" },
  });
  await withGasReport(agent, token1, {
    mint: { recipient: MALLORY.address, amount: "100" },
  });
  await withGasReport(agent, token2, {
    mint: { recipient: ALICE.address, amount: "300" },
  });

  console.info("listing markets...");
  await withGasReport(agent, deployedOverseer, {
    whitelist: {
      config: {
        config: {
          initial_exchange_rate: "0.2",
          reserve_factor: "1",
          seize_factor: "0.9",
        },
        entropy: randomHex(36),
        interest_model_contract: {
          address: deployedInterestModel.address,
          code_hash: deployedInterestModel.codeHash,
        },
        ltv_ratio: "0.7",
        prng_seed: randomHex(36),
        token_symbol: "SLATOM",
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
          initial_exchange_rate: "0.2",
          reserve_factor: "1",
          seize_factor: "0.9",
        },
        entropy: randomHex(36),
        interest_model_contract: {
          address: deployedInterestModel.address,
          code_hash: deployedInterestModel.codeHash,
        },
        ltv_ratio: "0.7",
        prng_seed: randomHex(36),
        token_symbol: "SLSCRT",
        underlying_asset: {
          address: token2.address,
          code_hash: token2.codeHash,
        },
      },
    },
  });

  let [market1, market2] = await agent.query(deployedOverseer, {
    markets: { pagination: { start: 0, limit: 10 } },
  });

  console.info("depositing...");
  await withGasReport(BOB, token1, {
    send: {
      recipient: market1.contract.address,
      recipient_code_hash: market1.contract.code_hash,
      amount: "100",
      msg: b64encode(JSON.stringify("deposit")),
    },
  });

  await withGasReport(ALICE, token2, {
    send: {
      recipient: market2.contract.address,
      recipient_code_hash: market2.contract.code_hash,
      amount: "300",
      msg: b64encode(JSON.stringify("deposit")),
    },
  });

  console.info("entering markets...");
  await withGasReport(BOB, deployedOverseer, {
    enter: {
      markets: [market1.contract.address, market2.contract.address],
    },
  });

  await withGasReport(ALICE, deployedOverseer, {
    enter: {
      markets: [market1.contract.address, market2.contract.address],
    },
  });

  await withGasReport(MALLORY, deployedOverseer, {
    enter: {
      markets: [market1.contract.address, market2.contract.address],
    },
  });

  console.info("borrowing...");
  await withGasReport(BOB, market2.contract, {
    borrow: {
      amount: "100",
    },
  });

  await withGasReport(MALLORY, market2.contract, {
    borrow: {
      amount: "100",
    },
  });

  console.info("repaying...");
  await withGasReport(BOB, token2, {
    send: {
      recipient: market2.contract.address,
      recipient_code_hash: market2.contract.code_hash,
      amount: "100",
      msg: b64encode(JSON.stringify({ repay: { borrower: null } })),
    },
  });
  console.table(gasTable, ["op", "gas_wanted", "gas_used"]);
}
