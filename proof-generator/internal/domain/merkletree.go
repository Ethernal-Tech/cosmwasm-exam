package domain

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"strings"
)

type MerkleTree struct {
	data []string;
	Nodes []string;
}

func NewMerkleTree(data []string) *MerkleTree {
	merkleTree := &MerkleTree {
		data: data,
	}

	merkleTree.generateLeaves()
	merkleTree.generateTree()

	return merkleTree
}

func (merkleTree *MerkleTree) generateLeaves() {
	for _, item := range merkleTree.data {
		hash := hash(item)
		merkleTree.Nodes = append(merkleTree.Nodes, hash)
	}
	if len(merkleTree.Nodes) % 2 != 0 {
		merkleTree.Nodes = append(merkleTree.Nodes, hash("0"))
	}
}

func (merkleTree *MerkleTree) generateTree() {
	currentLevel := merkleTree.Nodes
	for len(currentLevel) > 1 {
		var nextLevel []string

		for i := 0; i < len(currentLevel) - 1; i += 2 {
			combined := currentLevel[i] + currentLevel[i+1]
			hash := hash(combined)
			nextLevel = append(nextLevel, hash)
		}

		merkleTree.Nodes = append(merkleTree.Nodes, nextLevel...)
		currentLevel = nextLevel
	}
}

func hash(item string) string {
	sum := sha256.Sum256([]byte(item))
	return hex.EncodeToString(sum[:])
}

func (merkleTree *MerkleTree) String() string {
	if len(merkleTree.Nodes) == 0 {
		return "Empty Merkle Tree"
	}

	var result string
	levelSize := len(merkleTree.data)
	start := 0

	for levelSize > 0 {
		end := start + levelSize
		if end > len(merkleTree.Nodes) {
			end = len(merkleTree.Nodes)
		}
		
		level := merkleTree.Nodes[start:end]
		result = fmt.Sprintf("%s\n%s", strings.Join(level, " "), result)

		start = end
		levelSize = levelSize / 2
	}

	return result
}


