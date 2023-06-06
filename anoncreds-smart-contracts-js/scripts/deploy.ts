import { ethers } from "hardhat";

async function main() {
  const [deployer] = await ethers.getSigners();

  const AnoncredsRegistry = await ethers.getContractFactory("AnoncredsRegistry");

  console.log("Deploying contracts with the account:", deployer.address);

  const registry = await AnoncredsRegistry.deploy();

  await registry.deployed();

  console.log(
    `AnoncredsRegistry deployed to ${registry.address}`
  );
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
