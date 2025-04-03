package main

import (
	"fmt"
	"proof-generator/internal/domain")

func main() {
	fields := [][]bool{
		{true, false, true},
		{false, true, false},
		{true, true, false},
	}

	board := domain.NewBoard(fields)
	generator := domain.NewGenerator(board)

	fmt.Println(generator.MerkleTree)
}