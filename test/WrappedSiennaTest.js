const { accounts, contract } = require('@openzeppelin/test-environment');
const { BN } = require('@openzeppelin/test-helpers');
const { expect } = require('chai');

const WrappedSienna = artifacts.require('WrappedSienna');

describe('WrappedSienna', () => {
  const bridgeAddress = '0x2b89bf8ba858cd2fcee1fada378d5cd6936968be';
  const name = 'Wrapped SIENNA';
  const symbol = 'wSIENNA';

  beforeEach(async () => {
    this.token = await WrappedSienna.new(bridgeAddress);
  });

  it('has a name', async () => {
    expect(await this.token.name()).to.equal(name);
  });

  it('has a symbol', async () => {
    expect(await this.token.symbol()).to.equal(symbol);
  });

  it('has decimals', async () => {
    expect(await this.token.decimals()).to.be.bignumber.equal('18');
  });
});
