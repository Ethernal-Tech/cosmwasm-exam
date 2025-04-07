package domain

import "fmt"

type Board struct {
	Fields [][]bool
}

func NewBoard(fields [][]bool) *Board {
	return &Board{
		Fields: fields,
	}
}

func (board *Board) String() string {
	return fmt.Sprintf("%d", len(board.Fields))
}