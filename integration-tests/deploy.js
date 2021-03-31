import { say as sayer, SecretNetwork } from '@hackbg/fadroma'

import * as dotenv from 'dotenv'
import { resolve } from 'path'
import.meta.url

dotenv.config()

const say = sayer.tag(() => new Date().toISOString())

async function deploy() {
    const commit = process.argv[2]
    
    if (commit == undefined) {
        console.log("Specify the commit hash to deploy.");
        return;
    }

    SecretNetwork.Agent.APIURL = process.env.SECRET_REST_URL

    const snip20_wasm = resolve(`../dist/${commit}-snip20-reference-impl.wasm`)
    const exchange_wasm = resolve(`../dist/${commit}-exchange.wasm`)
    const ido_wasm = resolve(`../dist/${commit}-ido.wasm`)
    const factory_wasm = resolve(`../dist/${commit}-factory.wasm`)
  
    const client = await SecretNetwork.Agent.fromMnemonic({say, name: "deployer", mnemonic: process.env.MNEMONIC})
    const builder = new SecretNetwork.Builder({ say: say.tag('builder'), outputDir: '', agent: client })
  
    await builder.upload(exchange_wasm)
    await builder.upload(snip20_wasm)
    await builder.upload(ido_wasm)
    await builder.upload(factory_wasm)
}

deploy().catch((err) => {
    console.error(err);
});