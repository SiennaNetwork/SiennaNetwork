## `build/`

* `optimizer/`: build tool
* `outputs/`: contains build results
* `outputs/checksums.sha256.txt`: hashes of built binaries
* `outputs/<PACKAGE>@<COMMIT>.wasm`: built binaries (gitignored)
* `outputs/<PACKAGE>@<COMMIT>.<NETWORK>.upload`:
  code IDs; delete these to reupload (e.g. after reinitializing localnet)
