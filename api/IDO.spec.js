import { randomBytes } from "crypto";
import { SecretNetwork } from "@fadroma/scrt-agent";
import { gas } from "@fadroma/scrt-agent/gas.js";
import { abs } from "../ops/lib/index.js";
import SNIP20 from "./SNIP20.js";
import IDO from "./IDO.js";
import Factory from "./Factory.js";
import debug from "debug";

const log = debug("out");

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
      token: { id: tokenCodeId, code_hash: tokenCodeHash },
      ido: { id: idoCodeId, code_hash: idoCodeHash },
      factory: { id: factoryCodeId, code_hash: factoryCodeHash },
    });
  });

  beforeEach(async function setupEach() {
    this.timeout(0);
    context.factory = await context.agent.instantiate(
      new Factory({
        codeId: context.factory.id,
        label: `factory-${parseInt(Math.random() * 100000)}`,
        initMsg: {
          snip20_contract: context.token,
          lp_token_contract: context.token,
          pair_contract: context.token,
          ido_contract: context.ido,
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
        codeId: context.token.id,
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

    // context.buyingToken = await context.agent.instantiate(
    //   new SNIP20({
    //     codeId: context.token.id,
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
        codeId: context.ido.id,
        label: `ido-${parseInt(Math.random() * 100000)}`,
        initMsg: {
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
            start_time: null,
            end_time: parseInt(new Date().valueOf() / 1000 + 60 * 60), // after one minute
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
        },
      })
    );

    await context.sellingToken.mint(25, undefined, context.ido.address);
  });

  it("Does something", async function () {
    const amount = 1_000_000;
    this.timeout(0);
    const buyer = context.agents[1];

    const res = await context.ido.tx.swap(
      { amount: `${amount}` },
      buyer,
      undefined,
        [{ amount: `${amount}`, denom: "uscrt" }],
      "some-fees"
    );
    log(res);
  });

  after(async function cleanupAll() {
    this.timeout(0);
    await context.node.terminate();
  });
});
