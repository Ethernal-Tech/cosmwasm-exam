package game

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"proof-generator/internal/domain"
	"time"
)

type PlayerFileMeta struct {
	Address string `json:"address"`
}

func PlayMove(player string, contractAddr string, x, y int, value bool, proof []domain. ProofStep) error {
	msg := map[string]interface{}{
		"play": map[string]interface{}{
			"field": [2]int{x, y},
			"value": value,
			"proof": proof,
		},
	}

	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return fmt.Errorf("failed to marshal play msg: %w", err)
	}

	cmd := exec.Command("wasmd", "tx", "wasm", "execute", contractAddr, string(msgBytes),
		"--from="+player,
		"--chain-id=localnet",
		"--keyring-backend=test",
		"--gas=auto", "--gas-adjustment=1.3",
		"--broadcast-mode=sync",
		"-y",
	)

	fmt.Printf("Playing move at (%d, %d)...\n", x, y)

	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("play move failed: %v\nOutput: %s", err, string(output))
	}

	fmt.Println("Move played:\n", string(output))
	return nil
}


func Play(playerName string, generator *domain.Generator) {
	contractAddr, err := LoadContractAddress()
	if err != nil {
		fmt.Println("Failed to load contract address:", err)
		return
	}

	playerFile := fmt.Sprintf("%s.json", playerName)
	playerData, err := LoadPlayerMeta(playerFile)
	if err != nil {
		fmt.Println("Failed to load player:", err)
		return
	}

	var x, y int
	fmt.Print("Enter X: ")
	fmt.Scanln(&x)
	fmt.Print("Enter Y: ")
	fmt.Scanln(&y)

	field := domain.Field{Row: x, Column: y}
	fmt.Println(generator.Board)
	value := generator.Board.Fields[x][y]
	_, proof := generator.GenerateProof(field)

	time.Sleep(3 * time.Second)
	err = PlayMove(playerData.Address, contractAddr, x, y, value, proof)
	if err != nil {
		fmt.Println("Move failed:", err)
		return
	}

	fmt.Println("Move sent successfully!")
}

func LoadPlayerMeta(path string) (PlayerFileMeta, error) {
	file, err := os.ReadFile(path)
	if err != nil {
		return PlayerFileMeta{}, err
	}
	var data PlayerFileMeta
	if err := json.Unmarshal(file, &data); err != nil {
		return PlayerFileMeta{}, err
	}
	return data, nil
}

func LoadContractAddress() (string, error) {
	file, err := os.ReadFile("contract.json")
	if err != nil {
		return "", err
	}
	var data struct {
		Address string `json:"address"`
	}
	if err := json.Unmarshal(file, &data); err != nil {
		return "", err
	}
	return data.Address, nil
}