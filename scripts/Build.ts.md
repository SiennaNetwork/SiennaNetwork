# Building the Sienna smart contracts

> **Build command:** [@fadroma/cli/build](https://github.com/hackbg/fadroma/blob/v100/packages/commands/build.ts)
> **Source and Artifact types:** [@fadroma/ops/Core](https://github.com/hackbg/fadroma/blob/v100/packages/ops/Core.ts)
> **General builder logic:** [@fadroma/ops/Build](https://github.com/hackbg/fadroma/blob/v100/packages/ops/Build.ts)
> **Secret Network builder logic:** [@fadroma/scrt/ScrtBuild](https://github.com/hackbg/fadroma/blob/v100/packages/scrt/ScrtBuild.ts)

## Build artifact caching

Builds create pairs of `contract@commit.wasm` and `contract@commit.wasm.sha256` files
under `artifacts/`. The `.wasm` file is called a **build artifact** and the `.wasm.sha256`
file is called an **artifact checksum file**.

**If a corresponding `.wasm` file is present, building that contract from that commit becomes a no-op,
even if the source has changed.**

* **Env var:** `FADROMA_BUILD_ALWAYS`: set this to a non-empty value to always rebuild the contracts.

## Overview and usage

The command **`pnpm -w build`** prints all the available [*build sets*](#build-sets).

> NOTE: If you're at the top level of the workspace, the `-w` is optional.
>       Always make sure you're in the main workspace before trying to build.
>       For example, if you chdir into `deps/fadroma`, it won't work.

A build set is a set of [*build sources*](#build-sources).
Each build source represents the source code of a smart contract
that can be compiled to produce a *build artifact*.
Build artifacts are stored under [`/artifacts/`](../artifacts).

The command **`pnpm -w build something`** asks [Fadroma](https://github.com/hackbg/fadroma/tree/v100/packages/scrt)
to compile the smart contracts from the build set called `something` for
production.

> NOTE: Before building, make sure you have at least **2G of disk space**
>       for the [build container](#how-the-build-works).

Read on for the list of available [build sets](#build-sets) (groups of contracts),
[build sources](#build-sources) (individual contracts),
and an [overview of the build procedure](#how-the-build-works).

## Build sources

Each build set consists of one or more instances of Fadroma's **Source** object.
The Source object points to the source code of a smart contract, referenced by
**workspace**, **crate name**, and, optionally, **Git ref** (commit/branch/tag/etc).

Currently, Fadroma does not enumerate the crates in the project workspace,
nor does it identify which of these are smart contracts and which are libraries,
nor does it have unified support for smart contract versioning and migrations
(though all these features are in the pipeline!).

That's why the following lists of contracts and versions are necessary.

### List of contracts

> PLEASE: Keep these up to date when adding, removing, or renaming contracts.

```typescript
export const contracts = {
  TGE:       [ 'snip20-sienna'
             , 'sienna-mgmt'
             , 'sienna-rpt' ],
  Vesting:   [ 'sienna-mgmt'
             , 'sienna-rpt' ],
  Tokens:    [ 'amm-snip20'
             , 'lp-token' ]
  AMM:       [ 'factory'
             , 'exchange'
             , 'lp-token'
             , 'amm-snip20']
  Lend:      [ 'lend-interest-model'
             , 'lend-market'
             , 'lend-mock-oracle'
             , 'lend-oracle'
             , 'lend-overseer'
             , 'amm-snip20' ]
  Launchpad: [ 'launchpad'
             , 'ido' ]
  Rewards:   [ 'sienna-rewards' ]
  Router:    [ 'router' ]
}

contracts.all = [...new Set([
  ...contracts.TGE,
  ...contracts.Vesting,
  ...contracts.Tokens,
  ...contracts.AMM,
  ...contracts.Launchpad,
  ...contracts.Rewards,
  ...contracts.Router,
  ...contracts.Lend,
])]
```

### List of versions

> PLEASE: Keep this up to date when rolling out new project phases,
>         to maintain deployment reproducibility.

```typescript
export const versions = {
  HEAD:    'HEAD',
  AMM:     { v1: 'legacy/amm-v1', v2: 'legacy/amm-v2-rewards-v3' },
  Rewards: { v2: 'legacy/rewards-v2', v3: 'legacy/amm-v2-rewards-v3' },
  TGE:     { vest: 'HEAD', legacy: 'c34bfbcfe' } // TOOO: Is this the correct commit for legacy deploy? 
}
```

## Build sets

A build set is a function that returns an array of **Source** objects,
representing a list of smart contracts to build.

Build sets are collected in the default export of this module - the `build` object.
To add a new build set, add it like `build['my-build-set'] = () => [/*sources*/]`

```typescript
const build = {}
export default build
```

To add the build sources of the build set, you can use the `sources` helper function:
`build['my-build-set'] = () => sources('git-ref', ['crate-name', 'crate-name'])`.

> PROTIP: You can pass more than one list of crate names;
>         Fadroma's `Source.collect` does the array concatenation for you!

> PLEASE: No magic strings! Take the git ref from the `versions` object,
>         and the crate names from `contracts`.

```typescript
import { Source } from '@hackbg/fadroma'
import { workspace } from '@sienna/settings'
export function source (crate, ref = 'HEAD') {
  if (!contracts.all.includes(crate)) {
    throw new Error(`crate ${crate} not in scripts/Build.ts:contracts.all`)
  }
  return new Source(workspace, crate, ref)
}
export function sources (ref, ...crateLists) {
  return Source.collect(workspace, ref, ...crateLists)
}
```

So, without further ado, here are the currently supported build sets.
You can use these as references when defining your own.

### Building latest everything

The simplest build set consists of all the contracts in the current working tree.
It looks like this:

```typescript
build['latest'] = () => sources(versions.HEAD, contracts.all)
```

Runing **`pnpm -w build latest`** will build the contracts from this build set.

> WARNING: This will build the contracts for production, but will include any
>          non-committed changes from your working tree. Please don't deploy
>          such contracts, as this violates the principle of reproducibility.

### Building the full history of the project

The most complex build configuration builds all the contracts that
were ever in production. This complements `pnpm deploy history`,
enabling you to recreate the whole history of the smart contract system.

```typescript
build['history'] = () => [
  ...sources(versions.AMM.v1,     contracts.AMM, contracts.Launchpad, contracts.Tokens),
  ...sources(versions.Rewards.v2, contracts.Rewards),
  ...sources(versions.AMM.v2,     contracts.AMM, contracts.Tokens),
  ...sources(versions.Rewards.v3, contracts.Rewards)
  ...sources(versions.HEAD,       contracts.all)
]
```

Running **`pnpm -w build history`** will build the contracts from this build set.

> WARNING: This may take a very long time depending on your system. (Docker on Mac - 3hrs)

### Specific build configurations

These configurations represent specific groups of interdependent contracts.
This is useful mainly when you're working on a specific part of the system
and don't want to build unrelated things.

> PLEASE: Add your own build configurations here as needed.

```typescript
build['amm_v1']    = () => sources(versions.AMM.v1, contracts.AMM)
build['amm_v2']    = () => sources(versions.AMM.v2, contracts.AMM)
build['lend']      = () => sources(versions.HEAD, contracts.Lend)
build['router']    = () => sources(versions.HEAD, contracts.AMM, contracts.Router, contracts.Tokens)
build['launchpad'] = () => sources(versions.HEAD, contracts.Launchpad)
build['vesting']   = () => sources(versions.TGE.vest, contracts.TGE)
build['tge']       = () => sources(versions.TGE.legacy, contracts.TGE)
```

## Special case: building the templates for the AMM Factory

This function is used by `deployAMM`, and is an example of using the builder directly.

The AMM factory needs to be configured
with [code ID + code hash] pairs that the factory
uses to instantiate its contracts.

The set of templates supported by the factory
differs between AMMv1 and AMMv2 (v1 contained
some extra templates that weren't being used)

```typescript
import { Scrt_1_2 } from '@hackbg/fadroma'
export async function buildAMMTemplates (
  uploader: Uploader,
  version:  AMMVersion,
  ref:      string  = versions.AMM[version],
  builder:  Builder = Scrt_1_2.getBuilder()
): Promise<Record<string, {id:number,code_hash:string}>> {
  const crates = ['exchange', 'lp-token']
  if (version === 'v1') {
    console.info('Building extra (unused) templates required by AMM Factory v1...')
    crates.push('amm-snip20')
    crates.push('launchpad')
    crates.push('ido')
  }
  const srcs = sources(ref, ['exchange', 'lp-token'])
  const artifacts = await builder.buildMany(sources(ref, crates))
  const templates = []
  for (const artifact of artifacts) {
    templates.push(await uploader.upload(artifact))
  }
  const formatTemplate = ({ codeId, codeHash }) => ({
    id: Number(codeId), code_hash: codeHash
  })
  const formattedTemplates = {
    pair_contract:      formatTemplate(templates[0]),
    lp_token_contract:  formatTemplate(templates[1]),
  }
  if (version === 'v1') {
    Object.assign(formattedTemplates, {
      snip20_contract:    formatTemplate(templates[2]),
      ido_contract:       formatTemplate(templates[3]),
      launchpad_contract: formatTemplate(templates[4]),
    })
  }
  return formattedTemplates
}
```

## How the build works

The build output consists of two files being written to [/artifacts](../artifacts):
* `contract-name@version.wasm` (gitignored)
* `contract-name@version.wasm.sha256` (not gitignored).

The builder environment and procedure are defined in:
* [`@fadroma/ops/Build`](https://github.com/hackbg/fadroma/tree/v100/packages/ops/Build.ts).
* [`@fadroma/ops/Docker`](https://github.com/hackbg/fadroma/tree/v100/packages/ops/Docker.ts).
* [`@fadroma/scrt`](https://github.com/hackbg/fadroma/tree/v100/packages/scrt).

The build environment is based on an enhanced version of `enigmampc/secret-contract-optimizer`.
