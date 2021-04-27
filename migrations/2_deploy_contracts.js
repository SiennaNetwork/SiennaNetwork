const WrappedSienna = artifacts.require('WrappedSienna');

module.exports = function (deployer, networks, accounts) {
  // TODO: use real bridge address for e2e tests
  // deployer.deploy(WrappedSienna, accounts[0]);
  deployer.deploy(WrappedSienna, '0xFA22c1BF3b076D2B5785A527C38949be47Ea1082');
};
