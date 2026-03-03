import { BN } from '@coral-xyz/anchor';

// TODO: math.ts is currently only exported, not used internally. 
// Function signatures kept for backward compatibility.
export function safeAdd(a: number, b: number): number {
  const bnA = new BN(a);
  const bnB = new BN(b);
  const result = bnA.add(bnB);
  if (result.gt(new BN(Number.MAX_SAFE_INTEGER))) throw new Error('Math overflow');
  return result.toNumber();
}

export function safeSub(a: number, b: number): number {
  const bnA = new BN(a);
  const bnB = new BN(b);
  if (bnB.gt(bnA)) throw new Error('Math underflow');
  return bnA.sub(bnB).toNumber();
}

export function safeMul(a: number, b: number): number {
  if (a === 0 || b === 0) return 0;
  const bnA = new BN(a);
  const bnB = new BN(b);
  const result = bnA.mul(bnB);
  if (result.gt(new BN(Number.MAX_SAFE_INTEGER))) throw new Error('Math overflow');
  return result.toNumber();
}
