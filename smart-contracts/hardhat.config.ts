import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import * as dotenv from "dotenv";

dotenv.config({ path: '../.env' });

const config: HardhatUserConfig = {
  solidity: "0.8.18",
  networks: {
    hardhat: {
      accounts: {
        mnemonic: process.env.MNEMONIC
      },
    },
  }
};

export default config;
