# Sienna Scripts: Live

Smart contract development with live reloading.

```typescript
import { Console, bold } from '@hackbg/fadroma'
const console = new Console('@sienna/scripts/Live')
```

<table>

<tr><td valign="top">

* This script watches all files under `contracts/`.
  * [ ] TODO:
        It should also watch `libraries/` and know
        which crates depend on a changed lib. This
        can be done with:
        * `cargo build --unit-graph` ?
        * `cargo build --build-plan` ?
        * `cargo tree -i`            ?

</td><td valign="top">

```typescript
import { dirname, fileURLToPath, resolve } from '@hackbg/fadroma'
const __dirname = dirname(fileURLToPath(import.meta.url))
const watchRoot = resolve(__dirname, '../contracts/**/*')
console.log('Watching files under', watchRoot)
```

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

* [`chokidar`](https://www.npmjs.com/package/chokidar) is used
  to get notifications for changes in the Rust source code.
  * [ ] TODO: The TypeScript code of the scripts and clients
              should also be watched. Part of this is already
              in place, via Ganesha.

</td><td valign="top">

```typescript
import { FSWatcher } from 'chokidar'
const watcher = new FSWatcher({
  ignoreInitial:  true,
  usePolling:     false,
  followSymlinks: false,
  alwaysStat:     false,
  ignored: '**/target/**/*'
})
```

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

* Once the watcher has initialized, this script deploys the
  latest versions of everything, and only then starts actually
  watching for changes.

* When a change is detected, `cargo locate-project` is used
  to find the `Cargo.toml` corresponding to the crate which
  contains the changed file.

</td><td valign="top">

```typescript
import Deploy from './Deploy'
watcher.on('ready', async function deployAndStartWatching () {
  await Deploy.run('latest')
  watcher.on('all', async function onChange (event, path, stats) {
    if (!shouldRebuild(dirname(path))) return
    await uploadSource(pathToSource(path))
  })
})
watcher.add(watchRoot)
```

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

* `cargo locate-project` returns the path to the workspace's
  root `Cargo.toml` for files that are outside of a crate.
  These files are not part of a contract, therefore we ignore
  changes to them.

</td><td valign="top">

```typescript
function shouldRebuild (path) {
  const manifest = getManifestForPath(path)
  return manifest !== rootManifest
}

const rootManifest = getManifestForPath(__dirname)

import { execFileSync } from '@hackbg/fadroma'
function getManifestForPath (path: string): string {
  return String(execFileSync(
    'cargo', ['locate-project', '--message-format=plain'],
    { cwd: path }
  )).trim()
}
```

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

* From the crate name of the changed file,
  a `Source` object can be created.

</td><td valign="top">

```typescript
import { workspace } from '@sienna/settings'
function pathToSource (path: string): Source {
  const manifest = getManifestForPath(dirname(path))
  const {package:{name}} = TOML.parse(readFileSync(manifest, 'utf8'))
  return new Source(workspace, name)
}
```

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

* The `Source` object is then passed to a `Builder`
  to create an `Artifact`, representing the WASM blob
  that the chain can execute.
  * [ ] TODO: Disable builder caching

* The next step is to upload the `Artifact` using the `Uploader` instance
  to the devnet. This produces a `Template` containing the code ID and code hash
  needed to instantiate a contract from the `Template`.

* Then, deployment procedures in [Deploy.ts.md](./Deploy.ts.md) create the
  new contract instances with new versions of the code, correct `InitMsg`,
  updated inter-contract dependencies, etc.

</td><td valign="top">

```typescript
import { getSource } from './Build'
import { Scrt_1_2 } from '@hackbg/fadroma'
const builder = Scrt_1_2.getBuilder()
let building = false
async function uploadSource (source: Source): Template {
  if (building) await builder.kill()
  building = true
  const artifact = await builder.build(source)
  building = false
  return await uploader.upload(artifact)
}
```

</td></tr><tr><!--spacer--></tr><tr><td valign="top">

</td><td valign="top">

```typescript
```

</td></tr><tr><!--spacer--></tr></table>
