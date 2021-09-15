import { ScrtContract, loadSchemas, Agent } from "@fadroma/scrt"
import { TokenTypeFor_HumanAddr } from './factory/handle_msg.d'
import { randomBase64 } from '@fadroma/tools'
import { EnigmaUtils } from 'secretjs'
import { b64encode } from '@waiting/base64'
import { abs } from '../ops/index'
import { randomHex } from '@fadroma/tools'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./factory/init_msg.json",
  queryMsg:    "./factory/query_msg.json",
  queryAnswer: "./factory/query_response.json",
  handleMsg:   "./factory/handle_msg.json", })

export class Factory extends ScrtContract {
  code = { ...super.code, workspace: abs(), crate: 'factory' }
  init = { ...super.init, label: 'SiennaAMMFactory', msg: {
    get prng_seed () { return randomHex(36) },
    exchange_settings: { swap_fee:   { nom: 28, denom: 1000 }
                       , sienna_fee: { nom: 2, denom: 10000 }
                       , sienna_burner: null } } }
  constructor (agent: Agent) {
    super(schema, agent) }
  createExchange = (
    token_0: any,
    token_1: any,
    agent = this.instantiator
  ) => this.execute('create_exchange', {
    pair: { token_0: { custom_token: token_0 }
          , token_1: { custom_token: token_1 } },
    entropy: b64encode(EnigmaUtils.GenerateNewSeed().toString()) }, '', [], undefined, agent)
  /*this.tx.createExchange({
    pair:    { token_0, token_1 },
    entropy: b64encode(EnigmaUtils.GenerateNewSeed().toString()) })*/
  listExchanges = () =>
    this.q.listExchanges({pagination:{start:0,limit:100}}) }
