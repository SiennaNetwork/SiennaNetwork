import debug from "debug";
import { assert } from "chai";
import { randomBytes } from "crypto";
import { Scrt, ScrtGas } from "@fadroma/scrt";

import { SiennaSNIP20 } from "../index";
import { SNIP20 } from "../SNIP20.ts";
import { IDO } from "../IDO.ts";
import { Factory } from "../Factory.ts";

import siennajs from "../siennajs/index";

const IdoContract = siennajs.ido.IdoContract;
const Snip20Contract = siennajs.snip20.Snip20Contract;

const log = function () {
  debug(JSON.stringify(arguments, null, 2));
};

const getIDOInitMsg = function (context) {
  return {
    info: {
      // input_token: {
      //   custom_token: {
      //     contract_addr: context.buyingToken.address,
      //     token_code_hash: context.buyingToken.codeHash,
      //   },
      // },
      input_token: {
        native_token: {
          denom: "uscrt",
        },
      },
      rate: "1",
      sold_token: {
        address: context.sellingToken.address,
        code_hash: context.templates.SiennaSNIP20.codeHash,
      },
      whitelist: context.agents
        .map((a, i) => (i > 3 ? null : a.address)) // allow only first 4 agents excluding the admin
        .filter((v) => v !== null),
      max_seats: 5,
      max_allocation: "5",
      min_allocation: "1",
    },
    prng_seed: randomBytes(36).toString("hex"),
    entropy: randomBytes(36).toString("hex"),
    admin: context.agent.address,
    callback: {
      msg: Buffer.from(
        JSON.stringify({
          register_ido: {
            signature: "",
          },
        }),
        "utf8"
      ).toString("base64"),
      contract: {
        address: context.factory.address,
        code_hash: context.templates.Factory.codeHash,
      },
    },
  };
};

const activate_msg = function (start, end) {
  const start_time = start || parseInt(new Date().valueOf() / 1000);
  const end_time = end || parseInt(new Date().valueOf() / 1000 + 5);

  return Buffer.from(
    JSON.stringify({
      activate: {
        start_time,
        end_time,
      },
    }),
    "utf8"
  ).toString("base64");
};

