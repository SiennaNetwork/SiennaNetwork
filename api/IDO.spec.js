import { randomBytes } from "crypto";
import { SecretNetwork } from "@fadroma/scrt-agent";
import { gas } from "@fadroma/scrt-agent/gas.js";
import { abs } from "../ops/lib/index.js";
import SNIP20 from "./SNIP20.js";
import IDO from "./IDO.js";
import Factory from "./Factory.js";
import debug from "debug";
import { assert } from "chai";

const log = function () {
  debug("out")(JSON.stringify(arguments, null, 2));
};

const getIDOInitMsg = function (context, start, end) {
  const start_time = start || parseInt(new Date().valueOf() / 1000);
  const end_time = end || parseInt(new Date().valueOf() / 1000 + 5);

  return {
    info: {
      // input_token: {
      //   custom_token: {
      //     contract_addr: context.buyingToken.address,
      //     token_code_hash: context.buyingToken.code_hash,
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
        code_hash: context.sellingToken.codeHash,
      },
      whitelist: context.agents
        .map((a, i) => (i == 0 || i > 4 ? null : a.address)) // allow only first 4 agents excluding the admin
        .filter((v) => v !== null),
      max_seats: 5,
      max_allocation: "5",
      min_allocation: "1",
      start_time,
      end_time,
      prng_seed: randomBytes(36).toString("hex"),
      entropy: "",
    },
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
        code_hash: context.factory.codeHash,
      },
    },
  };
};

