export const CLIHelp = {
  USAGE:    "❓ Print usage info",
  STATUS:   "Show stored receipts from uploads and instantiations.",

  TGE:      "🚀 SIENNA token + vesting",
  SWAP:     "💱 Contracts of Sienna Swap/SWAP",
  REWARDS:  "🏆 SIENNA token + staking rewards",
  LEND:     "🏦 Contracts of Sienna Lend",

  DOCS:     "📖 Build the documentation and open it in a browser.",
  TEST:     "⚗️  Run test suites for all the individual components.",
  COVERAGE: "📔 Generate test coverage and open it in a browser.",
  SCHEMA:   "🤙 Regenerate JSON schema for each contract's API.",

  BUILD:         "👷 Compile contracts from source",
  BUILD_ALL:     "all contracts in workspace",
  BUILD_TGE:     "snip20-sienna, mgmt, rpt",
  BUILD_REWARDS: "snip20-sienna, rewards",
  BUILD_SWAP:    "amm-snip20, factory, exchange, lp-token",
  BUILD_LEND:    "snip20-lend + lend-atoken + configuration",

  SHELL:  "🐚 Launch a JavaScript REPL for talking to contracts directly"}

export const EnsemblesHelp = {
  TGE: {
    BUILD:       '👷 Compile contracts from working tree',
    CONFIG:      '📅 Convert a spreadsheet into a JSON schedule',
    DEPLOY:      '🚀 Build, init, and deploy the TGE',
    DEMO:        '🐒 Run the TGE demo (long-running integration test)',
    UPLOAD:      '📦 Upload compiled contracts to chain',
    INIT:        '🚀 Init new instances of already uploaded contracts',
    LAUNCH:      '🚀 Launch deployed vesting contract',
    CLAIM:       '⚡ Claim funds from a deployed contract',
    STATUS:      '👀 Print the status and schedule of a contract.',
    /*TRANSFER:    '⚡ Transfer ownership of contracts to another address',
      CONFIGURE:   '⚡ Upload a new JSON config to an already initialized contract',
      REALLOCATE:  '⚡ Update the allocations of the RPT tokens',
      ADD_ACCOUNT: '⚡ Add a new account to a partial vesting pool'*/ },
  Rewards: {
    TEST:        '🥒 Run unit tests',
    BENCHMARK:   '⛽ Measure gas costs',
    DEPLOY:      '🚀 Deploy TGE + Rewards, or attach Rewards to existing TGE',
    DEPLOY_ALL:  '🚀 Deploy new Sienna TGE + Rewards (needs TGE schedule)',
    DEPLOY_THIS: '🚀 Deploy just Sienna Rewards',
    ATTACH_TO:   '🚀 Add Sienna Rewards to this TGE deployment', } }
