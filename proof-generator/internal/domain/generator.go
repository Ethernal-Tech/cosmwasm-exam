package domain

import "strconv"

type Field struct {
	Row int;
	Column int;
}

type Generator struct {
	Board *Board;
	Index map[Field]int
	MerkleTree *MerkleTree;
 }

func NewGenerator(board *Board) *Generator {
	var data []string
	index := make(map[Field]int)

	for i, row := range board.Fields {
		for j, value := range row {
			data = append(data, strconv.FormatBool(value))
			index[Field{Row: i, Column: j}] = len(data) - 1
		}
	}

	merkleTree := NewMerkleTree(data)

	return &Generator {
		Board: board,
		MerkleTree: merkleTree,
		Index: index,
	}
}

func (generator *Generator) GenerateProof(field Field) (string, []ProofStep) {
	return generator.MerkleTree.GenerateProof(generator.Index[field])
}

func (generator *Generator) VerifyProof(field Field, proof []ProofStep) bool {
	data := strconv.FormatBool(generator.Board.Fields[field.Row][field.Column])
	return generator.MerkleTree.VerifyProof(data, proof) 
}