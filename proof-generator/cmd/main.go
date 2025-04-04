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

	data, proof := generator.GenerateProof(domain.Field{Row: 0, Column: 0})

	fmt.Println(data)
	fmt.Println(proof)

	fmt.Println(generator.VerifyProof(domain.Field{Row: 0, Column: 0}, proof))
}