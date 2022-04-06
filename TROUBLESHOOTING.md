# Troubleshooting

## Shallow checkout

If you forget `--recurse-submodules` on initial checkout,
or something goes wrong with your Git repo (both happen)
you may see this error:

```
ERR_PNPM_NO_MATCHING_VERSION_INSIDE_WORKSPACE  In deps/fadroma:
No matching version found for @hackbg/ganesha@* inside the workspace
```

To fetch the missing submodules, go to the root of the repo and run:

```sh
git submodule update --init --recursive
```

