import { ContractWithSchema, loadSchemas } from "@hackbg/fadroma"

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./amm/init_msg.json",
  queryMsg:    "./amm/query_msg.json",
  queryAnswer: "./amm/query_msg_response.json",
  handleMsg:   "./amm/handle_msg.json",
});

export default class AMM extends ContractWithSchema {
  constructor(options) {
    super(options, schema);
  }
}
