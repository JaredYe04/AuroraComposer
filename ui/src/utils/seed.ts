/** New session seed from current timestamp (unique on each app launch). */
export function sessionSeed(): number {
  return Date.now();
}

/** Roll a new random seed (dice button). */
export function rollSeed(): number {
  const mix = Math.floor(Math.random() * 0x7fffffff);
  return Date.now() ^ mix;
}

/** Clamp manual seed input to a non-negative safe integer. */
export function normalizeSeed(value: number): number {
  if (!Number.isFinite(value) || value < 0) return sessionSeed();
  return Math.min(Math.floor(value), Number.MAX_SAFE_INTEGER);
}
