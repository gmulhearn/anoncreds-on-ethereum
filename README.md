# Anoncreds on Ethereum Proof of Concept
Simple project demonstrating how a smart contract could be used as a VDR for anoncreds data (schemas and credential definitions).

The `anoncreds-smart-contracts-js` directory contains a cookie cutter `hardhat` project for developing and deploying the `AnoncredsRegistry` smart contract. Refer to the hardhat generated README for usage of the hardhat project.

Whilst the `eth-anoncreds-rust-demo` directory contains a rust demo binary, which uses `ethers` to connect with a `AnoncredsRegistry` smart contract instance and use it as a VDR in standard anoncreds issuer/holder & verifier/prover flows.

# Setup
To setup and run the demo:
1. create your `.env` file in the root of this project. Using `.env.example` as an example.
2. within `anoncreds-smart-contracts-js`: `npm install`
3. within `anoncreds-smart-contracts-js`: use hardhat to run a local ledger in a seperate terminal: `npx hardhat node`
4. within `anoncreds-smart-contracts-js`: use hardhat to deploy the `AnoncredsRegistry` contract to the local ledger: `npx hardhat run --network localhost scripts/deploy.ts`
5. copy the deployed address into the `REGISTRY_WETH_ADDRESS` const of the [anoncreds_eth_registry.rs file](/eth-anoncreds-rust-demo/src/anoncreds_eth_registry.rs)
6. within `eth-anoncreds-rust-demo`: run the demo!: `cargo run`