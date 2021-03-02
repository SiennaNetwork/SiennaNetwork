#!/usr/bin/env node

async function poc () {
  const say = require('./say')('poc')

  const schedule =
    { "total":              "1000"
    , "pools":
      [ { "name":           "Cool Pool"
        , "total":          "1000"
        , "partial":        false
        , "channels":
          [ { "name":       "Cool Channel"
            , "amount":     "1000"
            , "periodic":
              { "type":     "channel_periodic"
              , "amount":   "1000"
              , "cliff":    "0",
              , "start_at": 0
              , "interval": "1000"
              , "duration": "7000"
              , "expected_portion":   "142"
              , "expected_remainder": "6"
              }
            , "allocations":
              [ [ 0, [ { "addr": ALICE }
                     , { "addr": BOB   } ] ] } ] } ] } // too many!

  const client = require('./client')()
  const deploy = require('./deploy')
  const [xZibitA, xZibitB] = await Promise.all([
    deploy({
      client,
      token: `dist/2021-03-02-80f6297-snip20-reference-impl.wasm`,
      mgmt:  `dist/2021-03-02-80f6297-sienna-mgmt.wasm`,
    }),
    deploy({
      client,
      token: `dist/2021-03-02-bfdef1b-snip20-reference-impl.wasm`,
      mgmt:  `dist/2021-03-02-bfdef1b-sienna-mgmt.wasm`,
    })
  ])

  await Promise.all([
    config({client, mgmt: xZibitA, schedule})
    config({client, mgmt: xZibitB, schedule})
  ])

  await test(xZibitA)
  await test(xZibitB)

  async function test (X) {
  }

}


module.exports=(require.main&&require.main!==module)?poc:poc()
