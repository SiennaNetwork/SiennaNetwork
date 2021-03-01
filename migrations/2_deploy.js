const WrappedSienna = artifacts.require('WrappedSienna');
module.exports = function (deployer) {
  const bridgeAddress = '0x2b89bf8ba858cd2fcee1fada378d5cd6936968be';

  deployer.deploy(WrappedSienna, bridgeAddress);
};
