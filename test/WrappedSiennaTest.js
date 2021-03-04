//const { accounts, contract } = require('@openzeppelin/test-environment');
const { BN } = require('@openzeppelin/test-helpers');
const { expect, assert } = require('chai');

const WrappedSienna = artifacts.require('WrappedSienna');

contract('WrappedSienna', (accounts) => {
  const bridgeAddress = '0x2b89bf8ba858cd2fcee1fada378d5cd6936968be';
  const [initialHolder, recipient, anotherAccount] = accounts;
  const name = 'Wrapped SIENNA';
  const symbol = 'wSIENNA';

  beforeEach(async () => {
    this.token = await WrappedSienna.new(initialHolder);
  });

  it('has a name', async () => {
    const result = await this.token.name();
    assert.equal(result, name, 'The name is not correct.');
  });

  it('has a symbol', async () => {
    const result = await this.token.symbol();
    assert.equal(result, symbol, 'The symbol is not correct.');
  });

  it('has decimals', async () => {
    const result = await this.token.decimals();
    assert.equal(result, 18, 'Bignum not with 18 decimals.');
  });

  it('paused is successfull', async () => {
    const result = await this.token.pause({ from: initialHolder });
    assert.ok(result, 'Pause failed.');
  });

  it('unpaused is successfull', async () => {
    await this.token.pause({ from: initialHolder });
    const result = await this.token.unpause({ from: initialHolder });
    assert.ok(result, 'Unpaused failed.');
  });

  it('not allowed to pause from account with no pauser role', async () => {
    try {
      await this.token.pause({ from: recipient });
      assert.fail('The transaction should have thrown an error');
    } catch (err) {
      assert.include(
        err.message,
        'must have pauser role',
        'The error message should contain "must have pauser role"'
      );
    }
  });

  it('not allowed to unpause from account with no pauser role', async () => {
    try {
      await this.token.pause({ from: initialHolder });
      await this.token.unpause({ from: recipient });
      assert.fail('The transaction should have thrown an error');
    } catch (err) {
      assert.include(
        err.message,
        'must have pauser role',
        'The error message should contain "must have pauser role"'
      );
    }
  });

  it('transfer tokens from minter to other account (mint)', async () => {
    const minterBalanceBefore = await this.token.balanceOf(initialHolder);
    const accountBalanceBefore = await this.token.balanceOf(recipient);

    await this.token.transfer(recipient, new BN(1000000), {
      from: initialHolder,
    });

    const minterBalanceAfter = await this.token.balanceOf(initialHolder);
    const accountBalanceAfter = await this.token.balanceOf(recipient);

    assert.ok(
      minterBalanceAfter.eq(minterBalanceBefore),
      "Tokens in minter's account changed"
    );
    assert.ok(
      accountBalanceAfter.gt(accountBalanceBefore),
      "Tokens in the receiving account didn't increment"
    );
  });

  it('transfer tokens from other account to minter (burn)', async () => {
    await this.token.transfer(anotherAccount, new BN(2000000), {
      from: initialHolder,
    });
    const minterBalanceBefore = await this.token.balanceOf(initialHolder);
    const accountBalanceBefore = await this.token.balanceOf(anotherAccount);

    await this.token.transfer(initialHolder, new BN(1000000), {
      from: anotherAccount,
    });

    const minterBalanceAfter = await this.token.balanceOf(initialHolder);
    const accountBalanceAfter = await this.token.balanceOf(anotherAccount);

    assert.ok(
      minterBalanceAfter.eq(minterBalanceBefore),
      "Tokens in minter's account changed"
    );
    assert.ok(
      accountBalanceAfter.lt(accountBalanceBefore),
      "Tokens in the sender's account didn't decrement"
    );
  });

  it('transfer tokens from one account to another', async () => {
    await this.token.transfer(anotherAccount, new BN(2000000), {
      from: initialHolder,
    });
    const accountBalanceBefore = await this.token.balanceOf(anotherAccount);
    const recipientBalanceBefore = await this.token.balanceOf(recipient);

    await this.token.transfer(recipient, new BN(1000000), {
      from: anotherAccount,
    });

    const accountBalanceAfter = await this.token.balanceOf(anotherAccount);
    const recipientBalanceAfter = await this.token.balanceOf(recipient);

    assert.ok(
      accountBalanceAfter.lt(accountBalanceBefore),
      "Tokens in sender's account didn't decrement"
    );
    assert.ok(
      recipientBalanceAfter.gt(recipientBalanceBefore),
      "Tokens in the receiving account didn't increment"
    );
  });

  //account[1] to account[2] when acc[1] have no money - to fail
  xit('not allowed to transfer tokens from acount with no tokens to another account', async () => {

  });

  //try to transfer tokens during the contract is paused - need to fail
  xit('not allowed to transfer tokens during the contract is paused', async () => { });
});
