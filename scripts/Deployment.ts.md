# Deployments overview

|                     |                                           |
| ------------------- | ----------------------------------------- |
| **Entry point:**    | `pnpm deployment` or `pnpm -w deployment` |
| **Related:**        | [./Deploy](./Deploy)                      |
| **Implementation:** | [@fadroma/ops/Deploy](https://github.com/hackbg/fadroma/blob/v100/packages/ops/Deploy.ts)              |
| **Specification:**  | [@fadroma/ops/Deploy.spec](https://github.com/hackbg/fadroma/blob/v100/packages/ops/Deploy.spec.ts.md) |

```typescript
import Fadroma, { Console } from '@hackbg/fadroma'
const console = new Console('Sienna Deployment')
```

The Sienna platform consists of multiple smart contracts that
depend on each other's existence and configuration. A group of
such contracts is called a **Deployment**.

## Deployments belong to chains

Each deployment belongs to a particular **Chain** - we have one production
deployment on mainnet, one or more staging deployments on testnet,
and any number of ephemeral local deployments on devnet.

So, to do anything with those, you need to first decide which chain
you're looking at by setting `FADROMA_CHAIN`.

## How deployments are stored

Each deployment is represented by a `.yml` file under
[`receipts/$FADROMA_CHAIN/deployments/`](../receipts).
This file consists of **Receipts** - snippets of YAML containing
basic info about each contract that was added to the deployment.

# Deployment commands

> **See also:** [How commands work](./README#how-commands-work)

## Creating a new deployment

> Do this with `pnpm deployment new`

```typescript
Fadroma.command('new', Fadroma.Chain.FromEnv, Fadroma.Deploy.New)
```

This will create a new, empty deployment, and write the corresponding receipt file.

NOTE: **Some `deploy` commands, such as `pnpm deploy tge`, may automatically create
and select a new deployment.** Look at the command definition in [./Deploy](./Deploy.ts.md)
to see if it creates a new deployment or adds contract to the existing one.

* [ ] TODO: Add a yes/no prompt to creating a new deployment so that accidentally
  running the wrong command doesn't deselect the active deployment.

## Viewing the status of the selected deployment

> Do this with `pnpm deployment status`

```typescript
Fadroma.command('status', Fadroma.Chain.FromEnv, Fadroma.Deploy.Status)
```

## Selecting a different deployment

> Do this with `pnpm deployment select $NAME_OF_DEPLOYMENT`

```typescript
Fadroma.command('select', Fadroma.Chain.FromEnv, Fadroma.Deploy.Select)
```

## Listing deployments

> Do this with `pnpm deployment list`

```typescript
Fadroma.command('list', Fadroma.Chain.FromEnv, Fadroma.Deploy.List)
```

## Adding contracts to the selected deployment

> Do this with `pnpm deploy $SOMETHING`.

See [./Deploy](./Deploy.ts.md) for more info.

### How deployments are invoked in deploy commands

Each command may start a new Deployment, or append to the one that is currently selected.
This is represented by the `Fadroma.Deploy.New` and `Fadroma.Deploy.Append` steps which
you can add to the start of your command. Invoking either of them populates the
`deployment` and `prefix` keys in the `MigrationContext` for subsequent steps.

* Use `Fadroma.Deploy.New` when you want to start from a clean slate.
  It will create a new deployment under `/receipts/$FADROMA_CHAIN/$TIMESTAMP`.

* Use `Fadroma.Deploy.Append` when you want to add contracts to an
  existing deployment.

# Entry point

* [ ] TODO: This `Fadroma.module` thing sucks and will be removed as soon as things settle down.

```typescript
export default Fadroma.module(import.meta.url)
```
