import debug from "debug";
import { assert } from "chai";
import { randomBytes } from "crypto";
import { Scrt, gas } from "@fadroma/scrt";

import { abs } from "../ops/index";

import Launchpad from "./Launchpad";
import SNIP20 from "./SNIP20";
import IDO from "./IDO";
import Factory from "./Factory";

const log = function () {
  debug("out")(JSON.stringify(arguments, null, 2));
};

describe("Launchpad", () => {
  const fees = {
    upload: gas(10000000),
    init: gas(100000000),
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
    const [tokenBinary, launchpadBinary, factoryBinary] =
      await Promise.all([
        builder.build({ workspace, crate: "amm-snip20" }),
        builder.build({ workspace, crate: "launchpad" }),
        builder.build({ workspace, crate: "factory" }),
      ]);

    const T2 = +new Date();
    console.debug(`building took ${T2 - T1}msec`);

    // upload the contracts
    const { codeId: tokenCodeId, originalChecksum: tokenCodeHash } =
      await builder.uploadCached(tokenBinary);
    await agent.nextBlock;

    const { codeId: launchpadCodeId, originalChecksum: launchpadCodeHash } =
      await builder.uploadCached(launchpadBinary);
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
    await context.node.terminate();
  });
});
