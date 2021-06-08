const WrappedSienna = artifacts.require('WrappedSienna');

module.exports = function (deployer, networks, accounts) {
  // TODO: use real bridge address for e2e tests
  deployer.deploy(WrappedSienna, accounts[0]);
};
