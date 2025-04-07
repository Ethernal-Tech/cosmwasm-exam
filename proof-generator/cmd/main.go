package main

import (
	"fmt"
	"os"
	"proof-generator/internal/domain"
	"proof-generator/internal/game"
)

func main() {
	currentPlayer := "player1"
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
			fmt.Println(currentPlayer)
			if currentPlayer == "player1" {
				game.Play(currentPlayer, &player1Generator)
				currentPlayer = "player2"
			} else {
				game.Play(currentPlayer, &player2Generator)
				currentPlayer = "player1"
			}
		case 3:
			fmt.Println("Bye!")
			os.Exit(0)
		default:
			fmt.Println("Unknown option")
		}
	}
}