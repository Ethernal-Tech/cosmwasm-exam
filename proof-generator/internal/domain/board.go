package domain

type Board struct {
	fields [][]bool
}

func NewBoard(fields [][]bool) *Board {
	return &Board{
		fields: fields,
	}
}