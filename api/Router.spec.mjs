import debug from "debug";
import { assert } from "chai";
import { randomBytes } from "crypto";
import { Scrt, ScrtGas } from "@fadroma/scrt";

import { Exchange } from "./Exchange";
import { SwapRouter } from "./Router";
import { Factory } from "./Factory";
import { SNIP20 } from "./SNIP20";

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
      SwapRouter: new SwapRouter(),
      Factory: new Factory(),
      Exchange: new Exchange(),
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

    await initTokens(context);
    await initFactory(context);

    context.router = new SwapRouter({
      codeId: context.templates.SwapRouter.codeId,
      label: `router-${parseInt(Math.random() * 100000)}`,
      initMsg: {
        register_tokens: [
          context.tokenA.snip20Data(),
          context.tokenB.snip20Data(),
          context.tokenC.snip20Data(),
          context.tokenD.snip20Data(),
          context.tokenE.snip20Data(),
          context.tokenF.snip20Data(),
          context.tokenG.snip20Data(),
          context.tokenH.snip20Data(),
        ]
      },
    });
    await context.router.instantiate(context.agent);
  });

  it("Has instantiated router successfully", async function () {
    this.timeout(0);
  });

  after(async function cleanupAll() {
    this.timeout(0);
    await context.node.terminate();
  });
});

async function initTokens(context) {
  context.tokenA = new SNIP20({
    codeId: context.templates.SNIP20.codeId,
    codeHash: context.templates.SNIP20.codeHash,
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
  await context.tokenA.instantiate(context.agent);
  context.viewkeyA = (await context.tokenA.createViewingKey(context.agent)).key;

  context.tokenB = new SNIP20({
    codeId: context.templates.SNIP20.codeId,
    codeHash: context.templates.SNIP20.codeHash,
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
  await context.tokenB.instantiate(context.agent);
  context.viewkeyB = (await context.tokenB.createViewingKey(context.agent)).key;

  context.tokenC = new SNIP20({
    codeId: context.templates.SNIP20.codeId,
    codeHash: context.templates.SNIP20.codeHash,
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
  await context.tokenC.instantiate(context.agent);
  context.viewkeyC = (await context.tokenC.createViewingKey(context.agent)).key;

  context.tokenD = new SNIP20({
    codeId: context.templates.SNIP20.codeId,
    codeHash: context.templates.SNIP20.codeHash,
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
  await context.tokenD.instantiate(context.agent);
  context.viewkeyD = (await context.tokenD.createViewingKey(context.agent)).key;

  context.tokenE = new SNIP20({
    codeId: context.templates.SNIP20.codeId,
    codeHash: context.templates.SNIP20.codeHash,
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
  await context.tokenE.instantiate(context.agent);
  context.viewkeyE = (await context.tokenE.createViewingKey(context.agent)).key;

  context.tokenF = new SNIP20({
    codeId: context.templates.SNIP20.codeId,
    codeHash: context.templates.SNIP20.codeHash,
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
  await context.tokenF.instantiate(context.agent);
  context.viewkeyF = (await context.tokenF.createViewingKey(context.agent)).key;

  context.tokenG = new SNIP20({
    codeId: context.templates.SNIP20.codeId,
    codeHash: context.templates.SNIP20.codeHash,
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
  await context.tokenG.instantiate(context.agent);
  context.viewkeyG = (await context.tokenG.createViewingKey(context.agent)).key;

  context.tokenH = new SNIP20({
    codeId: context.templates.SNIP20.codeId,
    codeHash: context.templates.SNIP20.codeHash,
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
  await context.tokenH.instantiate(context.agent);
  context.viewkeyH = (await context.tokenH.createViewingKey(context.agent)).key;
}

async function initFactory(context) {
  const tokenIntoTokenType = function (token) {
    return { custom_token: { contract_addr: token.address, token_code_hash: token.codeHash } };
  }
  
  context.factory = new Factory({
      codeId: context.templates.Factory.codeId,
      label: `factory-${parseInt(Math.random() * 100000)}`,
      EXCHANGE: context.templates.Exchange,
      AMMTOKEN: context.templates.SNIP20,
      LPTOKEN: context.templates.SNIP20,
      IDO: context.templates.SNIP20, // Dummy
      LAUNCHPAD: context.templates.SNIP20, // Dummy
    });
  await context.factory.instantiate(context.agent);
  
  context.AB = await context.factory.createExchange(
    tokenIntoTokenType(context.tokenA),
    tokenIntoTokenType(context.tokenB),
  );
  
  context.BC = await context.factory.createExchange(
    tokenIntoTokenType(context.tokenB),
    tokenIntoTokenType(context.tokenC),
  );
  
  context.CD = await context.factory.createExchange(
    tokenIntoTokenType(context.tokenC),
    tokenIntoTokenType(context.tokenD),
  );
  
  context.DE = await context.factory.createExchange(
    tokenIntoTokenType(context.tokenD),
    tokenIntoTokenType(context.tokenE),
  );
  
  context.EF = await context.factory.createExchange(
    tokenIntoTokenType(context.tokenE),
    tokenIntoTokenType(context.tokenF),
  );
  
  context.FG = await context.factory.createExchange(
    tokenIntoTokenType(context.tokenF),
    tokenIntoTokenType(context.tokenG),
  );
  
  context.GH = await context.factory.createExchange(
    tokenIntoTokenType(context.tokenG),
    tokenIntoTokenType(context.tokenH),
  );
}