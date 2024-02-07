import assert from 'assert';
import { PublicKey } from '@solana/web3.js';
import { BN, Program, Provider, utils, web3 } from '@project-serum/anchor';
import { Metaplex } from '@metaplex-foundation/js';
import { GSO } from '@dual-finance/gso';
import { StakingOptions, STAKING_OPTIONS_PK} from '@dual-finance/staking-options';
import {
  createMint,
  createAssociatedTokenAccount,
  createTokenAccount,
  mintToAccount,
} from './utils/utils';
import { Gso as GSO_type } from '../target/types/gso';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

const { getAccount } = require('@solana/spl-token');

const anchor = require('@project-serum/anchor');

describe('gso', () => {
  anchor.setProvider(anchor.Provider.env());
  const provider: Provider = anchor.Provider.env();
  const program = anchor.workspace.Gso;

  const gsoHelper = new GSO(provider.connection.rpcEndpoint);
  const soHelper = new StakingOptions(provider.connection.rpcEndpoint);

  const metaplexId = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');
  const soKey = new PublicKey('4yx1NJ4Vqf2zT1oVLk4SySBhhDJXmXFt88ncm4gPxtL7');

  const EXPIRATION_DELAY_SEC: number = 100;
  let optionExpiration: number = Date.now() / 1_000 + EXPIRATION_DELAY_SEC;
  let lockupPeriodEnd: number = Date.now() / 1_000 + EXPIRATION_DELAY_SEC;
  let subscriptionPeriodEnd: number = Date.now() / 1_000 + EXPIRATION_DELAY_SEC;
  const numTokensInPeriod: number = 1_000_000;
  const numStake: number = 1_000_000;
  const lockupRatioTokensPerMillionLots: number = 200_000;
  let projectName: string = `TEST_${optionExpiration.toString()}`;
  const strikePrice: number = 50_000;
  const lotSize: number = 1;

  let gsoState: PublicKey;
  let soBaseMint: PublicKey;
  let soQuoteMint: PublicKey;
  let soBaseAccount: PublicKey;
  let soQuoteAccount: PublicKey;
  let xBaseMint: PublicKey;

  let soOptionMint: PublicKey;
  let userBaseAccount: PublicKey;
  let soUserOptionAccount: PublicKey;

  async function configure(lockupEndToSubscriptionPeriodOffset = 0) {
    console.log('Configuring');
    projectName = `TEST_${optionExpiration.toString()}`;

    gsoState = await gsoHelper.state(projectName);

    soBaseMint = await createMint(provider, undefined);
    soQuoteMint = await createMint(provider, undefined);

    console.log('Creating base account');
    soBaseAccount = await createAssociatedTokenAccount(
      provider,
      soBaseMint,
      provider.wallet.publicKey,
    );
    console.log('Minting base tokens');
    await mintToAccount(
      provider,
      soBaseMint,
      soBaseAccount,
      new anchor.BN(numTokensInPeriod),
      provider.wallet.publicKey,
    );

    console.log('Creating quote account');
    soQuoteAccount = await createAssociatedTokenAccount(
      provider,
      soQuoteMint,
      provider.wallet.publicKey,
    );

    soOptionMint = await soHelper.soMint(strikePrice, `GSO${projectName}`, soBaseMint);

    xBaseMint = await gsoHelper.xBaseMint(gsoState);
    subscriptionPeriodEnd = Date.now() / 1_000 + EXPIRATION_DELAY_SEC;
    lockupPeriodEnd = subscriptionPeriodEnd + lockupEndToSubscriptionPeriodOffset;
    optionExpiration = subscriptionPeriodEnd + lockupEndToSubscriptionPeriodOffset;

    console.log('Creating config instruction');
    const configInstruction = await gsoHelper.createConfigInstruction(
      lockupRatioTokensPerMillionLots,
      lockupPeriodEnd,
      optionExpiration,
      subscriptionPeriodEnd,
      new anchor.BN(numTokensInPeriod),
      projectName,
      new anchor.BN(strikePrice),
      provider.wallet.publicKey,
      soBaseMint,
      soQuoteMint,
      soBaseAccount,
      soQuoteAccount,
      lotSize,
    );

    console.log('Sending config instruction');
    const tx = new anchor.web3.Transaction();
    tx.add(configInstruction);
    await provider.send(tx);
  }

  async function stake() {
    console.log('Staking');

    // This is another account, not the same as used before.
    userBaseAccount = await createTokenAccount(
      provider,
      soBaseMint,
      provider.wallet.publicKey,
    );
    await mintToAccount(
      provider,
      soBaseMint,
      userBaseAccount,
      new anchor.BN(numStake),
      provider.wallet.publicKey,
    );

    console.log('Creating option account');
    soUserOptionAccount = await createAssociatedTokenAccount(
      provider,
      soOptionMint,
      provider.wallet.publicKey,
    );

    console.log('Creating xbase account');
    // userXBaseAccount
    await createAssociatedTokenAccount(
      provider,
      xBaseMint,
      provider.wallet.publicKey,
    );

    console.log('Creating stake instruction');
    const stakeInstruction = await gsoHelper.createStakeInstruction(
      numStake,
      projectName,
      provider.wallet.publicKey,
      soBaseMint,
      userBaseAccount,
    );

    const tx = new anchor.web3.Transaction();
    tx.add(stakeInstruction);
    await provider.send(tx);
  }

  async function unstake() {
    console.log('Unstaking');

    const unstakeInstruction = await gsoHelper.createUnstakeInstruction(
      numStake,
      projectName,
      provider.wallet.publicKey,
      userBaseAccount,
    );

    const tx = new anchor.web3.Transaction();
    tx.add(unstakeInstruction);
    await provider.send(tx);
  }

  async function withdraw() {
    console.log('Withdrawing');

    const withdrawInstruction = await gsoHelper.createWithdrawInstruction(
      projectName,
      soBaseMint,
      provider.wallet.publicKey,
      soBaseAccount,
    );

    const tx = new anchor.web3.Transaction();
    tx.add(withdrawInstruction);
    await provider.send(tx);
  }

  async function nameTokens() {
    console.log('Naming token');

    const [optionMetadata, _optionMintMetadataBump] = (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode('metadata')),
          metaplexId.toBuffer(),
          soOptionMint.toBuffer(),
        ],
        metaplexId,
      ));

    const [xBaseMetadata, _xBaseMintMetadataAccountBump] = (
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(anchor.utils.bytes.utf8.encode('metadata')),
          metaplexId.toBuffer(),
          xBaseMint.toBuffer(),
        ],
        metaplexId,
      ));
    const [soAuthority, _soAuthorityBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode('gso')),
        gsoState.toBuffer(),
      ],
      program.programId,
    );
    const soState = await soHelper.state(`GSO${projectName}`, soBaseMint);

    await program.rpc.nameTokens(
      {
        accounts: {
          authority: provider.wallet.publicKey,
          gsoState,
          xBaseMint,
          xBaseMetadata,
          soAuthority,
          soState,
          soOptionMint,
          optionMetadata,
          stakingOptionsProgram: soKey,
          tokenMetadataProgram: metaplexId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      },
    );
  }

  it('Configure', async () => {
    try {
      await configure();
    } catch (err) {
      console.log(err);
      assert(false);
    }
    const soBaseAccountAccount = await getAccount(provider.connection, soBaseAccount);
    assert.equal(soBaseAccountAccount.amount, 0);
  });

  it('Stake', async () => {
    try {
      await stake();
    } catch (err) {
      console.log(err);
      assert(false);
    }
    const userBaseAccountAccount = await getAccount(provider.connection, userBaseAccount);
    assert.equal(userBaseAccountAccount.amount, 0);

    const soUserOptionAccountAccount = await getAccount(provider.connection, soUserOptionAccount);
    assert.equal(
      soUserOptionAccountAccount.amount,
      numStake * (lockupRatioTokensPerMillionLots / 1_000_000),
    );
  });

  it('Unstake', async () => {
    console.log('Waiting before unstake');

    // Wait to be sure the subscription period has ended.
    await new Promise((r) => setTimeout(r, EXPIRATION_DELAY_SEC * 1_000));

    try {
      await unstake();
    } catch (err) {
      console.log(err);
      assert(false);
    }
    const userBaseAccountAccount = await getAccount(provider.connection, userBaseAccount);
    assert.equal(userBaseAccountAccount.amount, numStake);
  });

  it('Withdraw', async () => {
    console.log('Withdraw collateral tokens');

    // Already waited
    try {
      await withdraw();
    } catch (err) {
      console.log(err);
      assert(false);
    }
    const soBaseAccountAccount = await getAccount(provider.connection, soBaseAccount);
    assert.equal(Number(soBaseAccountAccount.amount), numTokensInPeriod);
  });

  it('UnstakeBeforeUnlockFail', async () => {
    await configure();
    await stake();
    // Do not wait until lockup has expired.
    try {
      await unstake();
      assert(false);
    } catch (err) {
      console.log(err);
    }
  });

  it('UnstakeBetweenSubscriptionPeriodEndAndUnlockFail', async () => {
    await configure(EXPIRATION_DELAY_SEC);
    await stake();

    // Only wait for the subscription period to end.
    console.log('Waiting for subscription period to end');
    await new Promise((r) => setTimeout(r, EXPIRATION_DELAY_SEC * 1_000));
    try {
      await unstake();
      assert(false);
    } catch (err) {
      console.log(err);
      assert(true);
    }

    console.log('Verifying subscription period ended');
    try {
      await stake();
      assert(false);
    } catch (err) {
      console.log(err);
      assert(true);
    }

    // After waiting more, it will succeed.
    console.log('Waiting for lockup to end');
    await new Promise((r) => setTimeout(r, EXPIRATION_DELAY_SEC * 1_000));
    try {
      await unstake();
    } catch (err) {
      assert(false);
    }
  });

  it('NameTokens', async () => {
    await configure();
    try {
      await nameTokens();
    } catch (err) {
      console.log(err);
      assert(false);
    }
    const metaplex = new Metaplex(provider.connection);
    const soNft = await metaplex.nfts().findByMint({ mintAddress: soOptionMint });
    assert.equal(soNft.name, `DUAL-GSO${projectName.substring(0, 15)}-5.00e4`);

    const xNft = await metaplex.nfts().findByMint({ mintAddress: xBaseMint });
    assert.equal(xNft.name, `DUAL-GSO-${projectName}`.substring(0, 24));
  });

  it('ConfigV2e2e', async () => {
    console.log('Configuring V2');
    projectName = `TEST_${optionExpiration.toString()}`;

    gsoState = await gsoHelper.state(projectName);
    soBaseMint = await createMint(provider, undefined);
    soQuoteMint = await createMint(provider, undefined);

    console.log('Creating base account');
    soBaseAccount = await createAssociatedTokenAccount(
      provider,
      soBaseMint,
      provider.wallet.publicKey,
    );
    console.log('Minting base tokens');
    await mintToAccount(
      provider,
      soBaseMint,
      soBaseAccount,
      new anchor.BN(numTokensInPeriod),
      provider.wallet.publicKey,
    );

    console.log('Creating quote account');
    soQuoteAccount = await createAssociatedTokenAccount(
      provider,
      soQuoteMint,
      provider.wallet.publicKey,
    );

    console.log('Creating config v2 instruction');

    const lockupRatioPerMillionLots = lockupRatioTokensPerMillionLots;
    const numTokensAtoms = new anchor.BN(numTokensInPeriod);
    const strikeAtomsPerLot = new anchor.BN(strikePrice);
    const authority = provider.wallet.publicKey;

    const xBaseMint = await gsoHelper.xBaseMint(gsoState);
    subscriptionPeriodEnd = Date.now() / 1_000 + EXPIRATION_DELAY_SEC;
    lockupPeriodEnd = subscriptionPeriodEnd;
    optionExpiration = subscriptionPeriodEnd;

    console.log('Creating config instruction');

    const [soAuthority, soAuthorityBump] = await web3.PublicKey.findProgramAddress(
      [
        Buffer.from(utils.bytes.utf8.encode('gso')),
        gsoState.toBuffer(),
      ],
      program.programId,
    );
    const so = new StakingOptions(provider.connection.rpcEndpoint);

    const soState = await so.state(`GSO${projectName}`, soBaseMint);
    const soBaseVault = await so.baseVault(`GSO${projectName}`, soBaseMint);
    const soOptionMint = await so.soMint(strikeAtomsPerLot, `GSO${projectName}`, soBaseMint);
    const baseVault = await gsoHelper.baseVault(gsoState);

    console.log('Sending config instruction');
    const otherMint = await createMint(provider, undefined);

    await program.rpc.configV2(
      new BN(1), /* period_num */
      new BN(lockupRatioPerMillionLots),
      new BN(lockupPeriodEnd),
      new BN(optionExpiration),
      new BN(subscriptionPeriodEnd),
      new BN(lotSize),
      numTokensAtoms,
      projectName,
      new BN(strikeAtomsPerLot),
      soAuthorityBump,
      {
        accounts: {
          authority,
          gsoState,
          soAuthority,
          soState,
          soBaseVault,
          soBaseAccount: soBaseAccount,
          soQuoteAccount: soQuoteAccount,
          soBaseMint: otherMint,
          soQuoteMint: soQuoteMint,
          soOptionMint,
          xBaseMint,
          baseVault,
          lockupMint: soBaseMint,
          stakingOptionsProgram: STAKING_OPTIONS_PK,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: web3.SystemProgram.programId,
          rent: web3.SYSVAR_RENT_PUBKEY,
        },
      },
    );

    await stake();

    // Wait to be sure the subscription period has ended.
    await new Promise((r) => setTimeout(r, EXPIRATION_DELAY_SEC * 1_000));

    try {
      await unstake();
    } catch (err) {
      console.log(err);
      assert(false);
    }
    const userBaseAccountAccount = await getAccount(provider.connection, userBaseAccount);
    assert.equal(userBaseAccountAccount.amount, numStake);
  });
});