describe("IDO", () => {
  const fees = {
    upload: gas(10000000),
    init: gas(10000000),
    exec: gas(10000000),
    send: gas(10000000),
  };

  const context = {};

  before(async function setupAll() {
    this.timeout(0);
    const T0 = +new Date();

    // connect to a localnet with a large number of predefined agents
    const numberOfAgents = 10;
    const agentNames = [...Array(numberOfAgents)].map((_, i) => `Agent${i}`);
    const localnet = SecretNetwork.localnet({
      stateBase: abs("artifacts"),
      genesisAccounts: ["ADMIN", ...agentNames],
    });
    const { node, network, builder, agent } = await localnet.connect();
    const agents = await Promise.all(
      agentNames.map((name) =>
        network.getAgent(name, { mnemonic: node.genesisAccount(name).mnemonic })
      )
    );
    console.log({ agents });
    agent.API.fees = fees;

    const T1 = +new Date();
    console.debug(`connecting took ${T1 - T0}msec`);

    // build the contracts
    const workspace = abs();
    const [tokenBinary, idoBinary, factoryBinary] = await Promise.all([
      builder.build({ workspace, crate: "amm-snip20" }),
      builder.build({ workspace, crate: "ido" }),
      builder.build({ workspace, crate: "factory" }),
    ]);

    const T2 = +new Date();
    console.debug(`building took ${T2 - T1}msec`);

    // upload the contracts
    const { codeId: tokenCodeId, originalChecksum: tokenCodeHash } =
      await builder.uploadCached(tokenBinary);
    await agent.nextBlock;

    const { codeId: idoCodeId, originalChecksum: idoCodeHash } =
      await builder.uploadCached(idoBinary);
    await agent.nextBlock;

    const { codeId: factoryCodeId, originalChecksum: factoryCodeHash } =
      await builder.uploadCached(factoryBinary);
    await agent.nextBlock;

    const T3 = +new Date();
    console.debug(`uploading took ${T3 - T2}msec`);
    console.debug(`total preparation time: ${T3 - T0}msec`);

    Object.assign(context, {
      node,
      network,
      builder,
      agent,
      agents,
      tokenInfo: { id: tokenCodeId, code_hash: tokenCodeHash },
      idoInfo: { id: idoCodeId, code_hash: idoCodeHash },
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
          snip20_contract: context.tokenInfo,
          lp_token_contract: context.tokenInfo,
          pair_contract: context.tokenInfo,
          ido_contract: context.idoInfo,
          exchange_settings: {
            swap_fee: {
              nom: 1,
              denom: 1,
            },
            sienna_fee: {
              nom: 1,
              denom: 1,
            },
            //   sienna_burner: null
          },
        },
      })
    );

    context.sellingToken = await context.agent.instantiate(
      new SNIP20({
        codeId: context.tokenInfo.id,
        label: `sold-token-${parseInt(Math.random() * 100000)}`,
        initMsg: {
          prng_seed: randomBytes(36).toString("hex"),
          name: "SoldToken",
          symbol: "SDT",
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
      
    context.viewkey = (await context.sellingToken.createViewingKey(context.agent)).key;

    // context.buyingToken = await context.agent.instantiate(
    //   new SNIP20({
    //     codeId: context.tokenInfo.id,
    //     label: `buy-token-${parseInt(Math.random() * 100000)}`,
    //     initMsg: {
    //       prng_seed: randomBytes(36).toString("hex"),
    //       name: "BuyToken",
    //       symbol: "BYT",
    //       decimals: 6,
    //       config: {
    //         public_total_supply: true,
    //         enable_deposit: true,
    //         enable_redeem: true,
    //         enable_mint: true,
    //         enable_burn: true,
    //       },
    //     },
    //   })
    // );

    context.ido = await context.agent.instantiate(
      new IDO({
        codeId: context.idoInfo.id,
        label: `ido-${parseInt(Math.random() * 100000)}`,
        initMsg: getIDOInitMsg(
          context,
          undefined,
          parseInt(new Date().valueOf() / 1000) + 60
        ),
      })
    );

    await context.sellingToken.mint(25, undefined, context.ido.address);
  });

    it("Does a swap successfully", async function () {
      this.timeout(0);
      const amount = 1_000_000;
      const buyer = context.agents[1];

      const res = await context.ido.tx.swap(
        { amount: `${amount}` },
        buyer,
        undefined,
        [{ amount: `${amount}`, denom: "uscrt" }]
      );

      assert.strictEqual(
        res.logs[0].events[1].attributes[1].value,
        `${amount}uscrt`
      );
      assert.strictEqual(res.logs[0].events[2].attributes[3].value, "1");
    });

    it("Fails swapping with non whitelisted agent", async function () {
      this.timeout(0);
      const amount = 1_000_000;
      const buyer = context.agents[6];

      try {
        await context.ido.tx.swap({ amount: `${amount}` }, buyer, undefined, [
          { amount: `${amount}`, denom: "uscrt" },
        ]);

        assert.strictEqual("shouldn't have passed!", false);
      } catch (e) {}
    });

    it("Fails swapping when trying to swap below and above limits", async function () {
      this.timeout(0);
      const lowAmount = 999_999;
      const highAmount = 6_000_000;
      const buyer = context.agents[1];

      try {
        await context.ido.tx.swap({ amount: `${lowAmount}` }, buyer, undefined, [
          { amount: `${lowAmount}`, denom: "uscrt" },
        ]);

        assert.strictEqual("shouldn't have passed, lowAmount!", false);
      } catch (e) {}

      try {
        await context.ido.tx.swap({ amount: `${highAmount}` }, buyer, undefined, [
          { amount: `${highAmount}`, denom: "uscrt" },
        ]);

        assert.strictEqual("shouldn't have passed, highAmount!", false);
      } catch (e) {}
    });

    it("Fails swapping when already swapped the max limit", async function () {
      this.timeout(0);
      const amount = 5_000_000;
      const secondAmount = 1_000_000;
      const buyer = context.agents[1];

      await context.ido.tx.swap({ amount: `${amount}` }, buyer, undefined, [
        { amount: `${amount}`, denom: "uscrt" },
      ]);

      try {
        await context.ido.tx.swap(
          { amount: `${secondAmount}` },
          buyer,
          undefined,
          [{ amount: `${secondAmount}`, denom: "uscrt" }]
        );

        assert.strictEqual("shouldn't have passed, secondAmount!", false);
      } catch (e) {}
    });

    it("Can swap multiple times as long as its all within the limits", async function () {
      this.timeout(0);
      const amount = 2_500_000;
      const buyer = context.agents[1];

      await context.ido.tx.swap({ amount: `${amount}` }, buyer, undefined, [
        { amount: `${amount}`, denom: "uscrt" },
      ]);

      await context.ido.tx.swap({ amount: `${amount}` }, buyer, undefined, [
        { amount: `${amount}`, denom: "uscrt" },
      ]);
    });

    it("Cannot swap before sale starts", async function () {
      this.timeout(0);
      context.ido1 = await context.agent.instantiate(
        new IDO({
          codeId: context.idoInfo.id,
          label: `ido-${parseInt(Math.random() * 100000)}`,
          initMsg: getIDOInitMsg(
            context,
            parseInt(new Date().valueOf() / 1000) + 60,
            parseInt(new Date().valueOf() / 1000) + 120
          ),
        })
      );

      await context.sellingToken.mint(25, undefined, context.ido1.address);

      const amount = 2_500_000;
      const buyer = context.agents[1];

      try {
        await context.ido1.tx.swap({ amount: `${amount}` }, buyer, undefined, [
          { amount: `${amount}`, denom: "uscrt" },
        ]);
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
      context.ido1 = await context.agent.instantiate(
        new IDO({
          codeId: context.idoInfo.id,
          label: `ido-${parseInt(Math.random() * 100000)}`,
          initMsg: getIDOInitMsg(
            context,
            parseInt(new Date().valueOf() / 1000) - 120,
            parseInt(new Date().valueOf() / 1000) - 60
          ),
        })
      );

      await context.sellingToken.mint(25, undefined, context.ido1.address);

      const amount = 2_500_000;
      const buyer = context.agents[1];

      try {
        await context.ido1.tx.swap({ amount: `${amount}` }, buyer, undefined, [
          { amount: `${amount}`, denom: "uscrt" },
        ]);
        assert.strictEqual("Shouldn't get here, swap is after sale ends", false);
      } catch (e) {
        assert.strictEqual(e.message.includes('"Sale has ended"'), true);
      }
    });

    it("Admin can add another buyer that can then swap funds", async function () {
      this.timeout(0);

      const buyer = context.agents[5];

      await context.ido.tx.admin_add_address({
        address: buyer.address,
      });

      const amount = 1_000_000;

      await context.ido.tx.swap({ amount: `${amount}` }, buyer, undefined, [
        { amount: `${amount}`, denom: "uscrt" },
      ]);
    });

  it("Admin can refund amounts after the sale has ended", async function () {
    this.timeout(0);

    context.ido1 = await context.agent.instantiate(
      new IDO({
        codeId: context.idoInfo.id,
        label: `ido-${parseInt(Math.random() * 100000)}`,
        initMsg: getIDOInitMsg(
          context,
          parseInt(new Date().valueOf() / 1000),
          parseInt(new Date().valueOf() / 1000) + 20
        ),
      })
    );

    await context.sellingToken.mint(25, undefined, context.ido1.address);

    const balance = await context.sellingToken.balance(
      context.agent.address,
      context.viewkey
    );

    assert.strictEqual(balance, '0');

    const buyer = context.agents[1];
    const amount = 5_000_000;

    await context.ido.tx.swap({ amount: `${amount}` }, buyer, undefined, [
      { amount: `${amount}`, denom: "uscrt" },
    ]);

    await (new Promise((ok) => setTimeout(ok, 60000)));

    await context.ido.tx.admin_refund();

    const balanceAfter = await context.sellingToken.balance(
      context.agent.address,
      context.viewkey
    );

    assert.strictEqual(balanceAfter, '20');
  });

  after(async function cleanupAll() {
    this.timeout(0);
    await context.node.terminate();
  });
});
