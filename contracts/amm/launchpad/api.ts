import { ScrtContract_1_2, loadSchemas, Agent } from "@fadroma/scrt";
import { randomHex } from "@fadroma/tools";
import { workspace } from "@sienna/settings";

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./schema/init_msg.json",
  queryMsg:    "./schema/query_msg.json",
  queryAnswer: "./schema/query_response.json",
  handleMsg:   "./schema/handle_msg.json",
});

// @ts-ignore
const decoder = new TextDecoder();
const decode = (buffer: any) => decoder.decode(buffer).trim();

export class LaunchpadContract extends ScrtContract_1_2 {

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

  code = { ...this.code, workspace, crate: "launchpad" };

  /**
   * This action will remove the token from the contract
   * and will refund all locked balances on that token back to users
   *
   * @param {number} amount
   * @param {Agent} [agent]
   * @returns
   */
  async adminRemoveToken(index: number, agent?: Agent) {
    return this.tx.admin_remove_token({ index }, agent);
  }

  /**
   * This method will perform the native token lock.
   *
   * NOTE: For any other token, use snip20 receiver interface
   *
   * @param {string|number|bigint} amount
   * @param {string} [denom]
   * @param {Agent} [agent]
   * @returns
   */
  async lock(
    amount: string | number | bigint,
    denom: string = "uscrt",
    agent?: Agent
  ) {
    return this.tx.lock({ amount: `${amount}` }, agent, undefined, [
      { amount: `${amount}`, denom },
    ]);
  }

  /**
   * This method will perform the native token unlock
   *
   * NOTE: For any other token, use snip20 receiver interface
   *
   * @param {string|number|bigint} entries
   * @param {Agent} [agent]
   * @returns
   */
  async unlock(entries: string | number | bigint, agent?: Agent) {
    return this.tx.unlock({ entries }, agent);
  }

  /**
   * Get the configuration information about the Launchpad contract
   *
   * @returns Promise<Array<{
   *  "token_type": { "native_token": { "denom": "uscrt" } },
   *  "segment": "25000000000",
   *  "bounding_period": 604800,
   *  "active": true,
   *  "token_decimals": 6,
   *  "locked_balance": "100000000000"
   * }>>
   */
  async info() {
    return this.q.launchpad_info();
  }

  /**
   * Get the balance and entry information for a user
   *
   * @param {string} address
   * @param {string} key
   * @returns Promise<Array<{
   *  "token_type": { "native_token": { "denom": "uscrt" } },
   *  "balance": "100000000000",
   *  "entries": [
   *    "1629402109",
   *    "1629402109",
   *    "1629402109",
   *    "1629402109",
   *  ],
   *  "last_draw": null,
   * }>>
   */
  async userInfo(address: string, key: string) {
    return this.q.user_info({
      address,
      key,
    });
  }

  /**
   * Do a test draw of the addresses
   *
   * @param {number} number
   * @param {string[]} tokens
   * @returns Promise<{
   *  "drawn_addresses": [
   *    "secret1h9we43xcfyljvadjj6wfw6444t8kty4kmajdhl",
   *    "secret1tld98vmz8gq0j738cwvu2feccfwl8wz3tnuu9e",
   *    "secret1avs82agh6g46xna6qklmjnnaj7yq3974ur8qpe",
   *    "secret1udwpspt6czruhrhadtchsjzgznrq8yq9emu6m4"
   *  ]
   * }>
   */
  async draw(number: number, tokens: string[]) {
    return this.q.draw({
      number,
      tokens,
      // @ts-ignore
      timestamp: parseInt(new Date().valueOf() / 1000),
    });
  }

  /**
   * Create viewing key for the agent
   *
   * @param {Agent} agent
   * @param {string} entropy
   * @returns
   */
  createViewingKey = (agent: Agent, entropy = randomHex(32)) =>
    this.tx
      .create_viewing_key({ entropy, padding: null }, agent)
      .then((tx) => ({
        tx,
        key: JSON.parse(decode(tx.data)).create_viewing_key.key,
      }));

  /**
   * Set viewing key for the agent
   *
   * @param {Agent} agent
   * @param {string} key
   * @returns
   */
  setViewingKey = (agent: Agent, key: string) =>
    this.tx.set_viewing_key({ key }, agent).then((tx) => ({
      tx,
      status: JSON.parse(decode(tx.data)).set_viewing_key.key,
    }));
}

