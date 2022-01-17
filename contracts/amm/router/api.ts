import { ScrtContract_1_2, loadSchemas, Agent } from "@fadroma/scrt";
import { workspace } from "@sienna/settings";

export const schema = loadSchemas(import.meta.url, {
  initMsg: "./router/init_msg.json",
  queryMsg: "./router/query_msg.json",
  handleMsg: "./router/handle_msg.json",
});

// @ts-ignore
const decoder = new TextDecoder();
const decode = (buffer: any) => decoder.decode(buffer).trim();

export class SwapRouterContract extends ScrtContract_1_2 {
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

  code = { ...this.code, workspace, crate: "router" };
}
