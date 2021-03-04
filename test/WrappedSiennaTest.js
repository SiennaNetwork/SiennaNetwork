const { BN } = require('@openzeppelin/test-helpers');
const { expect, assert } = require('chai');

const WrappedSienna = artifacts.require('WrappedSienna');

contract('WrappedSienna', (accounts) => {
  const [admin, bridgeAddress, anotherAccount1, anotherAccount2] = accounts;
  const name = 'Wrapped SIENNA';
  const symbol = 'wSIENNA';

  beforeEach(async () => {
    this.token = await WrappedSienna.new(admin);
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
    const result = await this.token.pause({ from: admin });
    assert.ok(result, 'Pause failed.');
  });

  it('unpaused is successfull', async () => {
    await this.token.pause({ from: admin });
    const result = await this.token.unpause({ from: admin });
    assert.ok(result, 'Unpaused failed.');
  });

  it('not allowed to pause from account with no pauser role', async () => {
    try {
      await this.token.pause({ from: anotherAccount1 });
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
      await this.token.pause({ from: admin });
      await this.token.unpause({ from: anotherAccount1 });
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
    const minterBalanceBefore = await this.token.balanceOf(admin);
    const accountBalanceBefore = await this.token.balanceOf(anotherAccount1);

    await this.token.transfer(anotherAccount1, new BN(1000000), {
      from: admin,
    });

    const minterBalanceAfter = await this.token.balanceOf(admin);
    const accountBalanceAfter = await this.token.balanceOf(anotherAccount1);

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
    await this.token.transfer(anotherAccount1, new BN(2000000), {
      from: admin,
    });
    const minterBalanceBefore = await this.token.balanceOf(admin);
    const accountBalanceBefore = await this.token.balanceOf(anotherAccount1);

    await this.token.transfer(admin, new BN(1000000), {
      from: anotherAccount1,
    });

    const minterBalanceAfter = await this.token.balanceOf(admin);
    const accountBalanceAfter = await this.token.balanceOf(anotherAccount1);

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
    await this.token.transfer(anotherAccount1, new BN(2000000), {
      from: admin,
    });
    const accountBalanceBefore = await this.token.balanceOf(anotherAccount1);
    const recipientBalanceBefore = await this.token.balanceOf(anotherAccount2);

    await this.token.transfer(anotherAccount2, new BN(1000000), {
      from: anotherAccount1,
    });

    const accountBalanceAfter = await this.token.balanceOf(anotherAccount2);
    const recipientBalanceAfter = await this.token.balanceOf(anotherAccount1);

    assert.ok(
      accountBalanceAfter.lt(accountBalanceBefore),
      "Tokens in sender's account didn't decrement"
    );
    assert.ok(
      recipientBalanceAfter.gt(recipientBalanceBefore),
      "Tokens in the receiving account didn't increment"
    );
  });

  it('not allowed to transfer tokens during the contract is paused', async () => {
    try {
      await this.token.transfer(anotherAccount1, new BN(2000000), {
        from: admin,
      });
      await this.token.pause({ from: admin });
      await this.token.transfer(anotherAccount2, new BN(1000000), {
        from: anotherAccount1,
      });
      assert.fail('The transaction should have thrown an error');
    } catch (err) {
      assert.include(
        err.message,
        'token transfer while paused',
        'The error message should contain "token transfer while paused"'
      );
    }
  });

  it('not allowed to transfer tokens from account with no tokens to another account', async () => {
    try {
      await this.token.transfer(anotherAccount1, new BN(1000000), {
        from: anotherAccount2,
      });
      assert.fail('The transaction should have thrown an error');
    } catch (err) {
      assert.include(
        err.message,
        'exceeds balance',
        'The error message should contain "exceeds balance"'
      );
    }
  });

});
