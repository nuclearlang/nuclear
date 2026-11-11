"""Word frequencies, written by hand in idiomatic Python (the same program as ../wordfreq.nuc)."""
WORDS = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa", "lambda", "mu", "nu", "xi", "omicron", "pi"]


def next_seed(seed: int) -> int:
    return (seed * 48271) % 2147483647


def main() -> None:
    counts: dict[str, int] = {}
    seed = 42
    total = 0
    for _ in range(2000000):
        seed = next_seed(seed)
        w = WORDS[seed % len(WORDS)]
        counts[w] = counts.get(w, 0) + 1
        total += 1
    ranked = sorted(counts, key=lambda w: (999999 - counts[w]) * 100 + WORDS.index(w))
    for w in ranked[:5]:
        print(f"{w} {counts[w]}")
    print(f"total {total} distinct {len(counts)}")


main()
