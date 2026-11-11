// Word frequencies, written by hand in idiomatic Dart (the same program as ../wordfreq.nuc).
const words = ['alpha', 'beta', 'gamma', 'delta', 'epsilon', 'zeta', 'eta', 'theta', 'iota', 'kappa', 'lambda', 'mu', 'nu', 'xi', 'omicron', 'pi'];

int next(int seed) => (seed * 48271) % 2147483647;

void main() {
  final counts = <String, int>{};
  var seed = 42;
  var total = 0;
  for (var i = 0; i < 2000000; i++) {
    seed = next(seed);
    final w = words[seed % words.length];
    counts[w] = (counts[w] ?? 0) + 1;
    total++;
  }
  int key(String w) => (999999 - counts[w]!) * 100 + words.indexOf(w);
  final ranked = counts.keys.toList()..sort((a, b) => key(a).compareTo(key(b)));
  for (final w in ranked.take(5)) {
    print('$w ${counts[w]}');
  }
  print('total $total distinct ${counts.length}');
}
