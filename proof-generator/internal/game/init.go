package game

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"proof-generator/internal/domain"
	"strings"
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
	*generator = *domain.NewGenerator(domain.NewBoard(data.Board))
	root := generator.GetRoot()

	return PlayerInstantiate{
		Address: data.Address,
		Stake:   data.Stake,
		Board:   root,
	}, nil
}

func InstantiateCW20(codeID string, minter string, initialBalances []map[string]string) (string, error) {
	type CW20InitMsg struct {
		Name            string `json:"name"`
		Symbol          string `json:"symbol"`
		Decimals        int    `json:"decimals"`
		InitialBalances []struct {
			Address string `json:"address"`
			Amount  string `json:"amount"`
		} `json:"initial_balances"`
		Mint struct {
			Minter string  `json:"minter"`
			Cap    *string `json:"cap"`
		} `json:"mint"`
	}

	var msg CW20InitMsg
	msg.Name = "BattleToken"
	msg.Symbol = "BTK"
	msg.Decimals = 6

	for _, bal := range initialBalances {
		msg.InitialBalances = append(msg.InitialBalances, struct {
			Address string `json:"address"`
			Amount  string `json:"amount"`
		}{
			Address: bal["address"],
			Amount:  bal["amount"],
		})
	}

	msg.Mint.Minter = minter
	msg.Mint.Cap = nil

	msgBytes, err := json.Marshal(msg)
	if err != nil {
		return "", fmt.Errorf("marshal cw20 init msg: %w", err)
	}
	fmt.Println("Instantiating CW20 with msg:\n", string(msgBytes))

	cmd := exec.Command("wasmd", "tx", "wasm", "instantiate", codeID, string(msgBytes),
		"--from=proof-generator",
		"--label=cw20-init-minter",
		"--admin=proof-generator",
		"--chain-id=localnet",
		"--keyring-backend=test",
		"--gas=auto", "--gas-adjustment=1.3",
		"--broadcast-mode=sync",
		"-y",
	)

	fmt.Println("Instantiating CW20 with proof-generator as minter...")
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("instantiate cw20 failed: %v\nOutput: %s", err, string(out))
	}

	cmdQuery := exec.Command("wasmd", "query", "wasm", "list-contract-by-code", codeID, "--output=json")
	result, err := cmdQuery.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("query contracts failed: %w\nOutput: %s", err, string(result))
	}
	var parsed struct {
		Contracts []string `json:"contracts"`
	}
	if err := json.Unmarshal(result, &parsed); err != nil {
		return "", fmt.Errorf("parse contract list: %w", err)
	}
	return parsed.Contracts[len(parsed.Contracts)-1], nil
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

func SetCW20Minter(tokenAddr, minter, from string) error {
	msg := map[string]interface{}{
		"update_minter": map[string]interface{}{
			"new_minter": minter,
		},
	}
	msgBytes, _ := json.Marshal(msg)

	cmd := exec.Command("wasmd", "tx", "wasm", "execute", tokenAddr, string(msgBytes),
		"--from="+from,
		"--chain-id=localnet",
		"--keyring-backend=test",
		"--gas=auto", "--gas-adjustment=1.3",
		"--broadcast-mode=sync",
		"-y",
	)

	fmt.Println("Setting CW20 minter:", minter)

	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("setting minter failed: %v\nOutput: %s", err, string(output))
	}

	var txhash string
	lines := strings.Split(string(output), "\n")
	for _, line := range lines {
		if strings.HasPrefix(line, "txhash:") {
			txhash = strings.TrimSpace(strings.TrimPrefix(line, "txhash:"))
			break
		}
	}

	if err := WaitForTxCommit(txhash); err != nil {
		fmt.Println("TX not committed:", err)
		return err
	}

	fmt.Println("Minter set successfully.")
	return nil
}

func WaitForTxCommit(txHash string) error {
	for i := 0; i < 100; i++ {
		cmd := exec.Command("wasmd", "query", "tx", txHash, "--output=json")
		out, err := cmd.CombinedOutput()
		outputStr := string(out)

		if err == nil && strings.Contains(outputStr, `"code": 0`) {
			fmt.Println("Transaction committed:")
			fmt.Println(outputStr)
			return nil
		}

		// Handle "tx not found" error from CometBFT
		if strings.Contains(outputStr, "rpc error") || strings.Contains(outputStr, "tx not found") {
			fmt.Printf("TX not found yet (%d/100), waiting...\n", i+1)
			time.Sleep(2 * time.Second)
			continue
		}

		fmt.Printf("Unexpected error: %v\nOutput: %s\n", err, outputStr)
		time.Sleep(2 * time.Second)
	}

	return fmt.Errorf("Transaction %s not found after retries", txHash)
}

