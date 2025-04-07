package game

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"proof-generator/internal/domain"
	"time"
)

type PlayerFile struct {
	Address string      `json:"address"`
	Stake   string      `json:"stake"`
	Board   [][]bool    `json:"board"`
}

type InstantiateFile struct {
	Admin        string `json:"admin"`
	TokenAddress string `json:"token_address"`
	Ships        int    `json:"ships"`
}

type PlayerInstantiate struct {
	Address string `json:"address"`
	Stake   string `json:"stake"`
	Board   string `json:"board"`
}

type InstantiateMsg struct {
	Admin        string              `json:"admin"`
	TokenAddress string              `json:"token_address"`
	Ships        int                 `json:"ships"`
	Players      []PlayerInstantiate `json:"players"`
}

func LoadPlayerData(path string, generator *domain.Generator) (PlayerInstantiate, error) {
	file, err := os.ReadFile(path)
	if err != nil {
		return PlayerInstantiate{}, fmt.Errorf("failed to read file %s: %w", path, err)
	}

	var data PlayerFile
	if err := json.Unmarshal(file, &data); err != nil {
		return PlayerInstantiate{}, fmt.Errorf("invalid player file %s: %w", path, err)
	}
	generator = domain.NewGenerator(domain.NewBoard(data.Board))
	root := generator.GetRoot()

	return PlayerInstantiate{
		Address: data.Address,
		Stake:   data.Stake,
		Board:   root,
	}, nil
}

func InstantiateContract(msg InstantiateMsg) error {
	jsonBytes, err := json.Marshal(msg)
	if err != nil {
		return fmt.Errorf("failed to marshal instantiate msg: %w", err)
	}
	initMsg := string(jsonBytes)

	codeID := "7"

	cmd := exec.Command("wasmd", "tx", "wasm", "instantiate", codeID, initMsg,
		"--from=proof-generator",
		"--label=battleship",
		"--admin="+msg.Admin,
		"--chain-id=localnet",
		"--gas=auto", "--gas-adjustment=1.3",
		"--keyring-backend=test",
		"--broadcast-mode=sync",
		"-y",
	)

	fmt.Println("Instantiating contract...")

	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("contract instantiation failed: %v\nOutput: %s", err, string(output))
	}

	fmt.Println("Contract instantiated:\n", string(output))

	address, err := GetLastContractAddressByCode("7")
	if err != nil {
		fmt.Println("Could not fetch contract address automatically:", err)
	} else {
		_ = saveContractAddress(address)
		fmt.Println("Contract address:", address)
	}

	return nil
}

func GetLastContractAddressByCode(codeID string) (string, error) {
	cmd := exec.Command("wasmd", "query", "wasm", "list-contract-by-code", codeID, "--output=json")

	output, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("failed to query contracts by code: %w\nOutput: %s", err, string(output))
	}

	var result struct {
		Contracts []string `json:"contracts"`
	}
	if err := json.Unmarshal(output, &result); err != nil {
		return "", fmt.Errorf("failed to parse contract list: %w", err)
	}

	if len(result.Contracts) == 0 {
		return "", fmt.Errorf("no contracts found for code ID %s", codeID)
	}

	return result.Contracts[len(result.Contracts)-1], nil
}

func saveContractAddress(addr string) error {
	type ContractInfo struct {
		Address string `json:"address"`
	}
	file, err := os.Create("contract.json")
	if err != nil {
		return err
	}
	defer file.Close()
	return json.NewEncoder(file).Encode(ContractInfo{Address: addr})
}

func ApproveCW20(contractAddr, tokenAddr, player string, amount string) error {
	msg := map[string]interface{}{
		"increase_allowance": map[string]interface{}{
			"spender": contractAddr,
			"amount":  amount,
		},
	}
	msgBytes, _ := json.Marshal(msg)

	cmd := exec.Command("wasmd", "tx", "wasm", "execute", tokenAddr, string(msgBytes),
		"--from="+player,
		"--chain-id=localnet",
		"--keyring-backend=test",
		"--gas=auto", "--gas-adjustment=1.3",
		"--broadcast-mode=sync",
		"-y",
	)

	fmt.Printf("Approving CW20 allowance for %s...\n", player)

	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("approval failed for %s: %v\nOutput: %s", player, err, string(output))
	}

	fmt.Println("Approved:", player)
	return nil
}

func StartGame(contractAddr string, from string) error {
	msg := map[string]interface{}{
		"start_game": map[string]interface{}{},
	}
	msgBytes, _ := json.Marshal(msg)

	cmd := exec.Command("wasmd", "tx", "wasm", "execute", contractAddr, string(msgBytes),
		"--from="+from,
		"--chain-id=localnet",
		"--keyring-backend=test",
		"--gas=auto", "--gas-adjustment=1.3",
		"--broadcast-mode=sync",
		"-y",
	)

	fmt.Printf("Starting game from %s...\n", from)

	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("start_game failed from %s: %v\nOutput: %s", from, err, string(output))
	}

	fmt.Println("Game started by:", from)
	return nil
}

func InitGame(player1Generator *domain.Generator, player2Generator *domain.Generator) {
	fmt.Println("Initializing game...")

	instantiateData, err := os.ReadFile("game-config.json")
	if err != nil {
		fmt.Println("Failed to read game-config.json:", err)
		return
	}
	var meta InstantiateFile
	if err := json.Unmarshal(instantiateData, &meta); err != nil {
		fmt.Println("Invalid game-config.json:", err)
		return
	}

	var players []PlayerInstantiate
	player1, err := LoadPlayerData("player1.json", player1Generator)
	if err != nil {
		fmt.Println(err)
		return
	}
	players = append(players, player1)

	player2, err := LoadPlayerData("player2.json", player2Generator)
	if err != nil {
		fmt.Println(err)
		return
	}
	players = append(players, player2)

	msg := InstantiateMsg{
		Admin:        meta.Admin,
		TokenAddress: meta.TokenAddress,
		Ships:        meta.Ships,
		Players:      players,
	}

	err = InstantiateContract(msg)
	if err != nil {
		fmt.Println("Error instantiating:", err)
		return
	}

	contractAddr, err := GetLastContractAddressByCode("7")
	if err != nil {
		fmt.Println("Could not fetch contract address automatically:", err)
		return
	}

	for _, p := range msg.Players {
		err := ApproveCW20(contractAddr, msg.TokenAddress, p.Address, p.Stake)
		if err != nil {
			fmt.Println("Approval failed for", p.Address, ":", err)
			return
		}
	}

	time.Sleep(5 * time.Second)

	err = StartGame(contractAddr, msg.Players[0].Address)
	if err != nil {
		fmt.Println("Could not start game from", msg.Players[0].Address, ":", err)
		return
	}

	fmt.Println("Game successfully instantiated!")
}