describe("IDO", () => {
  const fees = {
    upload: new ScrtGas(10000000),
    init: new ScrtGas(10000000),
    exec: new ScrtGas(10000000),
    send: new ScrtGas(10000000),
  };

  const context = {};

  context.getIdo = (agent) =>
    new IdoContract(context.ido.address, (agent || context.agent).API);

  context.getToken = (agent) =>
    new Snip20Contract(
      context.sellingToken.address,
      (agent || context.agent).API
    );

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
      SiennaSNIP20: new SiennaSNIP20(),
      IDO: new IDO(),
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
      console.debug(`Uploading ${contract.constructor.name}`);
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
      label: `factory-${parseInt(Math.random() * 100000)}`,
      EXCHANGE: context.templates.SiennaSNIP20,
      AMMTOKEN: context.templates.SiennaSNIP20,
      LPTOKEN: context.templates.SiennaSNIP20,
      IDO: context.templates.IDO,
      LAUNCHPAD: context.templates.SiennaSNIP20,
    });
    await context.factory.instantiate(context.agent);

    context.sellingToken = new SNIP20({
      codeId: context.templates.SiennaSNIP20.codeId,
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
    await context.sellingToken.instantiate(context.agent);

    context.ido = new IDO({
      codeId: context.templates.IDO.codeId,
      label: `ido-${parseInt(Math.random() * 100000)}`,
      initMsg: getIDOInitMsg(context),
    });
    await context.ido.instantiate(context.agent);

    context.viewkey = (
      await context.sellingToken.createViewingKey(context.agent)
    ).key;

    await context.sellingToken.mint(25);
    await context.sellingToken.tx.send({
      recipient: context.ido.address,
      amount: "25",
      msg: activate_msg(undefined, parseInt(new Date().valueOf() / 1000) + 60),
    });
  });

  it("Does a swap successfully", async function () {
    this.timeout(0);
    const amount = 1_000_000;
    const buyer = context.agents[1];

    const ido = context.getIdo(buyer);
    const res = await ido.exec().swap(`${amount}`);

    assert.strictEqual(
      res.logs[0].events[1].attributes[1].value,
      `${amount}uscrt`
    );
    assert.strictEqual(res.logs[0].events[2].attributes[3].value, "1");
  });

  it("Fails swapping with non whitelisted agent", async function () {
    this.timeout(0);
    const amount = 1_000_000;
    const buyer = context.agents[3];
    const ido = context.getIdo(buyer);

    try {
      await ido.exec().swap(`${amount}`);

      assert.strictEqual("shouldn't have passed!", false);
    } catch (e) {}
  });

  it("Fails swapping when trying to swap below and above limits", async function () {
    this.timeout(0);
    const lowAmount = 999_999;
    const highAmount = 6_000_000;
    const buyer = context.agents[1];
    const ido = context.getIdo(buyer);

    try {
      await ido.exec().swap(`${lowAmount}`);

      assert.strictEqual("shouldn't have passed, lowAmount!", false);
    } catch (e) {}

    try {
      await ido.exec().swap(`${highAmount}`);

      assert.strictEqual("shouldn't have passed, highAmount!", false);
    } catch (e) {}
  });

  it("Fails swapping when already swapped the max limit", async function () {
    this.timeout(0);
    const amount = 5_000_000;
    const secondAmount = 1_000_000;
    const buyer = context.agents[1];
    const ido = context.getIdo(buyer);

    await ido.exec().swap(`${amount}`);

    try {
      await ido.exec().swap(`${secondAmount}`);

      assert.strictEqual("shouldn't have passed, secondAmount!", false);
    } catch (e) {}
  });

  it("Can swap multiple times as long as its all within the limits", async function () {
    this.timeout(0);
    const amount = 2_500_000;
    const buyer = context.agents[1];
    const ido = context.getIdo(buyer);

    await ido.exec().swap(`${amount}`);
    await ido.exec().swap(`${amount}`);
  });

  it("Cannot swap before sale starts", async function () {
    this.timeout(0);
    context.ido1 = new IDO({
      codeId: context.templates.IDO.codeId,
      label: `ido-${parseInt(Math.random() * 100000)}`,
      initMsg: getIDOInitMsg(context),
    });
    await context.ido1.instantiate(context.agent);

    const ido = new IdoContract(context.ido1.address, context.agent.API);

    await context.sellingToken.mint(25);

    const start = parseInt(new Date().valueOf() / 1000) + 240;
    const end = parseInt(new Date().valueOf() / 1000) + 480;

    await ido.exec().activate("25", end, start);

    const amount = 2_500_000;
    const buyer = context.agents[1];

    const buyerIdo = new IdoContract(context.ido1.address, buyer.API);

    try {
      await buyerIdo.exec().swap(`${amount}`);
      assert.strictEqual(
        "Shouldn't get here, swap is before sale starts",
        false
      );
    } catch (e) {
      assert.strictEqual(e.message.includes("\"Sale hasn't started yet"), true);
    }
  });

  it("Cannot swap after sale ends", async function () {
    this.timeout(0);
    context.ido1 = new IDO({
      codeId: context.templates.IDO.codeId,
      label: `ido-${parseInt(Math.random() * 100000)}`,
      initMsg: getIDOInitMsg(context),
    });
    await context.ido1.instantiate(context.agent);

    await context.sellingToken.mint(25);
    
    const ido = new IdoContract(context.ido1.address, context.agent.API);

    await context.sellingToken.mint(25);

    const start = parseInt(new Date().valueOf() / 1000);
    const end = parseInt(new Date().valueOf() / 1000) + 60;

    await ido.exec().activate("25", end, start);

    const amount = 2_500_000;
    const buyer = context.agents[1];
    const buyerIdo = new IdoContract(context.ido1.address, buyer.API);

    try {
      await new Promise((ok) => setTimeout(ok, 120000));
      await buyerIdo.exec().swap(`${amount}`);
      assert.strictEqual("Shouldn't get here, swap is after sale ends", false);
    } catch (e) {
      log(e)
      assert.strictEqual(e.message.includes('"Sale has ended"'), true);
    }
  });

  it("Admin can add another buyer that can then swap funds", async function () {
    this.timeout(0);

    const buyer = context.agents[2];
    const ido = context.getIdo();

    await ido.exec().admin_add_addresses([buyer.address]);
    
    const amount = 1_000_000;

    const buyerIdo = new IdoContract(context.ido.address, buyer.API);

    await buyerIdo.exec().swap(`${amount}`);
  });

  it("Admin can refund and claim amounts after the sale has ended", async function () {
    this.timeout(0);

    context.ido1 = new IDO({
      codeId: context.templates.IDO.codeId,
      label: `ido-${parseInt(Math.random() * 100000)}`,
      initMsg: getIDOInitMsg(context),
    });
    await context.ido1.instantiate(context.agent);

    await context.sellingToken.mint(25);

    const ido = new IdoContract(context.ido1.address, context.agent.API);
    const start = parseInt(new Date().valueOf() / 1000);
    const end = parseInt(new Date().valueOf() / 1000) + 20;

    await ido.exec().activate("25", end, start);

    const balance = await context.sellingToken.balance(
      context.agent.address,
      context.viewkey
    );

    assert.strictEqual(balance, "0");

    const buyer = context.agents[1];
    const amount = 5_000_000;
    const buyerIdo = new IdoContract(context.ido1.address, buyer.API);

    await buyerIdo.exec().swap(`${amount}`);

    await new Promise((ok) => setTimeout(ok, 60000));

    await ido.exec().admin_refund();

    const balanceAfter = await context.sellingToken.balance(
      context.agent.address,
      context.viewkey
    );

    assert.strictEqual(balanceAfter, "20");

    const nativeBalanceBefore = await context.agent.balance;
    await ido.exec().admin_claim();
    const nativeBalanceAfter = await context.agent.balance;

    const approxBalance = Math.abs(
      parseInt(
        (parseInt(nativeBalanceAfter) - parseInt(nativeBalanceBefore)) /
          1_000_000
      )
    );

    // Approximate balance is 4-5 after dividing by 1mil because of the fees.
    assert.strictEqual([4, 5].indexOf(approxBalance) != -1, true);
  });

  it("Admin cannot refund before sale ends", async function () {
    this.timeout(0);

    try {
      await context.getIdo().exec().admin_refund();
    } catch (e) {
      console.log(e)
      assert.strictEqual(e.message.includes("Sale hasn't finished yet"), true);
    }
  });

  it("Admin can get correct status of the ido contract", async function () {
    this.timeout(0);

    const buyer = context.agents[1];
    const amount = 5_000_000;

    await context.getIdo(buyer).exec().swap(`${amount}`);

    const res = await context.getIdo().query().get_sale_status();

    assert.strictEqual(res.total_allocation, "25"); // total allocation
    assert.strictEqual(res.available_for_sale, "20"); // available for sale
  });

  it("Attempt instantiate and swap with a custom buying token", async function () {
    const swap_msg = function () {
      return Buffer.from(JSON.stringify({ swap: {} }), "utf8").toString(
        "base64"
      );
    };

    this.timeout(0);
    context.buyingToken = new SNIP20({
      codeId: context.templates.SiennaSNIP20.codeId,
      label: `token-${parseInt(Math.random() * 100000)}`,
      initMsg: {
        prng_seed: randomBytes(36).toString("hex"),
        name: "BuyToken",
        symbol: "BYT",
        decimals: 6,
        config: {
          public_total_supply: true,
          enable_deposit: true,
          enable_redeem: true,
          enable_mint: true,
          enable_burn: true,
        },
      },
    });
    await context.buyingToken.instantiate(context.agent);

    const initMsg = getIDOInitMsg(context);
    initMsg.info.input_token = {
      custom_token: {
        contract_addr: context.buyingToken.address,
        token_code_hash: context.templates.SiennaSNIP20.codeHash,
      },
    };

    context.idoB = new IDO({
      codeId: context.templates.IDO.codeId,
      label: `ido-${parseInt(Math.random() * 100000)}`,
      initMsg,
    });
    await context.idoB.instantiate(context.agent);
    const statusBefore = await context.idoB.q.sale_status();
    assert.strictEqual(statusBefore.status.is_active, false);

    await context.sellingToken.mint(25);
    await context.sellingToken.tx.send({
      recipient: context.idoB.address,
      amount: "25",
      msg: activate_msg(undefined, parseInt(new Date().valueOf() / 1000) + 300),
    });

    const statusAfter = await context.idoB.q.sale_status();
    assert.strictEqual(statusAfter.status.is_active, true);

    const buyer = context.agents[1];
    const amount = 5_000_000;

    const buyViewkey = (await context.buyingToken.createViewingKey(buyer)).key;
    const sellingViewkey = (await context.sellingToken.createViewingKey(buyer))
      .key;

    await context.buyingToken.mint(10_000_000, undefined, buyer.address);

    await context.buyingToken.sendIdo(
      context.idoB.address,
      amount,
      undefined,
      buyer
    );

    const buyBalanceAfter = await context.buyingToken.balance(
      buyer.address,
      buyViewkey
    );
    const sellBalanceAfter = await context.sellingToken.balance(
      buyer.address,
      sellingViewkey
    );

    assert.strictEqual(buyBalanceAfter, `${5_000_000}`);
    assert.strictEqual(sellBalanceAfter, "5");

    try {
      await context.buyingToken.tx.send(
        {
          recipient: context.idoB.address,
          amount: `${amount}`,
          msg: swap_msg(),
        },
        buyer
      );
    } catch (e) {
      log(e);
    }

    const buyBalanceAfterFail = await context.buyingToken.balance(
      buyer.address,
      buyViewkey
    );
    const sellBalanceAfterFail = await context.sellingToken.balance(
      buyer.address,
      sellingViewkey
    );

    assert.strictEqual(buyBalanceAfterFail, "5000000");
    assert.strictEqual(sellBalanceAfterFail, "5");
  });

  it("Try minting selling token onto IDO contract and then sending 0 tokens to activate the contract", async function () {
    this.timeout(0);

    context.idoB = new IDO({
      codeId: context.templates.IDO.codeId,
      label: `ido-${parseInt(Math.random() * 100000)}`,
      initMsg: getIDOInitMsg(context),
    });
    await context.idoB.instantiate(context.agent);

    const ido = new IdoContract(context.idoB.address, context.agent.API);

    const statusBefore = await ido.query().get_sale_status();
    assert.strictEqual(statusBefore.is_active, false);

    await context.sellingToken.mint(25, undefined, context.idoB.address);
    await ido.exec().activate("0", parseInt(new Date().valueOf() / 1000) + 60);

    const statusAfter = await ido.query().get_sale_status();
    assert.strictEqual(statusAfter.is_active, true);
  });

  after(async function cleanupAll() {
    this.timeout(0);
    await context.node.terminate();
  });
});
