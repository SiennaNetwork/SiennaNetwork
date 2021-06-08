Created from https://github.com/enigmampc/secret-template.git

## Build

```
make compile-optimized-reproducible
```

## Deploy

```
cp env.example .env
$EDITOR .env # populate with API, address, and mnemonic
yarn
node deploy.js
```
