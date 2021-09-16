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

    // connect to a localnet with a large number of predefined agents
    const agentNames = ['ALICE', 'BOB', 'CHARLIE', 'MALLORY'];
    context.chain = await Scrt.localnet_1_0().init();
    context.node = context.chain.node;
    context.agent = await context.chain.getAgent(context.node.genesisAccount("ADMIN"));

    const agents = context.agents = await Promise.all(
      agentNames.map((name) =>
        context.chain.getAgent(context.node.genesisAccount(name))
      )
    );
    console.log({ agents });
    context.agent.API.fees = fees;

    const T1 = +new Date();
    console.debug(`connecting took ${T1 - T0}msec`);

    context.templates = {
      AMMSNIP20: new AMMSNIP20(),
      Launchpad: new Launchpad(),
      Factory: new Factory()
    };

    // build the contracts
    await Promise.all(Object.values(context.templates).map(contract=>contract.build()));

    const T2 = +new Date();
    console.debug(`building took ${T2 - T1}msec`);

    // upload the contracts
    for (const contract of Object.values(context.templates)) {
      await contract.upload(context.agent);
      await context.agent.nextBlock;
    };

    const T3 = +new Date();
    console.debug(`uploading took ${T3 - T2}msec`);
    console.debug(`total preparation time: ${T3 - T0}msec`);
  });

  beforeEach(async function setupEach() {
    this.timeout(0);
    context.factory = new Factory({
      agent: context.agent,
      codeId: context.templates.Factory.codeId,
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
    });
    await context.factory.instantiate();

    context.token = new SNIP20({
      codeId: context.templates.SNIP20.codeId,
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
    await context.token.instantiate();

    context.viewkey = (await context.token.createViewingKey(context.agent)).key;

    context.launchpad = new Launchpad({
      codeId: context.templates.Launchpad.codeId,
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
    });
    await context.launchpad.instantiate();
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
