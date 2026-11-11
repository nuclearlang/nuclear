// Word frequencies, written by hand in idiomatic TypeScript (the same program as ../wordfreq.nuc).
const WORDS = ['alpha', 'beta', 'gamma', 'delta', 'epsilon', 'zeta', 'eta', 'theta', 'iota', 'kappa', 'lambda', 'mu', 'nu', 'xi', 'omicron', 'pi'];

function next(seed: number): number {
  return (seed * 48271) % 2147483647;
}

export function main(): void {
  const counts = new Map<string, number>();
  let seed = 42;
  let total = 0;
  for (let i = 0; i < 2000000; i++) {
    seed = next(seed);
    const w = WORDS[seed % WORDS.length];
    counts.set(w, (counts.get(w) ?? 0) + 1);
    total++;
  }
  const ranked = [...counts.keys()].sort(
    (a, b) => (999999 - counts.get(a)!) * 100 + WORDS.indexOf(a) - ((999999 - counts.get(b)!) * 100 + WORDS.indexOf(b)),
  );
  for (const w of ranked.slice(0, 5)) console.log(`${w} ${counts.get(w)}`);
  console.log(`total ${total} distinct ${counts.size}`);
}

main();
