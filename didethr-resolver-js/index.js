// import {Resolver} from 'did-resolver'
// import {getResolver} from 'ethr-did-resolver'
const { Resolver } = require('did-resolver');
const { getResolver } = require('ethr-did-resolver');

const did_registry_address = process.env.DID_REGISTRY_ADDRESS | "0x5fbdb2315678afecb367f032d93f642f64180aa3"
const did_url = process.env.DID | "did:ethr:gmtest:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"

async function run() {
    const providerConfig = {
        networks: [
            { name: "gmtest", rpcUrl: "http://localhost:8545", registry: did_registry_address },
        ]
    }

    const ethrDidResolver = getResolver(providerConfig)
    const didResolver = new Resolver(ethrDidResolver)

    const doc = await didResolver.resolve(did_url)
    console.log(JSON.stringify(doc, null, 2))
}

run()
    .catch(err => {
        console.error((err))
    })
    .then(() => {
        return 0
    })