import { ethers } from "hardhat";

// https://github.com/uport-project/ethr-did-registry?tab=readme-ov-file#contract-deployments
const MUMBAI_DID_ETHR_ADDR = "0xdCa7EF03e98e0DC2B855bE647C39ABe984fcF21B";

async function main() {
  const [deployer] = await ethers.getSigners();

  const EthereumDIDRegistry = await ethers.getContractFactory("EthereumDIDRegistry");
  const EthrDIDLinkedResourcesRegistry = await ethers.getContractFactory("EthrDIDLinkedResourcesRegistry");

  console.log("Deploying contracts with the account:", deployer.address);

  const ethereumDidRegistry = await EthereumDIDRegistry.deploy();
  await ethereumDidRegistry.deployed();

  console.log(
    `EthereumDIDRegistry deployed to ${ethereumDidRegistry.address}`
  );

  const resourcesRegistry = await EthrDIDLinkedResourcesRegistry.deploy(
    // "0xdCa7EF03e98e0DC2B855bE647C39ABe984fcF21B" // mumbai did:ethr addr
    ethereumDidRegistry.address
  );
  await resourcesRegistry.deployed();

  console.log(
    `EthrDIDLinkedResourcesRegistry deployed to ${resourcesRegistry.address}`
  );
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