func ApproveCW20(contractAddr, tokenAddr, player string, amount string) error {
	fmt.Println("stake:", amount)
	fmt.Println("cw20:", tokenAddr)
	msg := map[string]interface{}{
		"increase_allowance": map[string]interface{}{
			"spender": contractAddr,
			"amount":  amount,
		},
	}
	msgBytes, _ := json.Marshal(msg)
	fmt.Println("msg:", string(msgBytes))

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
	fmt.Println("Allowance tx command output:")
	fmt.Println(string(output))

	var txhash string
	lines := strings.Split(string(output), "\n")
	for _, line := range lines {
	if strings.HasPrefix(line, "txhash:") {
			txhash = strings.TrimSpace(strings.TrimPrefix(line, "txhash:"))
			break
		}
	}

	if err := WaitForTxCommit(txhash); err != nil {
		fmt.Println("TX not committed:", err)
		return err
	}

	time.Sleep(6 * time.Second)

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

	time.Sleep(5 * time.Second)

	fmt.Println("Game started by:", from)
	return nil
}

func QueryBalance(contractAddr, address string) {
	cmd := exec.Command("wasmd", "query", "wasm", "contract-state", "smart", contractAddr,
		fmt.Sprintf(`{"balance":{"address":"%s"}}`, address),
		"--output=json")

	out, err := cmd.CombinedOutput()
	if err != nil {
		fmt.Printf("Failed to query balance for %s: %v\nOutput: %s\n", address, err, string(out))
		return
	}

	fmt.Printf("Balance for %s: %s\n", address, out)
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

	cw20Addr, err := InstantiateCW20("6", msg.Admin, []map[string]string{
		{"address": player1.Address, "amount": "100000000"},
		{"address": player2.Address, "amount": "100000000"},
	})
	if err != nil {
		fmt.Println("CW20 instantiation failed:", err)
		return
	}

	time.Sleep(5 * time.Second)

	QueryBalance(cw20Addr, player1.Address)
	QueryBalance(cw20Addr, player2.Address)

	msg.TokenAddress = cw20Addr

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

	time.Sleep(5 * time.Second)

	err = SetCW20Minter(msg.TokenAddress, contractAddr, msg.Admin)
	if err != nil {
		fmt.Println("Failed to set minter:", err)
		return
	}

	time.Sleep(10 * time.Second)

	err = ApproveCW20(contractAddr, msg.TokenAddress, "player1", player1.Stake)
	if err != nil {
		fmt.Println("Approval failed for", player1.Address, ":", err)
		return
	}

	time.Sleep(5 * time.Second)

	err = ApproveCW20(contractAddr, msg.TokenAddress, "player2", player2.Stake)
	if err != nil {
		fmt.Println("Approval failed for", player2.Address, ":", err)
		return
	}

	time.Sleep(10 * time.Second)

	fmt.Println("== Checking CW20 allowances ==")

	err = CheckAllowance(meta.TokenAddress, player1.Address, contractAddr)
	if err != nil {
		fmt.Println("Failed to check allowance for player1:", err)
	}

	err = CheckAllowance(meta.TokenAddress, player2.Address, contractAddr)
	if err != nil {
		fmt.Println("Failed to check allowance for player2:", err)
	}

	err = StartGame(contractAddr, msg.Players[0].Address)
	if err != nil {
		fmt.Println("Could not start game from", msg.Players[0].Address, ":", err)
		return
	}

	fmt.Println("Game successfully instantiated!")
}

func CheckAllowance(cw20Addr, ownerAddr, spenderAddr string) error {
	query := map[string]interface{}{
		"allowance": map[string]interface{}{
			"owner":   ownerAddr,
			"spender": spenderAddr,
		},
	}
	queryBytes, err := json.Marshal(query)
	if err != nil {
		return fmt.Errorf("failed to marshal allowance query: %w", err)
	}

	cmd := exec.Command("wasmd", "query", "wasm", "contract-state", "smart", cw20Addr, string(queryBytes), "--output=json")
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("failed to query allowance: %v\nOutput: %s", err, output)
	}

	fmt.Printf("Allowance for spender %s from owner %s:\n%s\n", spenderAddr, ownerAddr, string(output))
	return nil
}