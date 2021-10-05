import debug from "debug";
import { assert } from "chai";
import { randomBytes } from "crypto";
import { Scrt, ScrtGas } from "@fadroma/scrt";

import { Launchpad } from "./Launchpad";
import { SNIP20 } from "./SNIP20";
import { Factory } from "./Factory";
import { IDO } from "./IDO";
import { generateMnemonic } from "bip39";

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
    const numberOfAgents = 100;
    const identities = [...Array(numberOfAgents)].map((_, i) => `Agent${i}`);
    identities.unshift("ADMIN");
    context.chain = await Scrt.localnet_1_0({ identities }).init();
    context.node = context.chain.node;
    context.agent = await context.chain.getAgent(
      context.node.genesisAccount("ADMIN")
    );

    const agents = (context.agents = (
      await Promise.all(
        identities.map((name, i) => {
          if (name === "ADMIN") return Promise.resolve(null);
          return context.chain.getAgent(context.node.genesisAccount(name));
        })
      )
    ).filter((i) => i !== null));

    console.log({ agents });
    context.agent.API.fees = fees;

    const T1 = +new Date();
    console.debug(`connecting took ${T1 - T0}msec`);

    context.templates = {
      SNIP20: new SNIP20(),
      Launchpad: new Launchpad(),
      Factory: new Factory(),
      IDO: new IDO(),
    };

    for (const template in context.templates) {
      console.debug(`Buidling ${template}`);
      await context.templates[template].build();
    }

    const T2 = +new Date();
    console.debug(`building took ${T2 - T1}msec`);

    // upload the contracts
    for (const template in context.templates) {
      console.debug(`Uploading ${template}`);
      await context.templates[template].upload(context.agent);
      await context.agent.nextBlock;
    }

    const T3 = +new Date();
    console.debug(`uploading took ${T3 - T2}msec`);

    context.factory = new Factory({
      codeId: context.templates.Factory.codeId,
      label: `factory-${parseInt(Math.random() * 100000)}`,
      initMsg: {
        prng_seed: randomBytes(36).toString("hex"),
        snip20_contract: {
          id: context.templates.SNIP20.codeId,
          code_hash: context.templates.SNIP20.codeHash,
        },
        lp_token_contract: {
          id: context.templates.SNIP20.codeId,
          code_hash: context.templates.SNIP20.codeHash,
        },
        pair_contract: {
          id: context.templates.SNIP20.codeId,
          code_hash: context.templates.SNIP20.codeHash,
        },
        launchpad_contract: {
          id: context.templates.Launchpad.codeId,
          code_hash: context.templates.Launchpad.codeHash,
        },
        ido_contract: {
          id: context.templates.IDO.codeId,
          code_hash: context.templates.IDO.codeHash,
        }, // dummy so we don't have to build it
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
            bounding_period: 0,
          },
          {
            token_type: {
              custom_token: {
                contract_addr: context.token.init.address,
                token_code_hash: context.templates.SNIP20.codeHash,
              },
            },
            segment: "25",
            bounding_period: 0,
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

    context.sellingToken = new SNIP20({
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
    await context.sellingToken.instantiate(context.agent);

    let i = 1;
    for (const a of context.agents) {
      console.debug(
        `Minting token, locking token and SCRT ${i} of ${context.agents.length}`
      );
      await context.token.mint(100_000_000, context.admin, a.address);
      await context.token.lockLaunchpad(context.launchpad.address, 100, a);
      await context.launchpad.lock(100, "uscrt", a);
      i++;
    }

    const T4 = +new Date();
    console.debug(`preparing contracts took ${T4 - T3}msec`);
    console.debug(`total preparation time: ${T4 - T0}msec`);
  });

  it("Benchmark the gas usage when IDO does a query call to get whitelist", async function () {
    this.timeout(0);

    const beforeInitBalance = await context.agent.balance;

    context.ido = new IDO({
      codeId: context.templates.IDO.codeId,
      label: `ido-${parseInt(Math.random() * 100000)}`,
      initMsg: {
        info: {
          input_token: {
            native_token: {
              denom: "uscrt",
            },
          },
          rate: "1",
          sold_token: {
            address: context.sellingToken.address,
            code_hash: context.templates.SNIP20.codeHash,
          },
          whitelist: [],
          max_seats: 100,
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
        launchpad: {
          launchpad: {
            address: context.launchpad.address,
            code_hash: context.templates.Launchpad.codeHash,
          },
          tokens: [null, context.token.address],
        },
      },
    });
    await context.ido.instantiate(context.agent);

    const afterInitBalance = await context.agent.balance;
    context.results1 = beforeInitBalance - afterInitBalance;

    console.debug(
      "QUERY BY ITSELF:",
      `Balance before: ${beforeInitBalance}, Balance after: ${afterInitBalance}, spent on init: ${
        beforeInitBalance - afterInitBalance
      }`
    );
  });

  it("Benchmark gas usage when IDO is given a whitelist", async function () {
    this.timeout(0);

    // const { drawn_addresses: whitelist } = await context.launchpad.draw(100, [
    //   null,
    //   context.token.address,
    // ]);

    const whitelist = [];
    const beforeInitBalance = await context.agent.balance;

    


    context.ido = new IDO({
      codeId: context.templates.IDO.codeId,
      label: `ido-${parseInt(Math.random() * 100000)}`,
      initMsg: {
        info: {
          input_token: {
            native_token: {
              denom: "uscrt",
            },
          },
          rate: "1",
          sold_token: {
            address: context.sellingToken.address,
            code_hash: context.templates.SNIP20.codeHash,
          },
          whitelist,
          max_seats: 100,
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
      },
    });
    await context.ido.instantiate(context.agent);

    const afterInitBalance = await context.agent.balance;
    context.results2 = beforeInitBalance - afterInitBalance;

    console.debug(
      "GETS WHITELIST:",
      `Balance before: ${beforeInitBalance}, Balance after: ${afterInitBalance}, spent on init: ${
        beforeInitBalance - afterInitBalance
      }`
    );

    console.debug(
      `Query on its own: ${context.results1}, gets whitelist: ${
        context.results2
      }. Difference: ${context.results1 - context.results2}`
    );
  });

  after(async function cleanupAll() {
    this.timeout(0);
    await context.node.terminate();
  });
});
