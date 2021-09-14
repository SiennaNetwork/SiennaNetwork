import { argv } from 'process'
import main from './index'

try {
  process.on('unhandledRejection', handleError)
  main(argv[2], ...argv.slice(3)) }
catch (e) {
  handleError(e) }

function handleError (e: Error) {
  console.error(e)
  const ISSUES = `https://github.com/SiennaNetwork/sienna/issues`
  console.info(`🦋 That was a bug! Report it at ${ISSUES}`)
  process.exit(1) }
