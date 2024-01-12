import { ethers } from "hardhat";

async function main() {
  const [deployer] = await ethers.getSigners();

  const EthereumDIDRegistry = await ethers.getContractFactory("EthereumDIDRegistry");
  const EthrDIDLinkedResourcesRegistry = await ethers.getContractFactory("EthrDIDLinkedResourcesRegistry");
  const EthrDLRRegistry = await ethers.getContractFactory("EthrDLRRegistry");

  console.log("Deploying contracts with the account:", deployer.address);

  const ethereumDidRegistry = await EthereumDIDRegistry.deploy();
  await ethereumDidRegistry.deployed();

  console.log(
    `EthereumDIDRegistry deployed to ${ethereumDidRegistry.address}`
  );

  const resourcesRegistry = await EthrDIDLinkedResourcesRegistry.deploy(ethereumDidRegistry.address);
  await resourcesRegistry.deployed();

  console.log(
    `EthrDIDLinkedResourcesRegistry deployed to ${resourcesRegistry.address}`
  );

  const dlrRegistry = await EthrDLRRegistry.deploy(ethereumDidRegistry.address);
  await dlrRegistry.deployed();

  console.log(
    `EthrDLRRegistry deployed to ${dlrRegistry.address}`
  );
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
