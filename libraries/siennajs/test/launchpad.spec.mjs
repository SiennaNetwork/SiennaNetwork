import debug from "debug";
import { assert } from "chai";
import { randomBytes } from "crypto";
import { Scrt, ScrtGas } from "@fadroma/scrt";

import { Launchpad } from "../Launchpad";
import { SNIP20 } from "../SNIP20";
import { Factory } from "../Factory";

import siennajs from "../siennajs/index";

const LaunchpadContract = siennajs.launchpad.LaunchpadContract;
const Snip20Contract = siennajs.snip20.Snip20Contract;

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

    context.t = new SNIP20({
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
    await context.t.instantiate(context.agent);

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
                contract_addr: context.t.init.address,
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

    context.getLaunchpad = (agent) =>
      new LaunchpadContract(
        context.launchpad.address,
        (agent || context.agent).API
      );
    
    context.token = new Snip20Contract(
      context.t.address,
      context.agent.API
    );

    context.viewkey = await context.token.exec().create_viewing_key();
  });

  it("Has instantiated launchpad successfully", async function () {
    this.timeout(0);

    await context.getLaunchpad().query().info();
  });

  it("User can lock tokens and will have proper number of entries", async function () {
    this.timeout(0);

    const buyer = context.agents[1];

    await context.token.exec().mint(buyer.address, "100");

    const launchpad = context.getLaunchpad(buyer);

    await launchpad.exec().lock("50", context.token.address);

    const res = await launchpad.query().info();

    assert.strictEqual(res[1].locked_balance, "50");

    const viewkey = (await launchpad.exec().create_viewing_key()).key;

    const userRes = await launchpad.query().user_info(buyer.address, viewkey);

    assert.strictEqual(userRes[0].balance, "50");
    assert.strictEqual(userRes[0].entries.length, 2);
  });

  it("User can lock tokens and will have proper number of entries", async function () {
    this.timeout(0);
    for (const a of context.agents) {
      await context.token.exec().mint(a.address, "100");
      const launchpad = context.getLaunchpad(a);

      await launchpad.exec().lock("50", context.token.address);

      const viewkey = (await launchpad.exec().create_viewing_key()).key;
      const userRes = await launchpad.query().user_info(a.address, viewkey);

      assert.strictEqual(userRes[0].balance, "50");
      assert.strictEqual(userRes[0].entries.length, 2);
    }

    const res = await context
      .getLaunchpad()
      .query()
      .draw(4, [context.token.address]);

    for (const a of context.agents) {
      assert.strictEqual(res.includes(a.address), true);
    }
  });

  it("User can unlock tokens", async function () {
    this.timeout(0);
    const buyer = context.agents[0];
    const launchpad = new LaunchpadContract(context.launchpad.address, buyer.API);

    await context.token.exec().mint(buyer.address, "100");

    const buyerViewkey = (await (new Snip20Contract(context.token.address, buyer.API)).exec().create_viewing_key()).key;
    const balanceBefore = await context.token.query().get_balance(buyerViewkey, buyer.address);
    assert.strictEqual("100", balanceBefore);

    await launchpad.exec().lock("50", context.t.address);
    
    const viewkey = (await launchpad.exec().create_viewing_key()).key;
    const userRes = await launchpad.query().user_info(buyer.address, viewkey);

    assert.strictEqual(userRes[0].balance, "50");
    assert.strictEqual(userRes[0].entries.length, 2);

    const res = await launchpad.exec().unlock(1, context.t.address);

    const balanceAfter = await context.token.query().get_balance(buyerViewkey, buyer.address);
    assert.strictEqual(balanceAfter, "75");
  });

  it("Attempt to remove the token and verify the balances of users are sent back", async function () {
    this.timeout(0);
    for (const a of context.agents) {
      await context.token.exec().mint(a.address, "100");

      const token = new Snip20Contract(context.t.address, a.API);
      const launchpad = new LaunchpadContract(context.launchpad.address, a.API);
      await launchpad.exec().lock("50", context.t.address);

      const viewkey = (await launchpad.exec().create_viewing_key()).key;
      a.tokenViewkey = (await token.exec().create_viewing_key()).key;

      const userRes = await launchpad.query().user_info(a.address, viewkey);
      const balance = await context.token.query().get_balance(a.tokenViewkey, a.address);

      assert.strictEqual(userRes[0].balance, "50");
      assert.strictEqual(userRes[0].entries.length, 2);
      assert.strictEqual(balance, "50");
    }

    await context.getLaunchpad().exec().admin_remove_token(1);

    for (const a of context.agents) {
      const balance = await context.token.query().get_balance(a.tokenViewkey, a.address);
      assert.strictEqual(balance, "100");
    }
  });

  after(async function cleanupAll() {
    this.timeout(0);
    await context.node.terminate();
  });
});
