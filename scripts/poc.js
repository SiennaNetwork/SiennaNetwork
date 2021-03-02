#!/usr/bin/env node

async function poc () {
  const say = require('./say')('poc')

  const client = require('./client')()
  const deploy = require('./deploy')
  const [xZibitA, xZibitB] = await Promise.all([
    deploy({client,
      token: `dist/2021-03-02-80f6297-snip20-reference-impl.wasm.gz`,
      mgmt:  `dist/2021-03-02-80f6297-sienna-mgmt.wasm.gz`,
    }),
    deploy({client,
      token: `dist/2021-03-02-80f6297-snip20-reference-impl.wasm.gz`,
      mgmt:  `dist/2021-03-02-80f6297-sienna-mgmt.wasm.gz`,
    })
  ])

  const config = require('./configure')
  await Promise.all([
    config(xZibitA, schedule),
    config(xZibitB, schedule)
  ])
}


module.exports=(require.main&&require.main!==module)?poc:poc()
