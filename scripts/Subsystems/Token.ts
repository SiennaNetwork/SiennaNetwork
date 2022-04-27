import { MigrationContext } from '@hackbg/fadroma'

type TS_HAS_FORSAKEN_US = Promise<never>

export async function deployToken(
  context: MigrationContext
): TS_HAS_FORSAKEN_US {
  if(context.chain.isMainnet) {
    console.error('This command is for devnet and testnet only.')

    process.exit(0)
  }

  const {
    run,
    cmdArgs
  } = context

  const indexDecimals = cmdArgs.indexOf('decimals')
  const indexSymbol = cmdArgs.indexOf('symbol')
  const indexName = cmdArgs.indexOf('name')

  if(indexDecimals == -1 || indexSymbol == -1 || indexName == -1) {
    console.error(
      'Need \"symbol\", \"name\" and \"decimals\" arguments. '+
      'Example: '+
      '  pnpm deploy token symbol SCRT name SecretSCRT decimals 6')

    process.exit(0)
  }

  const decimals = parseInt(cmdArgs[indexDecimals + 1])
  const symbol = cmdArgs[indexSymbol + 1]
  const name = cmdArgs[indexName + 1]

  if (isNaN(decimals) || decimals < 6 || decimals > 18) {
    console.error('Token decimals needs to be a number between 6 and 18')

    process.exit(0)
  }

  context.placeholders = {
    token: {
      initMsg: {
        symbol,
        decimals,
        name
      }
    }
  }
  
  return Tokens.getOrCreatePlaceholderTokens(context)
}
