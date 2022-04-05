---
literate: typescript
---

# Sienna Scripts

## Deprecation warning

```typescript
console.log(`
This command has been deprecated.

Previously, there were two mega-scripts, "dev" and "ops".
These have now been broken down into smaller modules.
Use "pnpm run" to get the list of top-level commands.
`.trimStart())
process.exit(1)
```

## How commands work

> See: [Fadroma CLI Documentation](https://github.com/hackbg/fadroma/blob/22.01/packages/cli/README.md)

The procedures defined in this directory are executed by the [Komandi](https://github.com/hackbg/fadroma/tree/21.12/packages/komandi)
library based on the command line arguments (see [Entry point](#entry-point)). Or, you can
use them from another script by importing this module.

## Chains

The active [`Chain`](https://github.com/hackbg/fadroma/blob/22.01/packages/ops/Chain.ts)
is selected via the `FADROMA_CHAIN` environment variable.
You can set it in a `.env` file in the root of the repo.

Run this script with `FADROMA_CHAIN` set to an empty value,
to list possible values.
