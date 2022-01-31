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
} from "@sienna/api"

import settings, { workspace } from "@sienna/settings"

const console = Console("@sienna/amm/upgrade");

export async function deployLend({
    chain, admin, deployment, prefix,
}: MigrationContext): Promise<{workspace: string}> {
    console.info(bold("Admin balance: "), await admin.balance)

    const [ INTEREST_MODEL, ORACLE, MARKET, OVERSEER ] = await chain.buildAndUpload(admin, [
        new InterestModelContract({workspace}),
        new LendOracleContract({workspace}),
        new LendMarketContract({workspace}),
        new LendOverseerContract({workspace})
    ])

    return {
        workspace,
    }
}