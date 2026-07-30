#include <algorithm>
#include <iostream>
#include <numeric>
#include <vector>

int main() {
  const std::vector<int> values{4, 8, 15, 16, 23, 42};
  const int sum = std::accumulate(values.begin(), values.end(), 0);
  const auto largest = std::max_element(values.begin(), values.end());

  std::cout << "count=" << values.size() << "\n";
  std::cout << "sum=" << sum << "\n";
  std::cout << "max=" << *largest << "\n";
  return 0;
}
