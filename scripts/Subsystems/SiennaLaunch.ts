import * as API from '@sienna/api'
import { MigrationContext, buildAndUploadMany, Console } from '@hackbg/fadroma'
import { versions, contracts, sources } from '../Build'
import { linkTuple } from '../misc'

const console = Console('Sienna Launch')

export type Address = string
export type Binary  = string
export type GitRef  = string

export interface LaunchpadDeployContext extends MigrationContext {
  /** Address of the admin. */
  admin: Address
}

export interface LaunchpadDeployResult {
  /** The deployed LPD contract. */
  LPD: API.LaunchpadClient
}

export async function deployLaunchpad (context: LaunchpadDeployContext):
  Promise<LaunchpadDeployResult>
{
  const {
    ref = versions.HEAD,
    src = sources(ref, contracts.Launchpad),
    builder,
    uploader,
    templates = await buildAndUploadMany(builder, uploader, src),
    deployment,
    prefix,
    agent,
    admin = agent.address,
  } = context
  // 1. Build and upload LPD contracts:
  const [launchpadTemplate, idoTemplate] = templates
  // 2. Instantiate the launchpad contract 
  let prng_seed: Binary = "";
  let entropy:   Binary = "";
  const lpdInstance = await deployment.init(agent, launchpadTemplate, 'LPD', {
    admin:     admin,
    tokens:    [],
    prng_seed: prng_seed,
    entropy:   entropy
  })
  const lpdLink = linkTuple(lpdInstance)
  // 3. Return interfaces to the contracts.
  //    This will add them to the context for
  //    subsequent steps. (Retrieves them through
  //    the Deployment to make sure the receipts
  //    were saved.)
  const client = (Class, name) => new Class({...deployment.get(name), agent})
  return {
    LPD: client(API.LaunchpadClient, 'LPD')
  }
}
