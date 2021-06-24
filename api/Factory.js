import { SecretNetworkContractWithSchema } from "@fadroma/scrt-agent";
import { loadSchemas } from "@fadroma/utilities";

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./factory/init_msg.json",
  queryMsg: "./factory/query_msg.json",
  queryAnswer: "./factory/query_response.json",
  handleMsg: "./factory/handle_msg.json",
});

export default class Factory extends SecretNetworkContractWithSchema {
  constructor(options) {
    super(options, schema);
  }
}
