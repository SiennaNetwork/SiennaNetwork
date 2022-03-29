# Sienna Scripts: Build

```typescript
import Fadroma, {
  bold, timestamp, Console,
  collectCrates, Scrt_1_2
} from '@hackbg/fadroma'
import { workspace } from '@sienna/settings'
const console = new Console('@sienna/scripts/Build')
const parallel = (...commands) => input => Promise.all(commands.map(command=>command(input)))
```

## Overview

**Run `pnpm -w build` to list the subsets of contracts that can be built.**

The command `pnpm -w build all` compiles all contracts for production.

The build output consists of two files being written to [/artifacts](../artifacts):
* `contract-name@version.wasm` (gitignored)
* `contract-name@version.wasm.sha256` (not gitignored).

Which contracts are built by each command is defined in:
* [`@sienna/tge/build.ts`](../contracts/tge/build.ts')
* [`@sienna/amm/build.ts`](../contracts/amm/build.ts')
* [`@sienna/lend/build.ts`](../contracts/tge/build.ts')

The builder environment and procedure are defined in:
* [`@fadroma/ops/Build`](https://github.com/hackbg/fadroma/tree/v100/packages/ops/Build.ts).
* [`@fadroma/ops/Docker`](https://github.com/hackbg/fadroma/tree/v100/packages/ops/Docker.ts).
* [`@fadroma/scrt-1.2`](https://github.com/hackbg/fadroma/tree/v100/packages/scrt-1.2).

You will need at least **2G of disk space** for the build container.

## Contract sources

The `getSources` function takes one optional argument, `ref`
(if you want to get `Source` objects pointing to a past commit),
and returns a mapping from crate name to `Source` object.

```typescript
export const getSources = collectCrates(workspace, [
  // TGE
  'snip20-sienna',
  'sienna-mgmt',
  'sienna-rpt',
  // Swap
  'amm-snip20',
  'lp-token',
  'factory',
  'exchange',
  'router',
  'ido',
  'launchpad',
  'sienna-rewards'
  // Lend
  'lend-interest-model',
  'lend-market',
  'lend-mock-oracle',
  'lend-oracle',
  'lend-overseer'
])
```

### Commits of note

```typescript
export const refs = {
  // TGE_v1: TODO find which commit was deployed on mainnet launch
  AMM_v1:     'legacy/amm-v1',
  AMM_v2:     '39e87e4',
  Rewards_v2: 'rewards-2.1.2',
  Rewards_v3: '39e87e4'
}
```

## Build combinations

These are groups of contracts that depend on each other
and you may want to build together. This is not particularly useful
and may be deprecated in the future as live mode evolves.

```typescript
Fadroma.command('all',
  buildTge.bind(null, 'HEAD'),
  buildTokens.bind(null, 'HEAD'),
  buildTokens.bind(null, refs['AMM_v1']),
  buildTokens.bind(null, refs['AMM_v2']),
  buildAmm.bind(null, 'HEAD'),
  buildAmm.bind(null, refs['AMM_v1']),
  buildAmm.bind(null, refs['AMM_v2']),
  buildLaunchpad.bind(null, refs['AMM_v2'])
  buildIdo.bind(null, refs['AMM_v2']),
  buildRewards.bind(null, refs['Rewards_v2']),
  buildRewards.bind(null, refs['Rewards_v3'])

Fadroma.command('router', parallel(
  buildTokens,
  buildRouter))

Fadroma.command('lend', parallel(
  buildTokens,
  buildLend))

Fadroma.command('amm_v1',
  buildAmm.bind(null, refs['AMM_v1']))

Fadroma.command('amm_v2',
  buildAmm.bind(null, refs['AMM_v2']))

export async function buildTge (ref?) {
  const sources = getSources(ref)
  return Scrt_1_2.getBuilder().buildMany([
    'snip20-sienna',
    'sienna-mgmt',
    'sienna-rpt',
  ].map(x=>sources[x]))
}

export async function buildTokens (ref?): Promise<Artifact[]> {
  const sources = getSources(ref)
  return Scrt_1_2.getBuilder().buildMany([
    'amm-snip20',
    'lp-token',
  ].map(x=>sources[x]))
}

export async function buildAmm (ref?): Promise<Artifact[]> {
  const sources = getSources(ref)
  return Scrt_1_2.getBuilder().buildMany([
    'factory',
    'exchange',
  ].map(x=>sources[x]))
}

export async function buildLaunchpad (ref?): Promise<Artifact[]> {
  const sources = getSources(ref)
  return Scrt_1_2.getBuilder().buildMany([
    'launchpad',
  ].map(x=>sources[x]))
}

export async function buildIdo (ref?): Promise<Artifact[]> {
  const sources = getSources(ref)
  return Scrt_1_2.getBuilder().buildMany([
    'ido',
  ].map(x=>sources[x]))
}

export async function buildRouter (ref?): Promise<Artifact[]> {
  const sources = getSources(ref)
  return Scrt_1_2.getBuilder().buildMany([
    'router',
  ].map(x=>sources[x]))
}

export async function buildRewards (ref?): Promise<Artifact[]> {
  const sources = getSources(ref)
  const builder = Scrt_1_2.getBuilder()
  return Scrt_1_2.getBuilder().buildMany([
    'sienna-rewards',
  ].map(x=>sources[x]))
}

export async function buildLend (ref?): Promise<Artifact[]> {
  const sources = getSources(ref)
  return Scrt_1_2.getBuilder().buildMany([
    'lend-interest-model',
    'lend-oracle',
    'lend-market',
    'lend-overseer',
    'lend-mock-oracle',
    'amm-snip20'
  ].map(x=>sources[x]))
}
```

## Building the templates for the AMM Factory

The AMM factory needs to be configured
with [code ID + code hash] pairs that the factory
uses to instantiate its contracts.

The set of templates supported by the factory
differs between AMMv1 and AMMv2 (v1 contained
some extra templates that weren't being used)

```typescript
export async function buildAMMTemplates (
  uploader: Uploader,
  version:  AMMVersion,
  builder:  Builder = Scrt_1_2.getBuilder()
): Promise<Record<string, {id:number,code_hash:string}>> {
  const crates  = getSources(refs[`AMM_${version}`])
  const sources = [crates['exchange'], crates['lp-token']]
  if (version === 'v1') {
    console.info('Building extra (unused) templates required by AMM Factory v1...')
    sources.push(crates['amm-snip20'])
    sources.push(crates['launchpad'])
    sources.push(crates['ido'])
  }
  const artifacts = await builder.buildMany(sources)
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

## Entry point

```typescript
Error.stackTraceLimit = Infinity
export default Fadroma.module(import.meta.url)
```
