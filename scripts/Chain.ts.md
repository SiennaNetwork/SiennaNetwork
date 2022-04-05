# Sienna Scripts: Devnet Management

```typescript
import Fadroma, { Console, bold } from '@hackbg/fadroma'
const console = Console('@sienna/Devnet')
```

For local develoment, an instance of Secret Network
can be launched in a Docker container. This is mostly
handled automatically by Fadroma; these commands are
here for the rare case where the devnet has to be
prodded manually.

## Showing devnet status

```typescript
Fadroma.command('status', Fadroma.Chain.FromEnv, Fadroma.Chain.Status)
```

## Resetting the devnet

> Run with `pnpm devnet reset`

`secretd` currently runs as `root` inside the devnet container,
which sometimes creates root-owned state files under `receipts/`.
This command deletes the active devnet, possibly running a cleanup container
to delete those files.

```typescript
Fadroma.command('reset', Fadroma.Chain.FromEnv, Fadroma.Chain.Reset)
```

## Entry point

```typescript
Fadroma.module(import.meta.url)
```
