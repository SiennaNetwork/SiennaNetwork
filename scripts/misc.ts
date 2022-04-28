import Fadroma from '@hackbg/fadroma'

export const linkTuple = instance => [instance.address, instance.codeHash]

export const linkStruct = instance => ({
  address:   instance?.address,
  code_hash: instance?.codeHash.toUpperCase()
})

export const templateStruct = template => ({
  id:        Number(template.codeId),
  code_hash: template.codeHash.toUpperCase()
})

/** Command fragment: add `...canBuildAndUpload` to the start of
  * a command to enable building and uploading contracts *from* local sources
  * and *for* Secret Network 1.2, *ignoring* the Deployments system. */
export const canBuildAndUpload = [
  Fadroma.Chain.FromEnv,
  Fadroma.Build.Scrt_1_2,
  Fadroma.Upload.FromFile,
]

/** Command fragment: add `...canBuildAndUpload` to the start of
  * a command to enable building and uploading contracts *from* local sources
    and *for* Secret Network 1.2, inside a *new* deployment. */
export const inNewDeployment = [
  ...canBuildAndUpload,
  Fadroma.Deploy.New
]

/** Command fragment: add `...canBuildAndUpload` to the start of
  * a command to enable building and uploading contracts *from* local sources
    and *for* Secret Network 1.2, inside the *currently selected* deployment. */
export const inCurrentDeployment = [
  ...canBuildAndUpload,
  Fadroma.Deploy.Append
]
