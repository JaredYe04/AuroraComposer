/** Safe fixed-point formatting for UI labels (avoids `.toFixed` on undefined). */
export function formatFixed(value: unknown, digits = 2, fallback = 0): string {
  const n =
    typeof value === 'number' && Number.isFinite(value)
      ? value
      : typeof fallback === 'number' && Number.isFinite(fallback)
        ? fallback
        : 0;
  return n.toFixed(digits);
}
