package main

import (
	"fmt"
	"strings"
)

func titleWord(word string) string {
	if word == "" {
		return word
	}
	return strings.ToUpper(word[:1]) + word[1:]
}

func titleWords(words []string) []string {
	titled := make([]string, 0, len(words))
	for _, word := range words {
		titled = append(titled, titleWord(word))
	}
	return titled
}

func main() {
	words := []string{"magika", "basic", "sample"}
	fmt.Println(strings.Join(titleWords(words), " "))
}
