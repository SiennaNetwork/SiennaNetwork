#[macro_use] extern crate fadroma;

use cosmwasm_std::CanonicalAddr;

contract!(

    State {

        Config {
            // Send from this address to launch the vesting process
            // TODO make configurable
            admin:    cosmwasm_std::CanonicalAddr,

            // The token contract that will be controlled
            // TODO see how this can be generated for testing
            token:    Option<cosmwasm_std::CanonicalAddr>,

            // Whether the vesting process has begun and when
            launched: Option<u64>
        }

        Recipients {
            // Addresses that can claim tokens
        }

    }

    InitMsg (deps, env, msg: {
        token: Option<cosmwasm_std::CanonicalAddr>
    }) {
        Config: {
            admin:    deps.api.canonical_address(&env.message.sender)?,
            token:    msg.token,
            launched: None
        }
        Recipients: {}
    }

    QueryMsg (deps, msg) {
        StatusQuery () {
            (c: Config) {
                msg::StatusResponse {
                    launched: None
                }
            }
        }
    }

    HandleMsg (deps, env, sender, msg) {
        Launch () {
            (c: Config) { is_admin(c, sender) }
            (c: &mut Config) {
                c.launched = Some(env.block.time);
            }
        },
        SetRecipient (address: cosmwasm_std::CanonicalAddr) {
            (c: Config) { has_not_launched(c) && is_admin(c, sender) }
            (r: &mut Recipients) {
                //r.set(r, to_vec(&Recipient {
                    //address,
                    //first_vesting: 0,
                    //interval:      0,
                    //last_vesting:  0,
                    //claimable:     0,
                //}))
            }
        },
        RemoveRecipient (address: cosmwasm_std::CanonicalAddr) {
            (c: Config) { has_not_launched(c) && is_admin(c, sender)  }
            (r: &mut Recipients) {
                //r.remove(sender)
            }
        },
        Claim () {
            (r: Recipients) { has_launched(c) && can_claim(r, sender) }
            (r: &mut Recipients) {
                //let mut recipient = r.iter_mut().find(
                    //|&x| x.address == sender
                //);
                //recipient.claimable = 0;
                //state::Recipients::set(sender.as_slice(), &to_vec(&sender)?);
            }
        }
    }

    Response {
        StatusResponse { launched: Option<u64> }
    }

);

fn has_launched (config: state::Config) -> bool {
    match config.launched { None => false, Some(_) => true }
}

fn has_not_launched (config: state::Config) -> bool {
    !has_launched(config)
}

fn is_admin (config: state::Config, addr: CanonicalAddr)
-> cosmwasm_std::StdResult<bool> {
    if addr != config.admin {
        Err(cosmwasm_std::StdError::Unauthorized { backtrace: None })
    } else {
        Ok(true)
    }
}

fn can_claim (recipients: state::Recipients, addr: CanonicalAddr) {
}

//message!(Recipient {
    //address:       CanonicalAddr,
    //first_vesting: u64,
    //interval:      u64,
    //last_vesting:  u64,
    //claimed:       u64,
//});
