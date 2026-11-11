// Primes below 1 000 000 by trial division, written by hand in idiomatic TypeScript (the same program as ../primes.nuc).
function isPrime(n: number): boolean {
  if (n < 2) return false;
  for (let i = 2; i * i <= n; i++) {
    if (n % i === 0) return false;
  }
  return true;
}

export function main(): void {
  let count = 0;
  let sum = 0;
  let largest = 0;
  for (let n = 2; n < 1000000; n++) {
    if (isPrime(n)) {
      count++;
      sum += n;
      largest = n;
    }
  }
  console.log(`count ${count} sum ${sum} largest ${largest}`);
}

main();
