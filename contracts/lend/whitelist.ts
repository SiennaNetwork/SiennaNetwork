import { MigrationContext, randomHex } from "@hackbg/fadroma";

import { workspace } from "@sienna/settings";

import { LendOverseerContract, InterestModelContract } from "@sienna/api";

const MARKET_INITIAL_EXCHANGE_RATE = "0.2";
const MARKET_RESERVE_FACTOR = "1";
const MARKET_SEIZE_FACTOR = "0.9";
const MARKET_LTV_RATIO = "0.7";
const MARKET_TOKEN_SYMBOL = "SSCRT";

export async function whitelist({ agent, deployment }: MigrationContext) {
  const OVERSEER = new LendOverseerContract({ workspace });
  const INTEREST_MODEL = new InterestModelContract({ workspace });

  const overseerContract = deployment.get(OVERSEER.name);
  const interestModelContract = deployment.get(INTEREST_MODEL.name);

  await agent.execute(
    overseerContract,
    {
      whitelist: {
        config: {
          config: {
            initial_exchange_rate: MARKET_INITIAL_EXCHANGE_RATE,
            reserve_factor: MARKET_RESERVE_FACTOR,
            seize_factor: MARKET_SEIZE_FACTOR,
          },
          entropy: randomHex(36),
          interest_model_contract: interestModelContract,
          ltv_ratio: MARKET_LTV_RATIO,
          prng_seed: randomHex(36),
          token_symbol: MARKET_TOKEN_SYMBOL,
          underlying_asset: {
            address: "",
            code_hash: "",
          },
        },
      },
    },
    []
  );
}
