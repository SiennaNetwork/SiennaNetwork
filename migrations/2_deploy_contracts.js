const WrappedSienna = artifacts.require('WrappedSienna');
const MockSienna = artifacts.require('MockSienna');

module.exports = function (deployer, networks, accounts) {
  const bridgeAddress = '0x2b89bf8ba858cd2fcee1fada378d5cd6936968be';

  //deployer.deploy(WrappedSienna, bridgeAddress); //TODO: add after to test with real network
  deployer.deploy(WrappedSienna, accounts[0]);
};
