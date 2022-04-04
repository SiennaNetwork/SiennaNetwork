import { MigrationContext, Deployment, bold, randomHex } from "@hackbg/fadroma";

import {
  InterestModelContract,
  LendOracleContract,
  LendMarketContract,
  LendOverseerContract,
  MockOracleContract,
} from "@sienna/api";

const OVERSEER_CLOSE_FACTOR = "0.5";
const OVERSEER_PREMIUM = "1.08";

const INTEREST_MODEL_BASE_RATE = "0.2";
const INTEREST_MODEL_BLOCK_YEAR = 6311520;
const INTEREST_MODEL_JUMP_MULTIPLIER = "0";
const INTEREST_MODEL_JUMP_THRESHOLD = "0";
const INTEREST_MODEL_MULTIPLIER_YEAR = "0.9";

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
}> {
  const INTEREST_MODEL = new InterestModelContract();
  const ORACLE = new LendOracleContract();
  const MARKET = new LendMarketContract();
  const OVERSEER = new LendOverseerContract();
  const MOCK_ORACLE = new MockOracleContract();

  const isLocal = agent.chain.isLocalnet;
  const isTest = agent.chain.isTestnet;
  const isMain = agent.chain.isMainnet;

  for (const contract of [INTEREST_MODEL, ORACLE, MARKET, OVERSEER]) {
    await agent.buildAndUpload([contract]);
  }

  if (isLocal) {
    await agent.buildAndUpload([MOCK_ORACLE]);
    await deployment.instantiate(agent, [MOCK_ORACLE, {}]);
  }

  let mockOracle = isLocal ? deployment.get(MOCK_ORACLE.name) : null;

  const bandTest = {
    address: "secret1ulxxh6erkmk4p6cjehz58cqspw3qjuedrsxp8f",
    codeHash:
      "dc6ff596e1cd83b84a6ffbd857576d7693d89a826471d58e16349015e412a3d3",
  };

  // TODO: replace with mainnet Band oracle
  const bandMain = {
    address: "secret1ulxxh6erkmk4p6cjehz58cqspw3qjuedrsxp8f",
    codeHash:
      "dc6ff596e1cd83b84a6ffbd857576d7693d89a826471d58e16349015e412a3d3",
  };

  let oracleContract: any;

  if (isTest) {
    oracleContract = bandTest;
  } else if (isMain) {
    oracleContract = bandMain;
  }

  await deployment.instantiate(agent, [
    INTEREST_MODEL,
    {
      base_rate_year: INTEREST_MODEL_BASE_RATE,
      blocks_year: INTEREST_MODEL_BLOCK_YEAR,
      jump_multiplier_year: INTEREST_MODEL_JUMP_MULTIPLIER,
      jump_threshold: INTEREST_MODEL_JUMP_THRESHOLD,
      multiplier_year: INTEREST_MODEL_MULTIPLIER_YEAR,
    },
  ]);

  await deployment.instantiate(agent, [
    OVERSEER,
    {
      close_factor: OVERSEER_CLOSE_FACTOR,
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
        address: isLocal ? mockOracle.address : oracleContract.address,
        code_hash: isLocal ? mockOracle.codeHash : oracleContract.codeHash,
      },
      premium: OVERSEER_PREMIUM,
      prng_seed: randomHex(36),
    },
  ]);

  return {
    OVERSEER,
    MARKET,
    INTEREST_MODEL,
    ORACLE,
    MOCK_ORACLE,
  };
}
