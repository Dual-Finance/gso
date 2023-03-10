import { Program, Provider, BN } from '@project-serum/anchor';
import { Keypair, PublicKey } from '@solana/web3.js';

const anchor = require('@project-serum/anchor');
const { TokenInstructions } = require('@project-serum/serum');
const { getAssociatedTokenAddress, createAssociatedTokenAccountInstruction, getMint } = require('@solana/spl-token');

const DEFAULT_MINT_DECIMALS = 6;

async function createMintInstructions(provider: Provider, authority: PublicKey, mint: PublicKey) {
  const instructions = [
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey: mint,
      space: 82,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(82),
      programId: TokenInstructions.TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeMint({
      mint,
      decimals: DEFAULT_MINT_DECIMALS,
      mintAuthority: authority,
    }),
  ];
  return instructions;
}

export async function createMint(provider: Provider, initialAuthority: PublicKey | undefined) {
  let authority = initialAuthority;
  if (authority === undefined) {
    authority = provider.wallet.publicKey;
  }
  const mint = anchor.web3.Keypair.generate();
  const instructions = await createMintInstructions(
    provider,
    authority,
    mint.publicKey,
  );

  const tx = new anchor.web3.Transaction();
  tx.add(...instructions);

  await provider.send(tx, [mint]);
  await new Promise((r) => setTimeout(r, 100));

  return mint.publicKey;
}

async function createTokenAccountInstrs(
  provider: Provider,
  newAccountPubkey: PublicKey,
  mint: PublicKey,
  owner: PublicKey,
  lamportsRequested: number,
) {
  let lamports = lamportsRequested;
  if (lamports === undefined) {
    lamports = await provider.connection.getMinimumBalanceForRentExemption(165);
  }
  return [
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey,
      space: 165,
      lamports,
      programId: TokenInstructions.TOKEN_PROGRAM_ID,
    }),
    TokenInstructions.initializeAccount({
      account: newAccountPubkey,
      mint,
      owner,
    }),
  ];
}

export async function createTokenAccount(provider: Provider, mint: PublicKey, owner: PublicKey) {
  const vault = anchor.web3.Keypair.generate();
  const tx = new anchor.web3.Transaction();
  tx.add(
    ...(await createTokenAccountInstrs(provider, vault.publicKey, mint, owner, undefined)),
  );
  await provider.send(tx, [vault]);
  await new Promise((r) => setTimeout(r, 100));
  return vault.publicKey;
}

export async function createAssociatedTokenAccount(
  provider,
  mint: PublicKey,
  owner: PublicKey,
) {
  const ata = await getAssociatedTokenAddress(mint, owner);
  const tx = new anchor.web3.Transaction();
  tx.add(await createAssociatedTokenAccountInstruction(owner, ata, owner, mint));
  await provider.send(tx);
  return ata;
}

async function createMintToAccountInstrs(
  mint: PublicKey,
  destination: PublicKey,
  amount: BN,
  mintAuthority: PublicKey,
) {
  return [
    TokenInstructions.mintTo({
      mint,
      destination,
      amount,
      mintAuthority,
    }),
  ];
}

export async function mintToAccount(
  provider: Provider,
  mint: PublicKey,
  destination: PublicKey,
  amount: BN,
  mintAuthority: PublicKey,
) {
  // mint authority is the provider
  const tx = new anchor.web3.Transaction();
  tx.add(
    ...(await createMintToAccountInstrs(
      mint,
      destination,
      amount,
      mintAuthority,
    )),
  );
  await provider.send(tx, []);
}

export function parsePriceAndExpiration(buf: Buffer) {
  // const overhead = buf.readBigUInt64LE(0);
  const strikePrice = Number(buf.readBigUInt64LE(8));
  const expiration = Number(buf.readBigUInt64LE(16));
  const mintPk = new PublicKey(buf.slice(24, 56));
  return {
    strikePrice,
    expiration,
    spl_mint: mintPk,
  };
}

export function toBeBytes(x: number) {
  const y = Math.floor(x / 2 ** 32);
  return Uint8Array.from(
    [y, y << 8, y << 16, y << 24, x, x << 8, x << 16, x << 24].map(
      (z) => z >>> 24,
    ),
  );
}

export function programPaidBy(provider: Provider, program: Program, payer: Keypair) {
  const newProvider = new anchor.Provider(
    provider.connection,
    new anchor.Wallet(payer),
    {},
  );
  return new anchor.Program(
    program.idl,
    program.programId,
    newProvider,
  );
}
