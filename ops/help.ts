const Help = {
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
      ADD_ACCOUNT: '⚡ Add a new account to a partial vesting pool'*/
  },
  Rewards: {
    TEST:        '🥒 Run unit tests',
    BENCHMARK:   '⛽ Measure gas costs',
    DEPLOY:      '🚀 Deploy TGE + Rewards, or attach Rewards to existing TGE',
    DEPLOY_ALL:  '🚀 Deploy new TGE + Rewards (needs TGE schedule)',
    DEPLOY_THIS: '🚀 Deploy just the Rewards',
    ATTACH_TO:   '🚀 Deploy Rewards attached to this TGE',
  }
}

export default Help
