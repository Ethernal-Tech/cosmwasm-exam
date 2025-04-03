package domain

import "strconv"

type Generator struct {
	Board *Board;
	MerkleTree *MerkleTree
 }

func NewGenerator(board *Board) *Generator {
	var data []string

	for _, row := range board.fields {
		for _, value := range row {
			data = append(data, strconv.FormatBool(value))
		}
	}

	merkleTree := NewMerkleTree(data)

	return &Generator {
		Board: board,
		MerkleTree: merkleTree,
	}
}