import { randomBytes } from 'crypto'
import { stderr } from 'process'
import { readdirSync, readFileSync, existsSync } from 'fs'
import assert from 'assert'

import bignum from 'bignumber.js'
import { table } from 'table'
import { render } from 'prettyjson'

import { SNIP20Contract, MGMTContract, RPTContract, RewardsContract } from '../api/index.js'

import { taskmaster } from '@fadroma/utilities'
import { fileURLToPath, resolve, basename, extname, dirname
       , readFile, writeFile } from '@fadroma/utilities/sys.js'
import { pull } from '@fadroma/utilities/net.js'

import { SecretNetwork } from '@fadroma/scrt-agent'
import Ensemble from '@fadroma/scrt-ops/ensemble.js'

import { conformChainIdToNetwork, conformNetworkToChainId
       , pickNetwork, pickInstance, pickKey } from './pick.js'
import { projectRoot, abs } from './root.js'

export const stateBase = abs('artifacts')

// decimals
export const fmtDecimals = d => x =>
  `${bignum(x).div(d).toString()}.${bignum(x).mod(d).toString().padEnd(18, '0')}`

export const SIENNA_DECIMALS = 18
export const ONE_SIENNA = bignum(`1${[...Array(SIENNA_DECIMALS)].map(()=>`0`).join('')}`)
export const fmtSIENNA = fmtDecimals(ONE_SIENNA)

export const SCRT_DECIMALS = 6
export const ONE_SCRT = bignum(`1${[...Array(SCRT_DECIMALS)].map(()=>`0`).join('')}`)
export const fmtSCRT = fmtDecimals(ONE_SCRT)

const prefix = new Date().toISOString().replace(/[-:\.]/g, '-').replace(/[TZ]/g, '_')
