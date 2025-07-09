#!/bin/bash

set -e

CHAIN_ID="localnet"
KEYRING="test"
DENOM="stake"
STAKED="100000000${DENOM}"

# Remove previous config
echo "Removing old wasmd config..."
rm -rf ~/.wasmd

# Init node
echo "Initializing new wasmd node..."
wasmd init $CHAIN_ID --chain-id $CHAIN_ID

# Create accounts
echo "Creating accounts..."
wasmd keys add player1 --keyring-backend $KEYRING
wasmd keys add player2 --keyring-backend $KEYRING
wasmd keys add proof-generator --keyring-backend $KEYRING

# Add genesis accounts
echo "Funding accounts in genesis..."
wasmd genesis add-genesis-account $(wasmd keys show player1 -a --keyring-backend $KEYRING) $STAKED
wasmd genesis add-genesis-account $(wasmd keys show player2 -a --keyring-backend $KEYRING) $STAKED
wasmd genesis add-genesis-account $(wasmd keys show proof-generator -a --keyring-backend $KEYRING) $STAKED

# Create validator tx for proof-generator
echo "Creating genesis tx..."
wasmd genesis gentx proof-generator $STAKED --keyring-backend $KEYRING --chain-id $CHAIN_ID

# Combine genesis transactions
echo "Collecting gentxs..."
wasmd genesis collect-gentxs

# Validate genesis
wasmd genesis validate-genesis

echo "Localnet setup complete!"
echo "Run your node with: wasmd start"
