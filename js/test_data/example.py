def print_primes(max_n: int) -> None:
    for i in range(2, max_n + 1):
        if is_prime(i):
            print(i)


def is_prime(n: int) -> bool:
    for i in range(2, n // 2 + 1):
        if n % i == 0:
            return False
    return True
