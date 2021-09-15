import debug from "debug";
import { assert } from "chai";
import { randomBytes } from "crypto";
import { Scrt, ScrtGas } from "@fadroma/scrt";

import { abs } from "../ops/index";

import { Launchpad } from "./Launchpad";
import { AMMSNIP20 } from "./SNIP20";
import { IDO } from "./IDO";
import { Factory } from "./Factory";

const log = function () {
  debug("out")(JSON.stringify(arguments, null, 2));
};

describe("Launchpad", () => {
  const fees = {
    upload: new ScrtGas(10000000),
    init: new ScrtGas(100000000),
    exec: new ScrtGas(10000000),
    send: new ScrtGas(10000000),
  };

  const context = {};

  before(async function setupAll() {
    this.timeout(0);
    const T0 = +new Date();

    context.AMMSNIP20 = new AMMSNIP20();
    context.Launchpad = new Launchpad();
    context.Factory   = new Factory();

    // connect to a localnet with a large number of predefined agents
    const numberOfAgents = 10;
    const agentNames = [...Array(numberOfAgents)].map((_, i) => `Agent${i}`);
    context.chain = Scrt.localnet_1_0();
    await context.chain.node.respawn();
    context.node = context.chain.node;
    context.agent = await context.chain.getAgent(context.node.genesisAccount("ADMIN"));

    const agents = await Promise.all(
      agentNames.map((name) =>
        context.chain.getAgent(name, { mnemonic: context.node.genesisAccount(name).mnemonic })
      )
    );
    console.log({ agents });
    agent.API.fees = fees;

    const T1 = +new Date();
    console.debug(`connecting took ${T1 - T0}msec`);

    // build the contracts
    await Promise.all([
      context.AMMSNIP20.build(),
      context.Launchpad.build(),
      context.Factory.build(),
    ]);

    const T2 = +new Date();
    console.debug(`building took ${T2 - T1}msec`);

    // upload the contracts
    const { codeId: tokenCodeId, originalChecksum: tokenCodeHash } =
      await context.AMMSNIP20.uploadCached();
    await context.agent.nextBlock;

    const { codeId: launchpadCodeId, originalChecksum: launchpadCodeHash } =
      await context.Launchpad.uploadCached();
    await context.agent.nextBlock;

    const { codeId: factoryCodeId, originalChecksum: factoryCodeHash } =
      await context.Factory.uploadCached();
    await context.agent.nextBlock;

    const T3 = +new Date();
    console.debug(`uploading took ${T3 - T2}msec`);
    console.debug(`total preparation time: ${T3 - T0}msec`);

    Object.assign(context, {
      builder,
      agents,
      tokenInfo: { id: tokenCodeId, code_hash: tokenCodeHash },
      launchpadInfo: { id: launchpadCodeId, code_hash: launchpadCodeHash },
      factoryInfo: { id: factoryCodeId, code_hash: factoryCodeHash },
    });
  });

  beforeEach(async function setupEach() {
    this.timeout(0);
    context.factory = await context.agent.instantiate(
      new Factory({
        codeId: context.factoryInfo.id,
        label: `factory-${parseInt(Math.random() * 100000)}`,
        initMsg: {
          prng_seed: randomBytes(36).toString("hex"),
          snip20_contract: context.tokenInfo,
          lp_token_contract: context.tokenInfo,
          pair_contract: context.tokenInfo,
          launchpad_contract: context.launchpadInfo,
          ido_contract: context.tokenInfo, // dummy so we don't have to build it
          exchange_settings: {
            swap_fee: {
              nom: 1,
              denom: 1,
            },
            sienna_fee: {
              nom: 1,
              denom: 1,
            },
            //   sienna_burner: null,
          },
        },
      })
    );

    context.token = await context.agent.instantiate(
      new SNIP20({
        codeId: context.tokenInfo.id,
        label: `token-${parseInt(Math.random() * 100000)}`,
        initMsg: {
          prng_seed: randomBytes(36).toString("hex"),
          name: "Token",
          symbol: "TKN",
          decimals: 18,
          config: {
            public_total_supply: true,
            enable_deposit: true,
            enable_redeem: true,
            enable_mint: true,
            enable_burn: true,
          },
        },
      })
    );

    context.viewkey = (await context.token.createViewingKey(context.agent)).key;

    context.launchpad = await context.agent.instantiate(
      new Launchpad({
        codeId: context.launchpadInfo.id,
        label: `launchpad-${parseInt(Math.random() * 100000)}`,
        initMsg: {
          tokens: [
            {
              token_type: { native_token: { denom: "uscrt" } },
              segment: "25",
              bounding_period: 10,
            },
            {
              token_type: {
                custom_token: {
                  contract_addr: context.token.address,
                  token_code_hash: context.token.codeHash,
                },
              },
              segment: "25",
              bounding_period: 10,
            },
          ],
          prng_seed: randomBytes(36).toString("hex"),
          entropy: randomBytes(36).toString("hex"),
          admin: context.agent.address,
          callback: {
            msg: Buffer.from(
              JSON.stringify({
                register_launchpad: {
                  signature: "",
                },
              }),
              "utf8"
            ).toString("base64"),
            contract: {
              address: context.factory.address,
              code_hash: context.factory.codeHash,
            },
          },
        },
      })
    );
  });

  it("Has instantiated launchpad successfully", async function () {
    this.timeout(0);

    const buyer = context.agents[1];

    await context.token.mint(100, undefined, buyer.address);

    await context.token.lockLaunchpad(context.launchpad.address, 50, buyer);

    const res = await context.launchpad.info();
    log(res);
  });

  after(async function cleanupAll() {
    this.timeout(0);
    await context.node.kill();
  });
});
