import {
  IAgent, IContract, ContractState, ContractConstructor,
  AugmentedScrtContract_1_2,
  randomHex
} from "@fadroma/scrt"
import { SNIP20Contract } from '@fadroma/snip20'
import { Init } from './schema/init.d'

import { RewardsTransactions } from './RewardsTransactions'
import { RewardsQueries } from './RewardsQueries'
export class RewardsContract extends AugmentedScrtContract_1_2<RewardsTransactions, RewardsQueries> {

  crate = 'sienna-rewards'

  name = 'SiennaRewards'

  initMsg: Init = {
    admin: this.instantiator?.address,
    config: {}
  }

  constructor (options: ContractState & {
    /** Admin agent */
    admin?:       IAgent,
    /** Address of other user that can increment the epoch */
    timekeeper?:  string,
    /** Staked token */
    lpToken?:     SNIP20Contract,
    /** Rewarded token */
    rewardToken?: SNIP20Contract,
    /** Bonding period config */
    bonding?:     number,
  } = {}) {
    super(options)
    this.initMsg.admin = options.admin?.address
    this.initMsg.config = {
      reward_vk:    randomHex(36),
      bonding:      options.bonding || 86400,
      timekeeper:   options.timekeeper,
      lp_token:     options.lpToken?.link,
      reward_token: options.rewardToken?.link,
    }
  }

}
