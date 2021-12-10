import { ScrtContract, loadSchemas, Agent } from "@fadroma/scrt";
import { abs } from "../ops/index";

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./exchange/init_msg.json",
  queryMsg: "./exchange/query_msg.json",
  handleMsg: "./exchange/handle_msg.json",
});

// @ts-ignore
const decoder = new TextDecoder();
const decode = (buffer: any) => decoder.decode(buffer).trim();

export class Exchange extends ScrtContract {
  constructor ({ prefix, admin, label, codeId, initMsg }: {
    prefix?:  string,
    admin?:   Agent,
    label?:   string,
    codeId?:  number
    initMsg?: any
  } = {}) {
    super({ agent: admin, schema })
    if (codeId)  this.blob.codeId = codeId
    if (prefix)  this.init.prefix = prefix
    if (label)   this.init.label  = label
    if (initMsg) this.init.msg    = initMsg
  }

  code = { ...this.code, workspace: abs(), crate: "exchange" };
}
