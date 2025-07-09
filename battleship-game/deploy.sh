#!/bin/bash

set -e

KEYRING=test
CHAIN_ID=localnet
FROM=proof-generator
OUT_FILE=../proof-generator/code_ids.json

ROOT_DIR=$(pwd)
ARTIFACTS_DIR="$ROOT_DIR/artifacts"
mkdir -p $ARTIFACTS_DIR

echo "Compiling contracts..."
cargo build --release --target wasm32-unknown-unknown

echo "Uploading contracts to chain..."

# Upload cw20-base
CW20_WASM="$ROOT_DIR/target/wasm32-unknown-unknown/release/cw20_base.wasm"
echo "Uploading cw20-base..."
TX=$(wasmd tx wasm store $CW20_WASM \
  --from $FROM --keyring-backend $KEYRING --chain-id $CHAIN_ID \
  --gas auto --gas-adjustment 1.3 \
  --broadcast-mode sync --yes --output json)

TX_HASH=$(echo $TX | jq -r '.txhash')
sleep 3
QUERY=$(wasmd query tx $TX_HASH --output json)
CW20_CODE_ID=$(echo $QUERY | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

echo "cw20-base CODE_ID: $CW20_CODE_ID"

# Upload game contract
GAME_WASM="$ROOT_DIR/target/wasm32-unknown-unknown/release/battleship_game.wasm"
echo "Uploading game..."
TX=$(wasmd tx wasm store $GAME_WASM \
  --from $FROM --keyring-backend $KEYRING --chain-id $CHAIN_ID \
  --gas auto --gas-adjustment 1.3 \
  --broadcast-mode sync --yes --output json)

TX_HASH=$(echo $TX | jq -r '.txhash')
sleep 5
QUERY=$(wasmd query tx $TX_HASH --output json)
GAME_CODE_ID=$(echo $QUERY | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

echo "game CODE_ID: $GAME_CODE_ID"

# Save code IDs to JSON
echo "Saving code IDs to $OUT_FILE..."
cat <<EOF > $OUT_FILE
{
  "cw20_code_id": "$CW20_CODE_ID",
  "game_code_id": "$GAME_CODE_ID"
}
EOF

echo "Contracts successfully deployed."
