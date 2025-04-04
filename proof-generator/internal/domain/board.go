package domain

type Board struct {
	Fields [][]bool
}

func NewBoard(fields [][]bool) *Board {
	return &Board{
		Fields: fields,
	}
}