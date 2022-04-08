# Sienna Scripts: Deploy

> Run me with `pnpm deploy` or `pnpm -w deploy`

```typescript
import Fadroma, {
    bold,
    colors,
    timestamp,
    Console,
    print,
    randomHex,
    MigrationContext,
    Deployments,
    Uploads,
    Chain,
    Scrt_1_2,
} from '@hackbg/fadroma';

const console = new Console('@sienna/scripts/Deploy');

import * as API from '@sienna/api';
import getSettings, { ONE_SIENNA } from '@sienna/settings';
import { refs, getSources } from './Build';
```

<table><tr><td valign="top">

## Command system overview

This script manages the deployments using the Fadroma command system,
implemented in `@fadroma/cli`.

> See also: [@fadroma/cli/index.ts](https://github.com/hackbg/fadroma/blob/v100/packages/cli/index.ts)

</td><td valign="top">

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

## Deployment system overview

The Sienna platform consists of multiple smart contracts that
depend on each other's existence and configuration. A group of
such contracts is called a **Deployment**.

The `Deployment` is represented by a `.yml` file
under `receipts/$FADROMA_CHAIN/deployments/`.
The deployment file contains **Receipts** -
snippets of YAML containing info about each contract.

> See also: [@fadroma/ops/Deploy.ts](https://github.com/hackbg/fadroma/blob/v100/packages/ops/Deploy.ts)

</td><td valign="top">

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

### Building and uploading

Each deploy command in this file begins by invoking
three pre-defined steps from Fadroma that populate
the `chain`, `agent`, `builder`, and `uploader` keys
of the `MigrationContext` for subsequent command steps.

The `chain` and `agent` are taken from the environment (or `.env` file).

> Set the `FADROMA_CHAIN` environment variable to choose between
> `Scrt_1_2_Devnet`, `Scrt_1_2_Testnet` and `Scrt_1_2_Mainnet`
> as the target of these commands.

The `builder` and `uploader` objects allow the source code of the contracts
to be reproducibly compiled and uploaded to the selected blockchain.

Builds create `.wasm` and `.wasm.sha256` files under `artifacts/`.
If a `.wasm` file for a contract is present, building that
contract becomes a no-op.

> Set the `FADROMA_BUILD_ALWAYS` environment variable to always rebuild
> the contracts.

> See also: [@fadroma/ops/Build.ts](https://github.com/hackbg/fadroma/blob/v100/packages/ops/Build.ts)

Uploads create `.json` files under `receipts/$FADROMA_CHAIN/uploads`.
If a upload receipt's code hash matches the one in the `.wasm.sha256`
for the corresponding contract, the upload becomes a no-op.

> Set the `FADROMA_UPLOAD_ALWAYS` environment variable to always reupload
> the compiled contracts.

> See also: [@fadroma/ops/Upload.ts](https://github.com/hackbg/fadroma/blob/v100/packages/ops/Upload.ts)

</td><td valign="top">

```typescript
const canBuildAndUpload = [
    Fadroma.Chain.FromEnv,
    Fadroma.Build.Scrt_1_2,
    Fadroma.Upload.FromFile,
];
```

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

### Deployment command contexts

Each deploy command may either start a new Deployment, or
append to the one that is currently selected. This is represented by the
`Fadroma.Deploy.New` and `Fadroma.Deploy.Append` pre-defined build steps.
Invoking either of them populates the `deployment` and `prefix` keys
in the `MigrationContext` for subsequent steps.

-   Use `Fadroma.Deploy.New` when you want to start from a clean slate.
    It will create a new deployment under `/receipts/$FADROMA_CHAIN/$TIMESTAMP`.

-   Use `Fadroma.Deploy.Append` when you want to add contracts to an
    existing deployment.

</td><td valign="top">

```typescript
Fadroma.command('new', Fadroma.Deploy.New);

const inNewDeployment = [...canBuildAndUpload, Fadroma.Deploy.New];

Fadroma.command('select', Fadroma.Deploy.Select);

const inCurrentDeployment = [...canBuildAndUpload, Fadroma.Deploy.Append];
```

</td></tr></table>

## Pre-confiured command steps

<table><tr><td valign="top">

This is a collection of shorthands for pre-configured procedures,
out of which the [deployment commands](#deployment-command-definitions) are composed.

The `function` syntax is used here to give proper `name`s
to the pre-configured procedures, so that they can be printed
to the console by the command runner.

The implementations of these procedures follow below.
Thank Eich for **hoisting**!

```typescript
const Sienna = {};
```

</td><td valign="top">

### Deploying

```typescript
Sienna.Deploy = {};

Sienna.Deploy.TGE = function deployTGE_HEAD({ run }) {
    return run(deployTGE, { version: 'vested' });
};

Sienna.Deploy.TGE.tge = function deployTGE_legacy({ run }) {
    return run(deployTGE, { version: 'legacy' });
};
Sienna.Deploy.TGE.vested = function deployTGE_vested({ run }) {
    return run(deployTGE, { version: 'vested' });
};

Sienna.Deploy.AMM = function deployAMM_HEAD({ run }) {
    return run(deployAMM, {
        ammVersion: 'v2',
        ref: 'HEAD',
    });
};
Sienna.Deploy.AMM.v1 = function deployAMM_v1({ run }) {
    return run(deployAMM, {
        ammVersion: 'v1',
    });
};
Sienna.Deploy.AMM.v2 = function deployAMM_v2({ run }) {
    return run(deployAMM, {
        ammVersion: 'v2',
    });
};

Sienna.Deploy.Router = deployRouter;

Sienna.Deploy.Rewards = function deployRewards_HEAD({ run }) {
    return run(deployRewards, {
        version: 'v3',
        adjustRPT: true,
        ref: 'HEAD',
    });
};
Sienna.Deploy.Rewards.v2 = function deployRewards_v2({ run }) {
    return run(deployRewards, {
        version: 'v2',
        adjustRPT: true,
    });
};
Sienna.Deploy.Rewards.v3 = function deployRewards_v3({ run }) {
    return run(deployRewards, {
        version: 'v3',
        adjustRPT: true,
    });
};

Sienna.Deploy.Lend = deployLend;
```

</td><td valign="top">

### Upgrading

```typescript
Sienna.Upgrade = {};

Sienna.Upgrade.AMM = {};
Sienna.Upgrade.AMM.v1_to_v2 = function upgradeAMM_v1_to_v2({ run }) {
    return run(upgradeAMM, {
        vOld: 'v1',
        vNew: 'v2',
    });
};

Sienna.Upgrade.AMM.Factory = {};
Sienna.Upgrade.AMM.Factory.v1_to_v2 = upgradeAMMFactory_v1_to_v2;

Sienna.Upgrade.AMM.Exchanges = {};
Sienna.Upgrade.AMM.Exchanges.v1_to_v2 = cloneAMMExchanges_v1_to_v2;

Sienna.Upgrade.Rewards = {};
Sienna.Upgrade.Rewards.v2_to_v3 = function upgradeRewards_v2_to_v3({ run }) {
    return run(upgradeRewards, {
        vOld: 'v2',
        vNew: 'v3',
    });
};
```

</td></tr></table>

> See: [Implementations of deployment and upgrade procedures](#implementations-of-deployment-and-upgrade-procedures)

## Deploying individual stages of the project

<table><tr><td valign="top">

### Deploying the Sienna TGE

> Run with: `pnpm deploy tge`

```typescript
Fadroma.command('tge legacy', ...inNewDeployment, Sienna.Deploy.TGE.tge);
Fadroma.command('tge vested', ...inNewDeployment, Sienna.Deploy.TGE.vested);
```

**The Sienna TGE (Token Generation Event)** is the
core of the Sienna Platform. It contains a token (SIENNA)
and two vesting contracts:

-   one with a complex, permanent schedule **(MGMT, short for Management)**
-   and one with a simple, reconfigurable schedule **(RPT, short for Remaining Pool Tokens)**.

Its deployment procedure takes the following parameters:

```typescript
type TGEAPIVersion = 'legacy' | 'vested';

export type TGEDeployOptions = {
    /** Address of the admin. */
    admin: string;
    /** Which version/type to deploy **/
    version: TGEAPIVersion;
    /** The schedule for the new MGMT.
     * Defaults to production schedule. */
    settings?: { schedule?: typeof settings.schedule };
};
```

And adds the following items to the migration context:

```typescript
export type TGEDeployResult = {
    /** The deployed SIENNA SNIP20 token contract. */
    SIENNA: API.Snip20Client;

    /** The deployed MGMT contract. */
    MGMT: API.MGMTClient;
    /** The deployed RPT contract. */
    RPT: API.RPTClient;
};
```

This will create a new deployment
under `/receipts/$FADROMA_CHAIN/$TIMESTAMP`,
and deploy just the TGE contracts.

</td><td valign="top">

```typescript
import { buildTge } from './Build'
import { testers, getRPTAccount } from './Configure'
import { schedule } from '@sienna/settings'

async function initMockTokens(deployment, agent, tokenTemplate,vesting) {

  return deployment.initMany(agent, tokenTemplate, vesting.map((contract) => {
    console.log(`Initing mock token: ${contract.name}`)

    const initMsg = {
      name: `Mock_${contract.name}`,
      symbol: contract.name.toUpperCase(),
      decimals: 18,
      config: {
        public_total_supply: true
      },
      prng_seed: randomHex(36)
    }
    return [contract.name, initMsg]
  }))
}



export async function deployTGE (
  context: MigrationContext & TGEDeployOptions
): Promise<TGEDeployResult> {

  const {
    agent, uploader,
    version,
    deployment, prefix,
    settings: { schedule, vesting } = getSettings(agent.chain.mode)
    admin = agent.address,
  } = context
  const { isTestnet, isDevnet, isMainnet } = agent.chain
  console.log(vesting)

  const [tokenBuild, mgmtBuild, rptBuild] = await buildTge(`TGE_${version}`)
  console.log(tokenBuild)


  const isVestedProduction = isMainnet && version == 'vested'

  const uploads =
        await uploader.uploadMany(isVestedProduction ? [mgmtBuild,rptBuild] : [tokenBuild, mgmtBuild, rptBuild])

    console.log(schedule)

  if(version == 'vested' && !isMainnet) {
    const [tokenTemplate, mgmtTemplate, rptTemplate] = uploads;

    const tokens = await initMockTokens(deployment,agent, tokenTemplate, vesting);

    const mgmtInstances = await deployment.initMany(agent, mgmtTemplate, vesting.map(({ name,schedule }, i) => {
      const tokenInstance = tokens[i];

      const tokenLink = { address: tokenInstance.address, code_hash: tokenInstance.codeHash.toUpperCase() }

      console.log(tokenLink)
      console.log(admin)
      name = `${name}.Mgmt[vested]`

      //TODO: schedule should be set along with the vesting configuration for the token.
      // How to handle testnet/dev schedule?
      const initMsg = {
            admin,
            token: tokenLink,
            prefund: true,
            schedule
      }


      return [name, initMsg]
    }))

    const rptInstances = await deployment.initMany(agent, rptTemplate, vesting.map(({ name, }, i) => {
      const mgmtInstance = mgmtInstances[i];
      const tokenInstance = tokens[i];

      const tokenLink = { address: tokenInstance.address, code_hash: tokenInstance.codeHash }
      const mgmtLink = { address: mgmtInstance.address, code_hash: mgmtInstance.codeHash }

      // TODO: use the schedule relative to the vesting configuration
      const rptAccount = Object.assign(getRPTAccount(schedule), { address: admin })
      const portion = rptAccount.portion_size

      const initMsg = {
        portion,
        distribution: [[admin, portion]]
        token: tokenLink,
        mgmt: mgmtLink
      }

      name = `${name}.Rpt.[vested]`

      return [name, initMsg]
    }))

    const mgmtClients = mgmtInstances.map(result => new API.MGMTClient[version]({...result, agent }))
    const rptClients = rptInstances.map(result => new API.RPTClient[version]({...result, agent }))
    const tokenClients = tokens.map(result => new API.SiennaSnip20Client({...result, agent }))

    return { ...mgmtClients, ...rptClients, ...tokenClients }

  } else if(version == 'legacy' && !isMainnet) {
    const tokenInitMsg = {
      name:      "Sienna",
      symbol:    "SIENNA",
      decimals:  18,
      config:    { public_total_supply: true },
      prng_seed: randomHex(36)
    }
    const tokenInstance = await deployment.init(
      agent, tokenTemplate, 'SIENNA', tokenInitMsg)
  }





}

async function deployMgmt (
  context: MigrationContext & TGEDeployOptions
): Promise<TGEDeployResult> {



}
```

</td></tr><tr><!--spacer--><tr><td valign="top">

### Deploying Sienna Swap (Factory + Exchanges)

```typescript
Fadroma.command('amm', ...inCurrentDeployment, Sienna.Deploy.AMM.v2);

Fadroma.command('amm legacy', ...inCurrentDeployment, Sienna.Deploy.AMM.v1);

Fadroma.command('factory v1', ...inCurrentDeployment, Sienna.Deploy.AMM.v1);
```

This procedure takes the active TGE deployment,
attaches a new AMM Factory to it, and uses
that factory to create the AMM Exchange liquidity pools
configured in the settings, and their LP tokens.

It takes the following parameters:

```typescript
export type AMMDeployOptions = {
    /** The version of the AMM to deploy */
    ammVersion: API.AMMVersion;
};
```

And adds the following items to the migration context:

```typescript
export type AMMDeployResult = {
    /** The deployed AMM Factory */
    FACTORY: API.AMMFactoryClient;
    /** The exchanges that were created */
    EXCHANGES: API.AMMExchangeClient[];
    /** The LP tokens that were created */
    LP_TOKENS: API.LPTokenClient[];
};
```

#### Deploying just the AMM Factory

The factory is the hub of the AMM.
In order to work, it needs to be configured
with the proper contract templates, so this
function builds and uploads those too (if absent).

> See also: [buildAMMTemplates](./Build.ts.md#building-the-templates-for-the-amm-factory).

```typescript
export type AMMFactoryDeployOptions = {
    /** Version of the factory to deploy. */
    version: AMMVersion;
    /** Code id and hash for the factory to deploy */
    template: Template;
    /** Relevant properties from global project config. */
    settings: { amm: { exchange_settings: object } };
    /** Config of new factory - goes into initMsg */
    config: {
        admin: string;
        prng_seed: string;
        exchange_settings: object;
    };
    /** Code ids+hashes of contracts
     * that the new factory can instantiate. */
    templates?: AMMFactoryTemplates;
};
```

#### Deploying just the AMM Exchanges

```typescript
export type AMMExchangesDeployOptions = {
    settings: { swapPairs: string[] };
    knownTokens: any;
    FACTORY: API.AMMFactoryClient;
    ammVersion: API.AMMVersion;
};
```

#### Deploying a single AMM Exchange through the factory

This procedure deploys a new exchange.
If the exchange already exists, it does nothing.
Factory doesn't allow 2 identical exchanges to exist anyway,
as compared by `TOKEN0` and `TOKEN1`.

</td><td valign="top">

```typescript
import { buildAMMTemplates, buildRouter } from './Build'
import * as Tokens from './Tokens'

async function deployAMM (
  context: MigrationContext & AMMDeployOptions
): Promise<AMMDeployResult> {
  const { run, ammVersion, ref } = context
  console.info('deployAMM', { ref })
  const FACTORY =
    await run(deployAMMFactory, { version: ammVersion, ref })
  const { EXCHANGES, LP_TOKENS } =
    await run(deployAMMExchanges, { FACTORY, ammVersion, ref })
  return { FACTORY, EXCHANGES, LP_TOKENS }
}

export async function deployAMMFactory (
  context: MigrationContext & AMMFactoryDeployOptions
): Promise<AMMFactoryClient> {
  // Default settings:
  const {
    agent, deployAgent,
    uploader,
    deployment, prefix, suffix = `+${timestamp()}`,
    settings: { amm: { exchange_settings } } = getSettings(agent.chain.mode),
    version   = 'v2',
    ref       = refs[`AMM_${version}`]
    builder   = new Scrt_1_2.Builder(),
    artifact  = await builder.build(getSources(ref)['factory'])
    template  = await uploader.upload(artifact),
    templates = await buildAMMTemplates(uploader, version, ref),
    config = {
      admin: agent.address,
      prng_seed: randomHex(36),
      exchange_settings
    }
  } = context
  console.info('deployAMMFactory', { ref })
  // If the templates are copied from v1, remove the extra templates
  if (version !== 'v1') {
    delete templates.snip20_contract
    delete templates.ido_contract
    delete templates.launchpad_contract
  }
  // Instantiate the new factory and return a client to it
  const name     = `AMM[${version}].Factory`
  const initMsg  = { ...config, ...templates }
  const instance = await deployment.init(
    deployAgent, template, name, initMsg
  )
  return new API.AMMFactoryClient[version]({
    ...deployment.get(name), agent
  })
}

async function deployAMMExchanges (options: MigrationContext & AMMExchangesDeployOptions) {
  const {
    run, agent, deployment,
    settings: { swapPairs } = getSettings(agent.chain.mode),
    knownTokens = await run(Tokens.getSupported),
    FACTORY,
    ammVersion
  } = options
  if (swapPairs.length > 0) {
    const createdPairs = []
    await agent.bundle().wrap(async bundle=>{
      const agent = FACTORY.agent
      FACTORY.agent = bundle
      const factory = new API.AMMFactoryClient({...FACTORY})
      for (const name of swapPairs) {
        const { token0, token1 } = Tokens.fromPairName(knownTokens, name)
        await factory.createExchange(token0, token1)
        createdPairs.push([token0, token1])
      }
      FACTORY.agent = agent
    })
    const { EXCHANGES } = await run(Receipts.saveCreatedPairs, {
      FACTORY, ammVersion, createdPairs
    })
    return {
      EXCHANGES: EXCHANGES.map(EXCHANGE=>EXCHANGE.EXCHANGE),
      LP_TOKENS: EXCHANGES.map(EXCHANGE=>EXCHANGE.LP_TOKEN)
    }
  }
}

async function deployAMMExchange (options) {
  const {
    agent, deployment, run,
    knownTokens = await run(Tokens.getSupportedTokens),
    FACTORY,
    name,
    ammVersion
  } = options
  const factory   = FACTORY.client(agent)
  const inventory = await factory.getTemplates()
  const { token0, token1 } = Tokens.fromName(knownTokens, name)
  try {
    const { EXCHANGE, LP_TOKEN } =
      await factory.getExchange(token0, token1)
    EXCHANGE.prefix = LP_TOKEN.prefix = deployment.prefix
    console.info(`${bold(name)}: Already exists.`)
    return { EXCHANGE, LP_TOKEN }
  } catch (e) {
    if (e.message.includes("Address doesn't exist in storage")) {
      await factory.createExchange(token0, token1)
      const exchange = await factory.getExchange(token0, token1)
      return Receipts.saveAMMExchange({
        deployment, ammVersion, inventory, exchange
      })
    } else {
      console.error(e)
      throw new Error(
        `${bold(`Factory::GetExchange(${name})`)}: '+
        'not found (${e.message})`
      )
    }
  }
}

export async function deployRouter (
  context: MigrationContext
): Promise {

  const { agent
        , builder
        , uploader, deployAgent
        , deployment, prefix
        } = context

  const [
    routerTemplate,
  ] = await uploader.uploadMany(await buildRouter())

  // Define name for deployed contracts
  const v = 'v2'
  const name = `AMM[${v}].Router`

  // Deploy router
  const router = await deployment.init(
    deployAgent, routerTemplate, name, {})

  // Return clients to the instantiated contracts
  return { router }
}
```

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

### Upgrading the AMM

```typescript
Fadroma.command(
    'amm v1_to_v2_all',
    ...inCurrentDeployment,
    Sienna.Upgrade.AMM.v1_to_v2
);

Fadroma.command(
    'amm v1_to_v2_factory',
    ...inCurrentDeployment,
    Sienna.Upgrade.AMM.Factory.v1_to_v2
);

Fadroma.command(
    'amm v1_to_v2_exchanges',
    ...inCurrentDeployment,
    Sienna.Upgrade.AMM.Exchanges.v1_to_v2
);
```

This procedure takes an existing AMM and
creates a new one with the same contract templates.
Then, recreate all the exchanges from the
old factory in the new one.

It takes the following parameters:

```typescript
export type AMMUpgradeOptions = {
    builder: Builder;
    generateMigration: boolean;
    vOld: API.AMMVersion;
    oldFactoryName: string;
    oldFactory: API.AMMFactoryClient;
    oldExchanges: API.AMMExchangeClient[];
    oldTemplates: any;
    vNew: API.AMMVersion;
    newRef: string;
    newFactoryTemplate: Template;
    name: string;
};
```

And adds the following items to the context:

```typescript
export type AMMUpgradeResult =
    | ScrtBundle
    | {
          // The factory that was created by the upgrade.
          FACTORY: API.AMMFactoryClient;
          // The exchanges that were created by the upgrade.
          EXCHANGES: API.ExchangeInfo[];
          // what about the LP tokens?
      };

type RedeployAMMExchangeOptions = {
    NEW_FACTORY: unknown;
    OLD_EXCHANGES: unknown;
    ammVersion: AMMVersion;
};

type RedeployAMMExchangeResult = {
    NEW_EXCHANGES: unknown;
};
```

</td><td valign="top">

```typescript
import * as Receipts from './Receipts'

async function upgradeAMM (
  context: MigrationContext & AMMUpgradeOptions
): Promise<AMMUpgradeResult> {

  const {
    run, chain, agent,
    deployment, prefix, suffix = `+${timestamp()}`,
    builder = new Scrt_1_2.Builder(),
    uploader

    generateMigration = false,

    // By default, the old factory and its exchanges
    // are automatically retrieved; context still allows
    // them to be passed in manually (for multisig mode?)
    vOld = 'v1',
    oldFactoryName = `AMM[${vOld}].Factory`,
    oldFactory     = new API.AMMFactoryClient[vOld]({
      ...deployment.get(oldFactoryName), agent
    }),
    oldExchanges = await oldFactory.listExchangesFull(),
    oldTemplates = await oldFactory.getTemplates(),

    vNew = 'v2',
    newRef = refs[`AMM_${vNew}`]
    newFactoryTemplate = await uploader.upload(
      await builder.build(getSources(ref)['factory'])
    )
  } = context

  // if we're generating the multisig transactions,
  // skip the queries and store all the txs in a bundle
  let bundle
  if (generateMigration) bundle = agent.bundle()

  // create the new factory instance
  const newFactory = await run(deployAMMFactory, {
    agent:     generateMigration ? bundle : agent,
    version:   vNew,
    template:  newFactoryTemplate,
    templates: oldTemplates,
    suffix
  }) as API.AMMFactoryClient

  // create the new exchanges, collecting the pair tokens
  const newPairs = await newFactory.createExchanges({
    pairs:     oldExchanges,
    templates: oldTemplates
  })

  let newExchanges
  if (!generateMigration) {
    console.log(newPairs.sort())
    newExchanges = await Receipts.saveExchangeReceipts(
      deployment, vNew, newFactory, newPairs
    )
  }

  return generateMigration ? bundle : {
    FACTORY:   newFactory,
    EXCHANGES: newExchanges
  }

}

export async function upgradeAMMFactory_v1_to_v2 (context) {
  const {
    run, deployment, prefix, suffix, clientAgent
  } = context
  const v1: Record<string, any> = {}
  v1.name = `AMM[v1].Factory`
  v1.factory = new API.AMMFactoryClient.v1({
    ...deployment.get(v1.name), agent: clientAgent
  })
  const v2: Record<string, any> = {}
  v2.client  = await run(deployAMMFactory, {
    version: 'v2', suffix
  })
  return { v1, v2 }
}

export async function cloneAMMExchanges_v1_to_v2 (context) {
  const { run, deployment, clientAgent, deployAgent } = context
  const v1: Record<string, any> = {}
  v1.name    = `AMM[v1].Factory`
  v1.factory = new API.AMMFactoryClient.v1({
    ...deployment.get(v1.name), agent: clientAgent
  })
  v1.pairs   = await v1.factory.listExchanges()
  console.info(bold(`AMM v1:`), v1.pairs.length, 'pairs')
  const v2: Record<string, any> = {}
  v2.name      = `AMM[v2].Factory`
  v2.readFactory  = new API.AMMFactoryClient.v2({
    ...deployment.get(v2.name), agent: clientAgent
  })
  v2.templates = await v2.readFactory.getTemplates()
  v2.existing  = await v2.readFactory.listExchanges()
  const existingV1PairsJSON = v1.pairs.map(x=>JSON.stringify(x.pair))
  const existingV2PairsJSON = v2.existing.map(x=>JSON.stringify(x.pair))
  const v2PairsToCreate = []
  for (const v1pairJSON of existingV1PairsJSON) {
    if (existingV2PairsJSON.includes(v1pairJSON)) {
      console.warn(bold(`Pair exists, not creating:`), v1pairJSON)
    } else {
      console.info(bold(`Will create pair:`), v1pairJSON)
      v2PairsToCreate.push({ pair: JSON.parse(v1pairJSON) })
    }
  }
  v2.writeFactory = new API.AMMFactoryClient.v2({
    ...deployment.get(v2.name), agent: deployAgent
  })
  console.log({read: v2.readFactory, write: v2.writeFactory})
  v2.pairs = await v2.writeFactory.createExchanges({
    templates: v2.templates,
    pairs:     v2PairsToCreate
  })
  v2.exchanges = await Receipts.saveExchangeReceipts(
    deployment, 'v2', v2.readFactory, v2.pairs
  )
  return { v1, v2 }
}

async function redeployAMMExchanges (
  context: MigrationContext & RedeployAMMExchangeOptions
): Promise<RedeployAMMExchangeResult> {
  const {
    agent, deployment,
    ammVersion, NEW_FACTORY, OLD_EXCHANGES = [],
  } = context
  // 1. create them in one go
  let NEW_EXCHANGES = []
  await agent.bundle(async agent=>{
    const bundled = NEW_FACTORY.client(agent)
    for (const { name, TOKEN_0, TOKEN_1 } of (OLD_EXCHANGES||[])) {
      const exchange = await bundled.createExchange(TOKEN_0, TOKEN_1)
      NEW_EXCHANGES.push([TOKEN_0, TOKEN_1])
    }
  })
  // 2. get them
  const factory = NEW_FACTORY.client(agent)
  const inventory = await NEW_FACTORY.client(agent).getTemplates()
  // 3. save the receipts
  const save = async ([TOKEN_0, TOKEN_1])=>{
    const exchange = await factory.getExchange(TOKEN_0, TOKEN_1)
    return Receipts.saveAMMExchange({
      deployment, ammVersion, inventory, exchange
    })
  }
  return { NEW_EXCHANGES: await Promise.all(NEW_EXCHANGES.map(save)) }
}
```

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

### Deploying Sienna Rewards

```typescript
Fadroma.command('rewards v2', ...inCurrentDeployment, Sienna.Deploy.Rewards.v2);

Fadroma.command('rewards v3', ...inCurrentDeployment, Sienna.Deploy.Rewards.v3);
```

```typescript
type RewardsDeployOptions = {
    /** Which address will be admin
     * of the new reward pools.
     * Defaults to the executing agent. */
    admin: string;
    /** The reward token.
     * Defaults to SIENNA */
    reward: API.Snip20Client;
    /** Version of the reward pools to deploy. */
    version: RewardsAPIVersion;
    /** CodeId+CodeHash for Rewards[version]. */
    template: Template;
    /** The AMM version to which
     * the rewards will be attached. */
    ammVersion: AMMVersion;
    /** Prevent label clashes when
     * running multiple local deploys. */
    suffix: string;
    /** Whether the new reward pools
     * should be configured in the RPT */
    adjustRPT: boolean;

    settings: {
        /** List of reward pairs to generate. */
        rewardPairs: Record<string, any>;
        timekeeper: string;
    };
};

type RewardsDeployResult = RewardsClient[];
```

</td><td valign="top">

```typescript
import { adjustRPTConfig } from './Configure'
async function deployRewards (
  context: MigrationContext & RewardsDeployOptions
): Promise<RewardsDeployResult> {
  const {
    run,
    agent, uploader, deployAgent, clientAgent,
    deployment, suffix,
    settings: { rewardPairs, timekeeper } = getSettings(agent.chain.mode),
    admin  = agent.address,
    reward = new API.Snip20Client({
      ...deployment.get('SIENNA'),
      agent: clientAgent
    }),
    version  = 'v3' as RewardsAPIVersion,
    ref = refs[`Rewards_${version}`]
    builder = new Scrt_1_2.Builder(),
    template = await uploader.upload(await builder.build(getSources(ref)['sienna-rewards'])),
    ammVersion = { v3: 'v2', v2: 'v1' }[version] as AMMVersion,
    adjustRPT = true
  } = context
  const createPools = Object.entries(rewardPairs)
  const results = await deployment.initMany(deployAgent, template, createPools.map(([name, amount])=>{
    // get the staked token by name
    if (name !== 'SIENNA') name = `AMM[${ammVersion}].${name}.LP`
    const staked = new API.Snip20Client(deployment.get(name))
    // define the name of the reward pool from the staked token
    name = `${name}.Rewards[${version}]`
    return [name, makeRewardsInitMsg[version](reward, staked, admin, timekeeper)]
  }))
  const rptConfig = createPools.map(([name, amount], i)=>[results[i].address, String(BigInt(amount)*ONE_SIENNA)])
  const clients = results.map(result=>new API.RewardsClient[version]({...result, agent: clientAgent}))
  if (adjustRPT) {
    await run(adjustRPTConfig, { RPT_CONFIG: rptConfig })
  }
  return clients
}
```

Rewards v2 and v3 have different APIs,
including different init message formats:

```typescript
const makeRewardsInitMsg = {
    v2(reward, staked, admin) {
        let threshold = 15940;
        let cooldown = 15940;

        const { SIENNA_REWARDS_V2_BONDING } = process.env;
        if (SIENNA_REWARDS_V2_BONDING) {
            console.warn(
                bold('Environment override'),
                'SIENNA_REWARDS_V2_BONDING=',
                SIENNA_REWARDS_V2_BONDING
            );
            threshold = Number(SIENNA_REWARDS_V2_BONDING);
            cooldown = Number(SIENNA_REWARDS_V2_BONDING);
        }

        return {
            admin,
            lp_token: {
                address: staked?.address,
                code_hash: staked?.codeHash,
            },
            reward_token: {
                address: reward?.address,
                code_hash: reward?.codeHash,
            },
            viewing_key: randomHex(36),
            ratio: ['1', '1'],
            threshold,
            cooldown,
        };
    },

    v3(reward, staked, admin, timekeeper) {
        let bonding = 86400;

        const { SIENNA_REWARDS_V3_BONDING } = process.env;
        if (SIENNA_REWARDS_V3_BONDING) {
            console.warn(
                bold('Environment override'),
                'SIENNA_REWARDS_V3_BONDING=',
                SIENNA_REWARDS_V3_BONDING
            );
            bonding = Number(SIENNA_REWARDS_V3_BONDING);
        }

        return {
            admin,
            config: {
                reward_vk: randomHex(36),
                lp_token: {
                    address: staked?.address,
                    code_hash: staked?.codeHash,
                },
                reward_token: {
                    address: reward?.address,
                    code_hash: reward?.codeHash,
                },
                timekeeper,
                bonding,
            },
        };
    },
};
```

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

## Upgrading Sienna Rewards

```typescript
Fadroma.command(
    'rewards v2_to_v3',
    ...inCurrentDeployment,
    Sienna.Upgrade.Rewards.v2_to_v3
);
```

```typescript
type RewardsUpgradeOptions = {
    settings: {
        /** Which address will be admin
         * of the new reward pools.
         * Defaults to the executing agent. */
        admin: string;
        /** Which address can call BeginEpoch
         * on the new reward pools.
         * Defaults to the value of `admin` */
        timekeeper: string;
    };

    /** The reward token.
     * Defaults to SIENNA */
    reward: API.Snip20Client;
    /** Old version that we are migrating from. */
    vOld: API.RewardsAPIVersion;
    /** New version that we are migrating to. */
    vNew: API.RewardsAPIVersion;
    /** Code id and code hash of new version. */
    template: Template;
    /** Version of the AMM that the
     * new reward pools will attach to. */
    newAmmVersion: API.AMMVersion;
};

type RewardsUpgradeResult = {
    REWARD_POOLS: API.RewardsClient[];
};
```

</td><td valign="top">

```typescript
async function upgradeRewards(
    context: MigrationContext & RewardsUpgradeOptions
): Promise<RewardsUpgradeResult> {
    const {
        run,
        chain,
        uploader,
        deployAgent,
        clientAgent,
        timestamp,
        deployment,
        prefix,
        suffix = `+${timestamp}`,
        settings: {
            admin = deployAgent.address,
            timekeeper = admin,
        } = getSettings(chain.mode),

        reward = new API.Snip20Client({
            ...deployment.get('SIENNA'),
            agent: clientAgent,
        }),
        vOld = 'v2',
        vNew = 'v3',
        builder = new Scrt_1_2.Builder(),
        ref = refs[vNew],
        template = await uploader.upload(
            await builder.build(getSources(ref)['sienna-rewards'])
        ),
        newAmmVersion = 'v2',
    } = context;

    // establish client classes
    const OldRewardsClient = API.RewardsClient[vOld];
    const NewRewardsClient = API.RewardsClient[vNew];

    // Collect info about old reward pools
    // (namely, what are their staked tokens)
    const isOldPool = (name) => name.endsWith(`.Rewards[${vOld}]`);
    const oldNames = Object.keys(deployment.receipts).filter(isOldPool);
    const oldReceipts = oldNames.map((name) => deployment.get(name));
    const oldPools = oldReceipts.map(
        (r) => new OldRewardsClient({ ...r, agent: clientAgent })
    );
    const stakedTokens = new Map();
    const stakedTokenNames = new Map();
    await Promise.all(
        oldPools.map(async (pool) => {
            console.info(bold('Getting staked token info for:'), pool.name);
            if (pool.name === 'SIENNA.Rewards[v2]') {
                stakedTokens.set(pool, reward);
                stakedTokenNames.set(reward, 'SIENNA');
            } else {
                const staked = await pool.getStakedToken();
                stakedTokens.set(pool, staked);
                const name = await staked.getPairName();
                stakedTokenNames.set(staked, name);
            }
        })
    );

    // Create new reward pools with same staked tokens as old reward pools
    // WARNING: This might've been the cause of the wrong behavior
    //          of the AMM+Rewards migration; new pools should point to new LP tokens.
    const newPools = await deployment.initMany(
        deployAgent,
        template,
        oldPools.map((oldPool) => {
            const staked = stakedTokens.get(oldPool);
            const name =
                staked.address === deployment.get('SIENNA').address
                    ? `SIENNA.Rewards[${vNew}]`
                    : `AMM[${newAmmVersion}].${stakedTokenNames.get(
                          staked
                      )}.LP.Rewards[${vNew}]`;
            return [
                name,
                makeRewardsInitMsg[vNew](reward, staked, admin, timekeeper),
            ];
        })
    );
    console.log({ newPools });

    // Return clients to new reward pools.
    const newPoolClients = newPools.map(
        (r) => new NewRewardsClient({ ...r, agent: clientAgent })
    );
    return { REWARD_POOLS: newPoolClients };
}
```

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

### Deploying Sienna Lend

> Run with `pnpm deploy lend`

```typescript
Fadroma.command('lend', ...inCurrentDeployment, Sienna.Deploy.Lend);
```

```typescript
type LendInterestModelOptions = {
    base_rate_year: string;
    blocks_year: number;
    jump_multiplier_year: string;
    jump_threshold: string;
    multiplier_year: string;
};

type LendOverseerOptions = {
    entropy: string;
    prng_seed: string;
    close_factor: string;
    premium: string;
};

type LendContracts = {
    OVERSEER: API.LendOverseerClient;
    MARKET: API.LendMarketClient;
    INTEREST_MODEL: API.InterestModelClient;
    ORACLE: API.LendOracleClient;
    MOCK_ORACLE: API.MockOracleClient;
    TOKEN1: API.AMMSnip20Client;
    TOKEN2: API.AMMSnip20Client;
};
```

</td><td valign="top">

```typescript
import { buildLend } from './Build'

export async function deployLend (
  context: MigrationContext & LendInterestModelOptions & LendOverseerOptions
): Promise<LendContracts> {

  // 1. Expand dependencies and settings from context
  const { agent
        , builder
        , uploader, deployAgent
        , deployment, prefix

        // Interest model settings:
        , base_rate_year       =      "0"
        , blocks_year          = 6311520
        , jump_multiplier_year =      "0"
        , jump_threshold       =      "0"
        , multiplier_year      =      "1"

        // Overseer settings:
        , entropy      =  randomHex(36)
        , prng_seed    =  randomHex(36)
        , close_factor =  "0.5"
        , premium      =  "1"
        } = context

  const { isDevnet } = agent.chain

  const [
    interestModelTemplate,
    oracleTemplate,
    marketTemplate,
    overseerTemplate,
    mockOracleTemplate,
    tokenTemplate,
  ] = await uploader.uploadMany(await buildLend())

  // Define names for deployed contracts
  const v = 'v1'
  const names = {
    interestModel: `Lend[${v}].InterestModel`,
    oracle:        `Lend[${v}].Oracle`,
    mockOracle:    `Lend[${v}].MockOracle`,
    overseer:      `Lend[${v}].Overseer`,
    token1:        `Lend[${v}].Placeholder.slATOM`,
    token2:        `Lend[${v}].Placeholder.slSCRT`
  }

  // Deploy placeholder tokens
  const tokenConfig = {
    enable_burn: true,
    enable_deposit: true,
    enable_mint: true,
    enable_redeem: true,
    public_total_supply: true,
  }
  const token1 = await deployment.init(
    deployAgent, tokenTemplate, names.token1, {
      name:     "slToken1",
      symbol:   "SLATOM",
      decimals:  18,
      prng_seed: randomHex(36),
      config:    tokenConfig,
    })
  const token2 = await deployment.init(
    deployAgent, tokenTemplate, names.token2, {
      name:     "slToken2",
      symbol:   "SLSCRT",
      decimals:  18,
      prng_seed: randomHex(36),
      config:    tokenConfig,
    })

  // Create the interest model
  await deployment.init(
    deployAgent, interestModelTemplate, names.interestModel, {
      base_rate_year,
      blocks_year,
      jump_multiplier_year,
      jump_threshold,
      multiplier_year
    })

  // Create the mock oracle
  const mockOracle = await deployment.init(
    deployAgent, mockOracleTemplate, names.mockOracle, {})

  // Create the overseer
  await deployment.init(
    deployAgent, overseerTemplate, names.overseer, {
      entropy, prng_seed, close_factor, premium,
      market_contract: {
        code_hash: marketTemplate.codeHash,
        id:        Number(marketTemplate.codeId)
      },
      oracle_contract: {
        code_hash: oracleTemplate.codeHash,
        id:        Number(oracleTemplate.codeId)
      },
      oracle_source: {
        code_hash: mockOracle.codeHash,
        address:   mockOracle.address
      }
    })

  // Return clients to the instantiated contracts
  return {
    OVERSEER:       new API.LendOverseerClient({
      ...deployment.get(names.overseer),      agent
    })
    INTEREST_MODEL: new API.InterestModelClient({
      ...deployment.get(names.interestModel), agent
    })
    // TODO: get oracle by querying overseer (once this query exists)
    // ORACLE:         new API.LendOracleClient({
    //   ...deployment.get(names.oracle),        agent
    // })
    MOCK_ORACLE:    new API.MockOracleClient({
      ...deployment.get(names.mockOracle),    agent
    })
    TOKEN1:         new API.AMMSnip20Client({
      ...deployment.get(names.token1),        agent
    })
    TOKEN2:         new API.AMMSnip20Client({
      ...deployment.get(names.token2),        agent
    })
  }

}
```

</td></tr></table>

## Deploying the full project in its various incarnations

<table><tr><td valign="top">

### Latest up-to-date deployment

> Run with `pnpm deploy latest`

This is the simplest and fastest way to get up and running.
It deploys the latest development versions of everything.

</td><td valign="top">

```typescript
Fadroma.command(
    'latest',
    ...inNewDeployment,
    Sienna.Deploy.TGE,
    Sienna.Deploy.AMM,
    Sienna.Deploy.Rewards,
    Sienna.Deploy.Router,
    Sienna.Deploy.Lend,
    Sienna.Deploy.Status
);
```

</td></tr><tr><!--spacer--></tr><td valign="top">

### Full historical deployment

> Run with `pnpm deploy all`

This is most faithful to production.
As blockchain deployments are append-only,
this goes through deploying the old versions
of contracts, then upgrading them to the latest
development versions.

</td><td valign="top">

```typescript
Fadroma.command(
    'all',
    ...inNewDeployment,
    Sienna.Deploy.TGE,
    Sienna.Deploy.AMM.v1,
    Sienna.Deploy.Rewards.v2,
    Sienna.Deploy.Router,
    Fadroma.Deploy.Status,
    Sienna.Upgrade.AMM.Factory.v1_to_v2,
    Sienna.Upgrade.AMM.Exchanges.v1_to_v2,
    Sienna.Upgrade.Rewards.v2_to_v3,
    Fadroma.Deploy.Status,
    Sienna.Deploy.Lend,
    Fadroma.Deploy.Status
);
```

</td><tr><!--spacer--></tr><td valign="top">

### Historical deployment of AMM+Rewards on top of existing TGE

> Run with `pnpm deploy sans-tge`

The TGE is the most stable part of the project.
If you have a deployment containing a TGE (such as created by `pnpm deploy tge`)
and want to iterate on deploying the rest, this command makes it faster
by about 20 seconds.

</td><td valign="top">

```typescript
Fadroma.command(
    'sans-tge',
    ...inCurrentDeployment,
    Sienna.Deploy.AMM.v1,
    Sienna.Deploy.Rewards.v2,
    Fadroma.Deploy.Status,
    Sienna.Upgrade.AMM.Factory.v1_to_v2,
    Sienna.Upgrade.AMM.Exchanges.v1_to_v2,
    Sienna.Upgrade.Rewards.v2_to_v3,
    Fadroma.Deploy.Status
);
```

</td><tr><!--spacer--></tr><td valign="top">

### Deploy latest AMM+Rewards on top of existing TGE

This command requires a [selected deployment](#select-the-active-deployment),
to which it adds the contracts for Sienna Swap.

</td><td valign="top">

```typescript
Fadroma.command('amm v2', ...inCurrentDeployment, Sienna.Deploy.AMM.v2);
```

</td></tr><tr><!--spacer--></tr><td valign="top">

### Legacy deployment (circa January 2022)

Use as basis to test the AMM v2 + Rewards v3 upgrade.

</td><td valign="top">

```typescript
Fadroma.command(
    'legacy',
    ...inNewDeployment,
    Sienna.Deploy.TGE,
    Fadroma.Deploy.Status,
    Sienna.Deploy.AMM.v1,
    Fadroma.Deploy.Status,
    Sienna.Deploy.Rewards.v2,
    Fadroma.Deploy.Status
);
```

</td></tr></table>

### Upgrading AMM v1 and Rewards v2 to AMM v2 and Rewards v3

```typescript

```

## Entry point

```typescript
export default Fadroma.module(import.meta.url);
```
