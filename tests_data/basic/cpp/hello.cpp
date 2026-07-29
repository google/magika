#include <iostream>
#include <vector>

bool is_prime(int n) {
    if (n < 2) return false;
    for (int i = 2; i * i <= n; i++) {
        if (n % i == 0) return false;
    }
    return true;
}

int main() {
    std::vector<int> primes;
    for (int i = 2; i <= 50; i++) {
        if (is_prime(i)) primes.push_back(i);
    }
    for (int p : primes) {
        std::cout << p << " ";
    }
    std::cout << std::endl;
    return 0;
}
