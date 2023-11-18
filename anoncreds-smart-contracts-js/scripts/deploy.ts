import { ethers } from "hardhat";

async function main() {
  const [deployer] = await ethers.getSigners();

  const EthereumDIDRegistry = await ethers.getContractFactory("EthereumDIDRegistry");

  const AnoncredsRegistry = await ethers.getContractFactory("AnoncredsRegistry");

  console.log("Deploying contracts with the account:", deployer.address);

  const ethereumDidRegistry = await EthereumDIDRegistry.deploy();
  await ethereumDidRegistry.deployed();

  console.log(
    `EthereumDIDRegistry deployed to ${ethereumDidRegistry.address}`
  );

  const anoncredsRegistry = await AnoncredsRegistry.deploy(ethereumDidRegistry.address);
  await anoncredsRegistry.deployed();

  console.log(
    `AnoncredsRegistry deployed to ${anoncredsRegistry.address}`
  );
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
