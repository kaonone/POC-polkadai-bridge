var DAIBridge = artifacts.require("./DAIBridge.sol");


module.exports = function(deployer, network, accounts) {
  let owner = accounts[0];
  
  let token = "0xc4375b7de8af5a38a93548eb8453a498222c4ff2"; //DAI

  
  //console.log('owner of storage contracts: ' + owner)

  deployer.deploy(DAIBridge, token,  {from: owner});
  
};

