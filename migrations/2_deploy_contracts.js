const WrappedSienna = artifacts.require('WrappedSienna');

module.exports = function (deployer, networks, accounts) {
  // TODO: use real bridge address for e2e tests
  // deployer.deploy(WrappedSienna, accounts[0]);

  // MAINNET SCRT_ETH BRIDGE
  deployer.deploy(WrappedSienna, '0xf4b00c937b4ec4bb5ac051c3c719036c668a31ec');
};
