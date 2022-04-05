# Sienna Scripts: Schema

## Overview

The contracts have the capability to output their API schema in the form of JSON schema.

From this, we create TypeScript type definitions via `json-schema-to-typescript`.

These type definitions are imported by the `Contract` classes.

```typescript
import Fadroma, { bold, timestamp, Console } from '@hackbg/fadroma'
const console = new Console('@sienna/scripts/Schema')

import { generateSchema } from '@hackbg/fadroma'
import { workspace } from '@sienna/settings'
Fadroma.command('generate', () => generateSchema(workspace, [
  "tge/mgmt",
  "tge/rpt",
  "tge/snip20-sienna",

  "amm/amm-snip20",
  "amm/exchange",
  "amm/factory",
  "amm/ido",
  "amm/launchpad",
  "amm/lp-token",
  "amm/rewards",
  "amm/router",

  "lend/interest_model",
  "lend/market",
  "lend/oracle",
  "lend/overseer",
  "lend/mock_band_oracle"
])
```

## Entry point

```typescript
Error.stackTraceLimit = Infinity
export default Fadroma.module(import.meta.url)
```
