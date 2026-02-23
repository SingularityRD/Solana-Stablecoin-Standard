import { PublicKey } from '@solana/web3.js';

const PROGRAM_ID = new PublicKey('SSSToken11111111111111111111111111111111111');

export function findStablecoinPda(assetMint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from('stablecoin'), assetMint.toBuffer()], PROGRAM_ID);
}

export function findMinterPda(stablecoin: PublicKey, minter: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from('minter'), stablecoin.toBuffer(), minter.toBuffer()], PROGRAM_ID);
}

export function findRolePda(stablecoin: PublicKey, account: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from('role'), stablecoin.toBuffer(), account.toBuffer()], PROGRAM_ID);
}

export function findBlacklistPda(stablecoin: PublicKey, account: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from('blacklist'), stablecoin.toBuffer(), account.toBuffer()], PROGRAM_ID);
}
