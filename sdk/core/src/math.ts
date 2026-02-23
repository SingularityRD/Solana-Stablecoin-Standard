export function safeAdd(a: number, b: number): number {
  const result = a + b;
  if (result < a) throw new Error('Math overflow');
  return result;
}

export function safeSub(a: number, b: number): number {
  if (b > a) throw new Error('Math underflow');
  return a - b;
}

export function safeMul(a: number, b: number): number {
  if (a === 0 || b === 0) return 0;
  const result = a * b;
  if (a > 0 && b > 0 && result / a !== b) throw new Error('Math overflow');
  return result;
}
