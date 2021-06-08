# `libraries/`

* `fadroma`: smart contract microframework containing common boilerplate code
* `kukumba`: helper macro for writing BDD (Cucumber) test cases
* `linear-map`: a map-like wrapper around `Vec`
  * workaround for `serde_json_wasm` not implementing `serialize_map`
* `platform`: the entire SecretNetwork platform
  * contains modifications to `SecretJS` for better error handling
  * hoping to upstream those eventually
* `schedule`: business logic for `../contracts/mgmt`
* `utils`: miscellaneous utilities (not used on this branch, came in via backmerge)
