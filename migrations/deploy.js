/* eslint-disable */
// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const { TOKEN_PROGRAM_ID } = require("@solana/spl-token");
const fs = require('fs');

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  let program = anchor.workspace.ChicksStakingFlexible;

  let mintPubkey;
  if (program.programId.toString() === 'XASp8U7ZSJ9sJfUaMKk5dxuw3Hf4xkLPBcoHZ4seoC1') {
    mintPubkey = new anchor.web3.PublicKey("FUnRfJAJiTtpSGP9uP5RtFm4QPsYUPTVgSMoYrgVyNzQ"); // token address
  } else {
    mintPubkey = new anchor.web3.PublicKey("cxxShYRVcepDudXhe7U62QHvw8uBJoKFifmzggGKVC2"); // token address
  }

  const [vaultPubkey, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
    [mintPubkey.toBuffer()],
    program.programId
  )

  const [stakingPubkey, stakingBump] =
  await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from(anchor.utils.bytes.utf8.encode('staking'))],
    program.programId
  )
  console.log('program id', program.programId.toString());
  console.log('vaultPubkey', vaultPubkey.toString(), vaultBump);
  console.log('stakingPubkey', stakingPubkey.toString(), stakingBump);

  console.log('Before');
  try {
    let stakingAccount = await program.account.stakingAccount.fetch(
      stakingPubkey
    );
    console.log('stakingAccount', stakingAccount);
  } catch (e) {
    console.log(e);
  }

  // init
  // const lockTime = new anchor.BN(3600 * 24 * 8 * 7) // 8 weeks
  // const fee_percent = 250;
  //
  // try {
  //   await program.rpc.initialize(vaultBump, stakingBump, lockTime, fee_percent, {
  //     accounts: {
  //       tokenMint: mintPubkey,
  //       tokenVault: vaultPubkey,
  //       stakingAccount: stakingPubkey,
  //       initializer: provider.wallet.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     },
  //   })
  // } catch(e) {
  //   console.log(e);
  // }

  // updateLockTime
  let new_lockTime = new anchor.BN(3600 * 24 * 7 * 8) // 8 weeks

  try {
    await program.rpc.updateLockTime(stakingBump, new_lockTime, {
      accounts: {
        initializer: provider.wallet.publicKey,
        stakingAccount: stakingPubkey,
      },
    })
  } catch(e) {
    console.log(e);
  }
  //
  // try {
  //   await program.rpc.updateFeePercent(stakingBump, 200, {
  //     accounts: {
  //       initializer: provider.wallet.publicKey,
  //       stakingAccount: stakingPubkey,
  //     },
  //   })
  // } catch(e) {
  //   console.log(e);
  // }

  console.log('After');
  try {
    let stakingAccount = await program.account.stakingAccount.fetch(
      stakingPubkey
    );
    console.log('stakingAccount', stakingAccount);
  } catch (e) {
    console.log(e);
  }
}
