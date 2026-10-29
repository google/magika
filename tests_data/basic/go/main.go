package main

import "fmt"

func add(a, b int) int {
	return a + b
}

func main() {
	result := add(3, 4)
	fmt.Printf("3 + 4 = %d\n", result)
}
