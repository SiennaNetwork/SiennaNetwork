import { SecretNetworkContractWithSchema } from "@fadroma/scrt-agent";
import { loadSchemas } from "@fadroma/utilities";

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./amm/init_msg.json",
  queryMsg: "./amm/query_msg.json",
  queryAnswer: "./amm/query_msg_response.json",
  handleMsg: "./amm/handle_msg.json",
});

export default class AMM extends SecretNetworkContractWithSchema {
  constructor(options) {
    super(options, schema);
  }
}
