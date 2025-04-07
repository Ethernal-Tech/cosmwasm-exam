package main

import (
	"fmt"
	"os"
	"proof-generator/internal/domain"
	"proof-generator/internal/game"
)

func main() {
	var player1Generator domain.Generator
	var player2Generator domain.Generator

	for {
		fmt.Println("\n=== Battleship Game ===")
		fmt.Println("1. Init game")
		fmt.Println("2. Play move")
		fmt.Println("3. Exit")
		fmt.Print("Choose option: ")

		var choice int
		_, err := fmt.Scanln(&choice)
		if err != nil {
			fmt.Println("Invalid input")
			continue
		}

		switch choice {
		case 1:
			game.InitGame(&player1Generator, &player2Generator)
		case 2:
			game.Play()
		case 3:
			fmt.Println("Bye!")
			os.Exit(0)
		default:
			fmt.Println("Unknown option")
		}
	}
}