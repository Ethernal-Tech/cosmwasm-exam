package main

import (
	"fmt"
	"proof-generator/internal/domain")

func main() {
	fmt.Println("---------------------------Generator 1---------------------------")

	fields1 := [][]bool {
		{false, false, false},
		{false, true, false},
		{false, false, false},
	}

	board1 := domain.NewBoard(fields1)
	generator1 := domain.NewGenerator(board1)

	fmt.Println(generator1.MerkleTree)

	data1, proof1 := generator1.GenerateProof(domain.Field{Row: 1, Column: 1})

	fmt.Println(data1)
	fmt.Println(proof1)

	fmt.Println(generator1.VerifyProof(domain.Field{Row: 1, Column: 1}, proof1))

	fmt.Println("---------------------------Generator 2---------------------------")

	fields2 := [][]bool {
		{false, true, false},
		{false, false, false},
		{false, false, false},
	}

	board2 := domain.NewBoard(fields2)
	generator2 := domain.NewGenerator(board2)

	fmt.Println(generator2.MerkleTree)

	data2, proof2 := generator2.GenerateProof(domain.Field{Row: 1, Column: 0})

	fmt.Println(data2)
	fmt.Println(proof2)

	fmt.Println(generator2.VerifyProof(domain.Field{Row: 1, Column: 0}, proof2))

}