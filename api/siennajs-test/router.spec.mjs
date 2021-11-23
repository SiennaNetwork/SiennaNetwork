import debug from "debug";
import { assert } from "chai";
import { randomBytes } from "crypto";
import { Scrt, ScrtGas } from "@fadroma/scrt";

import { Exchange } from "../Exchange";
import { SwapRouter } from "../Router";
import { Factory } from "../Factory";
import { SNIP20, LPToken } from "../SNIP20";

import * as siennajs from "../siennajs/index";

const Assembler = siennajs.default.hop.Assembler;
const RouterContract = siennajs.default.router.RouterContract;
const TokenTypeAmount = siennajs.default.core.TokenTypeAmount;

const log = function () {
  debug("out")(JSON.stringify(arguments, null, 2));
};

describe("Router", () => {
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
      LPToken: new LPToken(),
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

    await initTokens(context);
    await initFactory(context);

    context.router = new SwapRouter({
      codeId: context.templates.SwapRouter.codeId,
      label: `router-${parseInt(Math.random() * 100000)}`,
      initMsg: {
        register_tokens: [
          context.tokenA.tokenTypeData(),
          context.tokenB.tokenTypeData(),
          context.tokenC.tokenTypeData(),
          context.tokenD.tokenTypeData(),
          context.tokenE.tokenTypeData(),
          context.tokenF.tokenTypeData(),
          context.tokenG.tokenTypeData(),
          context.tokenH.tokenTypeData(),
        ]
      },
    });
    await context.router.instantiate(context.agent);
        
    context.siennaRouter = new RouterContract(context.router.address, context.agent.API);
  });

  it("Has instantiated router successfully", async function () {
    this.timeout(0);
  });

  it("Generate exchange path and try to do the exchange", async function () {
    this.timeout(0);

    await context.tokenA.mint(100);

    const A = { custom_token: { contract_addr: context.tokenA.address, token_code_hash: context.tokenA.codeHash } };
    const B = { custom_token: { contract_addr: context.tokenD.address, token_code_hash: context.tokenD.codeHash } };

    // Generate hops using the assembler, we will have to push the pairs we have in it, and then
    // give it A and B to generate hops that will be then sent to the router for a swap
    const hops = new Assembler(context.pairs).from(A).to(B).get_tree().into_hops();
    console.log(JSON.stringify(hops, null, 2))

    const res = await context.siennaRouter.exec().swap(hops, "10", context.agent.address);
    console.log(JSON.stringify(res, null, 2));
    // await context.tokenA.send(context.router.address, '10', { hops, to: context.agent.address } );

    const balanceA = await context.tokenA.balance(context.agent.address, context.viewkeyA);
    const balanceD = await context.tokenD.balance(context.agent.address, context.viewkeyD);

    assert.strictEqual(parseInt(balanceA), 90);
    assert.strictEqual(parseInt(balanceD), 10);
  });

  // it("Do the exchange path in reverse", async function () {
  //   this.timeout(0);

  //   await context.tokenH.mint(100);

  //   const balanceA1 = parseInt(await context.tokenA.balance(context.agent.address, context.viewkeyA));
  //   const balanceH1 = parseInt(await context.tokenH.balance(context.agent.address, context.viewkeyH));

  //   const H = { custom_token: { contract_addr: context.tokenH.address, token_code_hash: context.tokenH.codeHash } };
  //   const A = { custom_token: { contract_addr: context.tokenA.address, token_code_hash: context.tokenA.codeHash } };

  //   const hops = new Assembler(context.pairs).from(H).to(A).get_tree().into_hops();

  //   await context.tokenH.send(context.router.address, '10', { hops, to: context.agent.address } );

  //   const balanceA2 = parseInt(await context.tokenA.balance(context.agent.address, context.viewkeyA));
  //   const balanceH2 = parseInt(await context.tokenH.balance(context.agent.address, context.viewkeyH));

  //   const totalBalanceA = balanceA2 - balanceA1;

  //   assert.strictEqual(parseInt(totalBalanceA) > 10, true);
  //   assert.strictEqual(parseInt(balanceH2), 90);
  // });

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
      name: "TokenA",
      symbol: "TKNA",
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
      name: "TokenB",
      symbol: "TKNB",
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
      name: "TokenC",
      symbol: "TKNC",
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
      name: "TokenD",
      symbol: "TKND",
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
      name: "TokenE",
      symbol: "TKNE",
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
      name: "TokenF",
      symbol: "TKNF",
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
      name: "TokenG",
      symbol: "TKNG",
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
      name: "TokenH",
      symbol: "TKNH",
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

  const intoPairInfo = function (response) {
    let A = { Scrt: {} };
    if (response.token_0.custom_token) {
      A = { custom_token: { contract_addr: response.token_0.custom_token.contract_addr, token_code_hash: response.token_0.custom_token.token_code_hash } };
    }
    let B = { Scrt: {} };
    if (response.token_1.custom_token) {
      B = { custom_token: { contract_addr: response.token_1.custom_token.contract_addr, token_code_hash: response.token_1.custom_token.token_code_hash } };
    }

    return {
      A,
      B,
      pair_address: response.exchange.address,
      pair_code_hash: context.templates.Exchange.codeHash,
    };
  }

  context.factory = new Factory({
    codeId: context.templates.Factory.codeId,
    label: `factory-${parseInt(Math.random() * 100000)}`,
    EXCHANGE: context.templates.Exchange,
    AMMTOKEN: context.templates.SNIP20,
    LPTOKEN: context.templates.LPToken,
    ROUTER: context.templates.SwapRouter,
    IDO: context.templates.SNIP20, // Dummy
    LAUNCHPAD: context.templates.SNIP20, // Dummy
  });
  await context.factory.instantiate(context.agent);

  context.AB = intoPairInfo(await context.factory.createExchange(
    tokenIntoTokenType(context.tokenA),
    tokenIntoTokenType(context.tokenB),
  ));
  await context.tokenA.mint(100, undefined, context.AB.pair_address);
  await context.tokenB.mint(100, undefined, context.AB.pair_address);

  context.BC = intoPairInfo(await context.factory.createExchange(
    tokenIntoTokenType(context.tokenB),
    tokenIntoTokenType(context.tokenC),
  ));
  await context.tokenB.mint(100, undefined, context.BC.pair_address);
  await context.tokenC.mint(100, undefined, context.BC.pair_address);

  context.CD = intoPairInfo(await context.factory.createExchange(
    tokenIntoTokenType(context.tokenC),
    tokenIntoTokenType(context.tokenD),
  ));
  await context.tokenC.mint(100, undefined, context.CD.pair_address);
  await context.tokenD.mint(100, undefined, context.CD.pair_address);

  context.DE = intoPairInfo(await context.factory.createExchange(
    tokenIntoTokenType(context.tokenD),
    tokenIntoTokenType(context.tokenE),
  ));
  await context.tokenD.mint(100, undefined, context.DE.pair_address);
  await context.tokenE.mint(100, undefined, context.DE.pair_address);

  context.EF = intoPairInfo(await context.factory.createExchange(
    tokenIntoTokenType(context.tokenE),
    tokenIntoTokenType(context.tokenF),
  ));
  await context.tokenE.mint(100, undefined, context.EF.pair_address);
  await context.tokenF.mint(100, undefined, context.EF.pair_address);

  context.FG = intoPairInfo(await context.factory.createExchange(
    tokenIntoTokenType(context.tokenF),
    tokenIntoTokenType(context.tokenG),
  ));
  await context.tokenF.mint(100, undefined, context.FG.pair_address);
  await context.tokenG.mint(100, undefined, context.FG.pair_address);

  context.GH = intoPairInfo(await context.factory.createExchange(
    tokenIntoTokenType(context.tokenG),
    tokenIntoTokenType(context.tokenH),
  ));
  await context.tokenG.mint(100, undefined, context.GH.pair_address);
  await context.tokenH.mint(100, undefined, context.GH.pair_address);

  context.pairs = [
    context.AB,
    context.BC,
    context.CD,
    context.DE,
    context.EF,
    context.FG,
    context.GH,
  ];

  console.log(context.pairs);
}