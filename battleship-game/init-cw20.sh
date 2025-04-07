#!/bin/bash

set -e

KEYRING=test
CHAIN_ID=localnet
FROM=proof-generator
CODE_ID=$(jq -r '.cw20_code_id' code_ids.json)
INIT_FILE=init-cw20.json

echo "Instantiating CW20 contract..."

TX=$(wasmd tx wasm instantiate $CODE_ID "$(cat $INIT_FILE)" \
  --from $FROM \
  --label "cw20-stake" \
  --admin $(wasmd keys show $FROM -a --keyring-backend $KEYRING) \
  --keyring-backend $KEYRING \
  --chain-id $CHAIN_ID \
  --gas auto --gas-adjustment 1.3 \
  --broadcast-mode sync \
  --yes \
  --output json)

TX_HASH=$(echo $TX | jq -r '.txhash')

wait_for_tx() {
  local TX_HASH=$1
  local MAX_TRIES=10
  local COUNT=0

  while true; do
    RESULT=$(wasmd query tx $TX_HASH --output json 2>/dev/null || true)
    if echo "$RESULT" | jq -e '.logs' > /dev/null; then
      echo "$RESULT"
      return 0
    fi

    COUNT=$((COUNT + 1))
    if [ $COUNT -ge $MAX_TRIES ]; then
      echo "Transaction $TX_HASH not found after $MAX_TRIES attempts."
      exit 1
    fi

    sleep 2
  done
}

QUERY=$(wait_for_tx $TX_HASH)
ADDR=$(echo $QUERY | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')

echo "CW20 contract instantiated at: $ADDR"
echo "{ \"cw20_address\": \"$ADDR\" }" > cw20_address.json
