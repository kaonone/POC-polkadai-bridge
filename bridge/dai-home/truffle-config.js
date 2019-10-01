
const HDWalletProvider = require("truffle-hdwallet-provider");
require('dotenv').config()  // Stores environment-specific variable from '.env' to process.env
require('babel-register');
require('babel-polyfill');


console.log(process.env.METAMASK_MNEMONIC);
console.log(process.env.INFURA_API_KEY);
module.exports = {
  // See <http://truffleframework.com/docs/advanced/configuration>
  // to customize your Truffle configuration!
   compilers: {
	 solc: {
	   version: "0.5.9"  // Change this to whatever you need
	 }
   },
	networks: {
		development: {
			host: 'localhost',
			port: 8545,
			network_id: '*',
			gas: 4600000
		},

		mainnet: {
			  provider: function () {
	          return new HDWalletProvider(process.env.METAMASK_MNEMONIC, "https://mainnet.infura.io/v3/" + process.env.INFURA_API_KEY)
	      },
	      network_id: 1,
	      network_id: 1,
     	  gas: 7000000,
     	  skipDryRun:true
		},
		
		kovan: {
	          provider: function () {
	          return new HDWalletProvider(process.env.METAMASK_MNEMONIC, "https://kovan.infura.io/v3/" + process.env.INFURA_API_KEY)
	      },
	      network_id: 42,
	      gas: 7000000
	    },
	},
	mocha: {
	   reporter: 'eth-gas-reporter',
	   reporterOptions: {
	     gasPrice: 21
	   }
	},
};