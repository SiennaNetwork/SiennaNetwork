import { ContractWithSchema, loadSchemas } from "@hackbg/fadroma"

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./ido/init_msg.json",
  queryMsg: "./ido/query_msg.json",
  queryAnswer: "./ido/query_response.json",
  handleMsg: "./ido/handle_msg.json",
});

export default class IDO extends ContractWithSchema {
  constructor(options) {
    super(options, schema);
  }

  swap(amount, agent) {
    return this.tx.swap(
      { amount: `${amount}` },
      agent,
      undefined,
      JSON.stringify([{ amount: `${amount}`, denom: "uscrt" }])
    );
  }
}
