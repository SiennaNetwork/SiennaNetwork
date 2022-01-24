import debug from "debug";
import { assert } from "chai";
import { randomBytes } from "crypto";
import { Scrt, ScrtGas } from "@hackbg/fadroma";

import { Launchpad } from "./Launchpad";
import { SNIP20 } from "./SNIP20";
import { Factory } from "./Factory";

import * as siennajs from "./siennajs/index";

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
    const agentNames = ["ALICE", "BOB", "CHARLIE", "MALLORY"];
    context.chain = await Scrt.localnet_1_0().init();
    context.node = context.chain.node;
    context.agent = await context.chain.getAgent(
      context.node.genesisAccount("ADMIN")
    );

    const agents = (context.agents = await Promise.all(
      agentNames.map((name) =>
        context.chain.getAgent(context.node.genesisAccount(name))
      )
    ));
    console.log({ agents });
    context.agent.API.fees = fees;

    const T1 = +new Date();
    console.debug(`connecting took ${T1 - T0}msec`);

    context.templates = {
      SNIP20: new SNIP20(),
      Launchpad: new Launchpad(),
      Factory: new Factory(),
    };

    // build the contracts
    await Promise.all(
      Object.values(context.templates).map((contract) => contract.build())
    );

    const T2 = +new Date();
    console.debug(`building took ${T2 - T1}msec`);

    // upload the contracts
    for (const contract of Object.values(context.templates)) {
      await contract.upload(context.agent);
      await context.agent.nextBlock;
    }

    const T3 = +new Date();
    console.debug(`uploading took ${T3 - T2}msec`);
    console.debug(`total preparation time: ${T3 - T0}msec`);
  });

  beforeEach(async function setupEach() {
    this.timeout(0);
    context.factory = new Factory({
      codeId: context.templates.Factory.codeId,
      AMMTOKEN: context.templates.SNIP20,
      LPTOKEN: context.templates.SNIP20,
      IDO: context.templates.SNIP20,
      EXCHANGE: context.templates.SNIP20,
      LAUNCHPAD: context.templates.Launchpad,
      label: `factory-${parseInt(Math.random() * 100000)}`,
    });
    await context.factory.instantiate(context.agent);

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
    });
    await context.token.instantiate(context.agent);

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
                contract_addr: context.token.init.address,
                token_code_hash: context.templates.SNIP20.codeHash,
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
            address: context.factory.init.address,
            code_hash: context.templates.Factory.codeHash,
          },
        },
      },
    });
    await context.launchpad.instantiate(context.agent);
  });

  it("Has instantiated launchpad successfully", async function () {
    this.timeout(0);
  });

  it("User can lock tokens and will have proper number of entries", async function () {
    this.timeout(0);

    const buyer = context.agents[1];

    await context.token.mint(100, undefined, buyer.address);

    await context.token.lockLaunchpad(context.launchpad.address, 50, buyer);

    const res = await context.launchpad.info();

    assert.strictEqual(res.launchpad_info[1].locked_balance, "50");

    const viewkey = (await context.launchpad.createViewingKey(buyer)).key;

    const userRes = await context.launchpad.userInfo(buyer.address, viewkey);

    assert.strictEqual(userRes.user_info[0].balance, "50");
    assert.strictEqual(userRes.user_info[0].entries.length, 2);
  });

  it("User can lock tokens and will have proper number of entries", async function () {
    this.timeout(0);
    for (const a of context.agents) {
      await context.token.mint(100, undefined, a.address);
      await context.token.lockLaunchpad(context.launchpad.address, 50, a);

      const viewkey = (await context.launchpad.createViewingKey(a)).key;
      const userRes = await context.launchpad.userInfo(a.address, viewkey);

      assert.strictEqual(userRes.user_info[0].balance, "50");
      assert.strictEqual(userRes.user_info[0].entries.length, 2);
    }

    const res = await context.launchpad.draw(4, [context.token.address]);

    for (const a of context.agents) {
      assert.strictEqual(res.drawn_addresses.includes(a.address), true);
    }
  });

  it("User can unlock tokens", async function () {
    this.timeout(0);
    const buyer = context.agents[0];

    await context.token.mint(100, undefined, buyer.address);
    await context.token.lockLaunchpad(context.launchpad.address, 50, buyer);

    const viewkey = (await context.launchpad.createViewingKey(buyer)).key;
    const userRes = await context.launchpad.userInfo(buyer.address, viewkey);

    assert.strictEqual(userRes.user_info[0].balance, "50");
    assert.strictEqual(userRes.user_info[0].entries.length, 2);

    const res = await context.token.unlockLaunchpad(
      context.launchpad.address,
      1,
      buyer
    );

    assert.strictEqual(res.logs[0].events[1].attributes[2].value, "1"); // unlocked entries
    assert.strictEqual(res.logs[0].events[1].attributes[3].value, "25"); // amount unlocked
    assert.strictEqual(res.logs[0].events[1].attributes[4].value, "1"); // left entries in the launchpad
  });

  it("Attempt to remove the token and verify the balances of users are sent back", async function () {
    this.timeout(0);
    for (const a of context.agents) {
      await context.token.mint(100, undefined, a.address);
      await context.token.lockLaunchpad(context.launchpad.address, 50, a);

      const viewkey = (await context.launchpad.createViewingKey(a)).key;
      a.tokenViewkey = (await context.token.createViewingKey(a)).key;

      const userRes = await context.launchpad.userInfo(a.address, viewkey);
      const balance = await context.token.balance(a.address, a.tokenViewkey);

      assert.strictEqual(userRes.user_info[0].balance, "50");
      assert.strictEqual(userRes.user_info[0].entries.length, 2);
      assert.strictEqual(balance, "50");
    }

    await context.launchpad.adminRemoveToken(1);

    for (const a of context.agents) {
      const balance = await context.token.balance(a.address, a.tokenViewkey);
      assert.strictEqual(balance, "100");
    }
  });

  after(async function cleanupAll() {
    this.timeout(0);
    await context.node.terminate();
  });
});

