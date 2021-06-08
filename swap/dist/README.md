# Production builds.

This directory intentionally left empty.

If you run `make` in the repo's root, 
a production build will show up here.

The only file that should persist
is `checksums.sha256.txt`, which
accumulates the checksums for the
build outputs of different commits
to catch changes in the toolchain.

`ido.wasm` is not a part of the project, but is needed for testing and deploying, so it is kept for convenience.
