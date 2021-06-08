Ephemeral storage for localnet. Safe to delete.
* Make sure not to commit the deletion of `.gitignore` and `README.md` though.
  (If you delete `.gitignore` a whole bunch of temp files may end up in the repo.)
* Delete `uploads` to clear just the upload cache.
  (Currently needed after building new binaries.)
* Delete `.secretcli`, `.secretd`, `.sgx-secrets` to clear the node state.
  (You may need to use `sudo` because Docker creates them as `root`.) Then,
  delete `uploads` and `instances` because what they point to doesn't exist anymore.
