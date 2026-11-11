// Primes below 1 000 000 by trial division, written by hand in idiomatic Dart (the same program as ../primes.nuc).
bool isPrime(int n) {
  if (n < 2) return false;
  for (var i = 2; i * i <= n; i++) {
    if (n % i == 0) return false;
  }
  return true;
}

void main() {
  var count = 0;
  var sum = 0;
  var largest = 0;
  for (var n = 2; n < 1000000; n++) {
    if (isPrime(n)) {
      count++;
      sum += n;
      largest = n;
    }
  }
  print('count $count sum $sum largest $largest');
}
