# Battleship Game on CosmWasm

This is a decentralized implementation of the classic Battleship game using CosmWasm smart contracts and a Go-based proof generator for creating board Merkle proofs.

## The project contains 3 main components:

1. Game contract
2. CW20 token contract
3. Proof generator

### Game contract
 
 The game smart contract has the basic functionality that any battleship game has - playing moves. In addition, it allows players to stake their cw20 fungible tokens. The winner of the game is rewarded with the opponents tokens and is also minted a small amount of tokens as a bonus.

 ### CW20 contract

 The CW20 contract represents the tokens that players use for staking. It is a cw20-base implementation, with a slight adjustment to allow only an admin to change the token minter.

 ### Proof generator

 This is an off-chain component implemented in Go. The main feature of this component is to store player boards and generate Merkle proofs for player moves, because, for security reasons, the game contract does not store the whole board, only its Merkle root. It also acts as a console-based interface, through which players can instantiate and interact with the game contract itself.

 ## How to run

 To run this game on your machine you will need Rust, Go and wasmd installed. 

 **1. Initialize wasmd**

 From the root of the directory run:

```bash
./init-localnet.sh
```

This will **REMOVE** any previous wasmd configuration you might have had, so caution is advised.

**2. Start wasmd**

```bash
wasmd start
```

**3. Deploy the contracts**

From battleship-game directory run:

```bash
./deploy.sh
```

**4. Instantiate CW20**

From battleship-game directory run:

```bash
./init-cw20.sh
```

**5. Configure the game**

- After instantiating cw20, its address will be stored in battleship/cw20_address.json, copy it into proof-generator/game-config.json, field token_address
- Next query the wasmd node:

    ```bash
    wasmd keys show proof-generator -a --keyring-backend=test
    ```
    copy the output into the admin field of the same file

- For ships, put the desired number, just be careful to match the board
- Next query both player addresses:
    ```bash
    wasmd keys show player1 -a --keyring-backend=test
    ```
    ```bash
    wasmd keys show player2 -a --keyring-backend=test
    ```
    and copy them in their corresponding json files (proof-generator/player1.json, proof-generator/player1.json)
- In player files, decide how many tokens you want to stake, and your board layout
- From proof-generator directory run:
    ```bash
    go run ./cmd/main.go
    ```
- Select option 1 from the menu to initialize the game
- Select 2 to play your moves


