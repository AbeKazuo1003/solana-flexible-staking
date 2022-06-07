/* eslint-disable */
const anchor = require('@project-serum/anchor');
const { TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token');
const utils = require('./utils');
const assert = require('assert');
const fs = require('fs');
const md5 = require('md5');

let program = anchor.workspace.ChicksStakingFlexible;

//Read the provider from the configured environmnet.
//represents an outside actor
//owns mints out of any other actors control, provides initial $$ to others
const envProvider = anchor.Provider.env();

//we allow this convenience var to change between default env and mock user(s)
//initially we are the outside actor
let provider = envProvider;
//convenience method to set in anchor AND above convenience var
//setting in anchor allows the rpc and accounts namespaces access
//to a different wallet from env
function setProvider(p) {
  provider = p;
  anchor.setProvider(p);
  program = new anchor.Program(program.idl, program.programId, p);
}
setProvider(provider);

const sleep = (ms) =>
  new Promise((resolve) => setTimeout(resolve, ms));

console.log("md5", md5('test'));

describe('step-staking', async () => {
  //hardcoded in program, read from test keys directory for testing
  let mintKey;
  let mintObject;
  let mintPubkey;

  //the program's vault for stored collateral against xToken minting
  let vaultPubkey;
  let vaultBump;

  //the program's account for stored initializer key and lock end date
  let stakingPubkey;
  let stakingBump;
  let lock_time = new anchor.BN(3600 * 24 * 8 * 7); //8 weeks
  let new_lock_time = new anchor.BN(5);
  let feePercent = 250; //new anchor.BN(250);

  //the user's staking account for stored deposit amount
  let userStakingPubkey;
  let userStakingBump;
  let userStakingPubkey2;
  let userStakingBump2;

  let hodlUserKey = new anchor.web3.PublicKey("FYFmPpquZRxLSiaYK6FDpFZseYxt7FmxqhfNPjjmHdiR");
  console.log("hodlUserKey", hodlUserKey.toString());

  it('Is initialized!', async () => {
    //setup logging event listeners
    program.addEventListener('PriceChange', (e, s) => {
      console.log('Price Change In Slot ', s);
      console.log('From', e.oldStepPerXstepE9.toString());
      console.log('From', e.oldStepPerXstep.toString());
      console.log('To', e.newStepPerXstepE9.toString());
      console.log('To', e.newStepPerXstep.toString());
    });

    //this already exists in ecosystem
    //test step token hardcoded in program, mint authority is wallet for testing
    let rawdata = fs.readFileSync(
      'tests/keys/step-teST1ieLrLdr4MJPZ7i8mgSCLQ7rTrPRjNnyFdHFaz9.json'
    );
    let keyData = JSON.parse(rawdata);
    mintKey = anchor.web3.Keypair.fromSecretKey(new Uint8Array(keyData));
    mintObject = await utils.createMint(
      mintKey,
      provider,
      provider.wallet.publicKey,
      null,
      9,
      TOKEN_PROGRAM_ID
    );
    mintPubkey = mintObject.publicKey;

    [vaultPubkey, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
      [mintPubkey.toBuffer()],
      program.programId
    );

    [vaultPubkey, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
      [mintPubkey.toBuffer()],
      program.programId
    );

    [stakingPubkey, stakingBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(anchor.utils.bytes.utf8.encode('staking'))],
        program.programId
      );

    await program.rpc.initialize(vaultBump, stakingBump, lock_time, feePercent, {
      accounts: {
        tokenMint: mintPubkey,
        tokenVault: vaultPubkey,
        stakingAccount: stakingPubkey,
        initializer: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
    });
  });

  let walletTokenAccount;

  it('Mint test tokens', async () => {
    walletTokenAccount = await mintObject.createAssociatedTokenAccount(
      provider.wallet.publicKey
    );
    await utils.mintToAccount(
      provider,
      mintPubkey,
      walletTokenAccount,
      1000_000_000_000
    );
  });

  // it('Swap token for xToken', async () => {
  //   const handle1 = 'test';
  //   const key1 = md5(handle1);
  //   [userStakingPubkey, userStakingBump] =
  //     await anchor.web3.PublicKey.findProgramAddress(
  //       [hodlUserKey.toBuffer(), key1],
  //       program.programId
  //     );
  //
  //   await program.rpc.stakeByService(
  //     vaultBump,
  //     stakingBump,
  //     userStakingBump,
  //     key1,
  //     new anchor.BN(100_000_000_000),
  //     {
  //       accounts: {
  //         tokenMint: mintPubkey,
  //         tokenFrom: walletTokenAccount,
  //         tokenFromAuthority: provider.wallet.publicKey,
  //         targetUserAccount: hodlUserKey,
  //         tokenVault: vaultPubkey,
  //         stakingAccount: stakingPubkey,
  //         userStakingAccount: userStakingPubkey,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       },
  //     }
  //   );
  //
  //   let userStakingAccount = await program.account.userStakingAccount.fetch(
  //     userStakingPubkey
  //   );
  //   let amount = new anchor.BN(100_000_000_000);
  //
  //   assert.strictEqual(parseInt(userStakingAccount.amount), amount.toNumber());
  //   assert.strictEqual(
  //     await getTokenBalance(walletTokenAccount),
  //     900_000_000_000
  //   );
  //   assert.strictEqual(parseInt(userStakingAccount.amount), amount.toNumber());
  //   assert.strictEqual(
  //     parseInt(userStakingAccount.xTokenAmount),
  //     amount.toNumber()
  //   );
  //   assert.strictEqual(await getTokenBalance(vaultPubkey), 100_000_000_000);
  // });

  it('First staking', async () => {
    const handle1 = 'test1';
    const key1 = md5(handle1);
    [userStakingPubkey, userStakingBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [provider.wallet.publicKey.toBuffer(), key1],
        program.programId
      );

    await program.rpc.stake(
      vaultBump,
      stakingBump,
      userStakingBump,
      key1,
      new anchor.BN(100_000_000_000),
      {
        accounts: {
          tokenMint: mintPubkey,
          tokenFrom: walletTokenAccount,
          tokenFromAuthority: provider.wallet.publicKey,
          tokenVault: vaultPubkey,
          stakingAccount: stakingPubkey,
          userStakingAccount: userStakingPubkey,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
      }
    );

    let userStakingAccount = await program.account.userStakingAccount.fetch(
      userStakingPubkey
    );
  });


  it('Redeem xToken for token before lock end time', async () => {
    console.log("xTokenFromAuthority", provider.wallet.publicKey.toString())
    console.log("vaultPubkey", vaultPubkey.toString())
    console.log("stakingPubkey", stakingPubkey.toString())
    console.log("userStakingPubkey", userStakingPubkey.toString())
    const handle1 = 'test1';
    const key1 = md5(handle1);
    await program.rpc.unstake(
      vaultBump,
      stakingBump,
      userStakingBump,
      key1,
      new anchor.BN(100_000_000_000),
      {
        accounts: {
          tokenMint: mintPubkey,
          xTokenFromAuthority: provider.wallet.publicKey,
          tokenVault: vaultPubkey,
          stakingAccount: stakingPubkey,
          userStakingAccount: userStakingPubkey,
          tokenTo: walletTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      }
    );

    // console.log("checking --- result");
    //
    // // let userStakingAccount = await program.account.userStakingAccount.fetch(
    // //   userStakingPubkey
    // // );
    //
    // assert.strictEqual(
    //   await getTokenBalance(walletTokenAccount),
    //   975_000_000_000
    // );
    // // assert.strictEqual(parseInt(userStakingAccount.amount), 0);
    // // assert.strictEqual(parseInt(userStakingAccount.xTokenAmount), 0);
    // assert.strictEqual(await getTokenBalance(vaultPubkey), 25_000_000_000);
  });

  it('Update lock end date', async () => {
    await program.rpc.updateLockTime(stakingBump, new_lock_time, {
      accounts: {
        initializer: provider.wallet.publicKey,
        stakingAccount: stakingPubkey,
      },
    });

    let stakingAccount = await program.account.stakingAccount.fetch(
      stakingPubkey
    );
    console.log("stakingAccount.lock_time", stakingAccount);
    assert.strictEqual(
      parseInt(stakingAccount.lockTime),
      new_lock_time.toNumber()
    );
  });


  it('Update fee', async () => {
    await program.rpc.updateFeePercent(stakingBump, 350, {
      accounts: {
        initializer: provider.wallet.publicKey,
        stakingAccount: stakingPubkey,
      },
    });

    let stakingAccount = await program.account.stakingAccount.fetch(
      stakingPubkey
    );
    console.log("stakingAccount.lock_time", stakingAccount);
    assert.strictEqual(
      parseInt(stakingAccount.feePercent),
      350
    );
  });

  it('Second staking', async () => {
    const handle2 = 'test2';
    const key2 = md5(handle2);
    [userStakingPubkey2, userStakingBump2] =
      await anchor.web3.PublicKey.findProgramAddress(
        [provider.wallet.publicKey.toBuffer(), key2],
        program.programId
      );

    await program.rpc.stake(
      vaultBump,
      stakingBump,
      userStakingBump2,
      key2,
      new anchor.BN(200_000_000_000),
      {
        accounts: {
          tokenMint: mintPubkey,
          tokenFrom: walletTokenAccount,
          tokenFromAuthority: provider.wallet.publicKey,
          tokenVault: vaultPubkey,
          stakingAccount: stakingPubkey,
          userStakingAccount: userStakingPubkey2,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
      }
    );

    // let userStakingAccount2 = await program.account.userStakingAccount.fetch(
    //   userStakingPubkey2
    // );
    // let amount = new anchor.BN(200_000_000_000);
    //
    // assert.strictEqual(parseInt(userStakingAccount2.amount), amount.toNumber());
    // assert.strictEqual(
    //   await getTokenBalance(walletTokenAccount),
    //   775_000_000_000
    // );
    // assert.strictEqual(parseInt(userStakingAccount2.amount), amount.toNumber());
    // assert.strictEqual(
    //   parseInt(userStakingAccount2.xTokenAmount),
    //   amount.toNumber()
    // );
    // assert.strictEqual(await getTokenBalance(vaultPubkey), 225_000_000_000);
  });

  it('Redeem xToken after lock end time', async () => {
    await sleep(6000);
    console.log("xTokenFromAuthority", provider.wallet.publicKey.toString())
    console.log("vaultPubkey", vaultPubkey.toString())
    console.log("stakingPubkey", stakingPubkey.toString())
    console.log("userStakingPubkey", userStakingPubkey2.toString())
    const handle2 = 'test2';
    const key2 = md5(handle2);
    await program.rpc.unstake(
      vaultBump,
      stakingBump,
      userStakingBump2,
      key2,
      new anchor.BN(200_000_000_000),
      {
        accounts: {
          tokenMint: mintPubkey,
          xTokenFromAuthority: provider.wallet.publicKey,
          tokenVault: vaultPubkey,
          stakingAccount: stakingPubkey,
          userStakingAccount: userStakingPubkey2,
          tokenTo: walletTokenAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        },
      }
    );

    // console.log("checking --- result");
    //
    // // let userStakingAccount = await program.account.userStakingAccount.fetch(
    // //   userStakingPubkey
    // // );
    //
    // assert.strictEqual(
    //   await getTokenBalance(walletTokenAccount),
    //   1000_000_000_000
    // );
    // // assert.strictEqual(parseInt(userStakingAccount.amount), 0);
    // // assert.strictEqual(parseInt(userStakingAccount.xTokenAmount), 0);
    // assert.strictEqual(await getTokenBalance(vaultPubkey), 0);
  });

});

async function getTokenBalance(pubkey) {
  return parseInt(
    (await provider.connection.getTokenAccountBalance(pubkey)).value.amount
  );
}
