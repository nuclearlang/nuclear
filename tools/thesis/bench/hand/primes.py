"""Primes below 1 000 000 by trial division, written by hand in idiomatic Python (the same program as ../primes.nuc)."""


def is_prime(n: int) -> bool:
    if n < 2:
        return False
    i = 2
    while i * i <= n:
        if n % i == 0:
            return False
        i += 1
    return True


def main() -> None:
    count = 0
    total = 0
    largest = 0
    for n in range(2, 1000000):
        if is_prime(n):
            count += 1
            total += n
            largest = n
    print(f"count {count} sum {total} largest {largest}")


main()
