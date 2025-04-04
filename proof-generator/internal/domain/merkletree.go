package domain

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"strings"
	"slices"
)

type Node struct {
	parent *Node;
	left *Node
	right *Node
	data string;
}

type MerkleTree struct {
	data []string;
	Leaves []*Node;
	Root *Node;
}

type ProofStep struct {
	hash string;
	isLeft bool;
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
		node := &Node {
			parent: nil,
			data: hash,
		}
		merkleTree.Leaves = append(merkleTree.Leaves, node)
	}
	if len(merkleTree.Leaves) % 2 != 0 {
		merkleTree.Leaves = append(merkleTree.Leaves, merkleTree.Leaves[len(merkleTree.Leaves)-1])
	}
}

func (merkleTree *MerkleTree) generateTree() {
	merkleTree.Root = generateTree(merkleTree.Leaves)
}

func generateTree(level []*Node) *Node {
	if len(level) == 1 {
		return level[0];
	}

	if len(level) % 2 != 0 {
		level = append(level, level[len(level)-1])
	}

	var nextLevel []*Node

	for i := 0; i < len(level) - 1; i += 2 {
		combined := level[i].data + level[i+1].data
		hash := hash(combined)
		nextLevel = append(
			nextLevel, 
			&Node {
				parent: nil,
				data: hash,
			},
		)
		level[i].parent = nextLevel[len(nextLevel) - 1]
		level[i+1].parent = nextLevel[len(nextLevel) - 1]
		level[i].right = level[i+1]
		level[i+1].left = level[i]
	}

	return generateTree(nextLevel)
}

func hash(item string) string {
	sum := sha256.Sum256([]byte(item))
	return hex.EncodeToString(sum[:])
}

func (merkleTree *MerkleTree) String() string {
	if len(merkleTree.Leaves) == 0 {
		return "Merkle Tree is empty"
	}

	var levels [][]string
	currentLevel := slices.Clone(merkleTree.Leaves)

	for {
		var hashes []string
		var nextLevel []*Node

		for _, node := range currentLevel {
			hashes = append(hashes, node.data)
			if node.parent != nil {
				nextLevel = appendIfMissing(nextLevel, node.parent)
			}
		}

		levels = append(levels, hashes)

		if len(nextLevel) == 0 {
			break
		}

		currentLevel = nextLevel
	}

	var sb strings.Builder
	for i := len(levels) - 1; i >= 0; i-- {
		sb.WriteString(fmt.Sprintf("Level %d:\n", len(levels)-1-i))
		for _, hash := range levels[i] {
			sb.WriteString(fmt.Sprintf("  %s\n", hash))
		}
	}

	return sb.String()
}

func appendIfMissing(nodes []*Node, node *Node) []*Node {
	if slices.Contains(nodes, node) {
			return nodes
		}
	return append(nodes, node)
}

func (merkleTree *MerkleTree) GenerateProof(dataIndex int) (string, []ProofStep) {
	var proof []ProofStep

	dataToProve := merkleTree.data[dataIndex]
	leaf := merkleTree.Leaves[dataIndex]

	node := leaf
	for node != nil {
		
		if node.left != nil {
			proof = append(
				proof, 
				ProofStep {
					hash: node.left.data,
					isLeft: true,
				},
			)
		}

		if node.right != nil {
			proof = append(
				proof, 
				ProofStep {
					hash: node.right.data,
					isLeft: false,
				},
			)
		}

		node = node.parent
	}

	return dataToProve, proof
}

func (merkleTree *MerkleTree) VerifyProof(data string, proof []ProofStep) bool {
	currentHash := hash(data)

	for _, step := range proof {
		if step.isLeft {
			currentHash = hash(step.hash + currentHash)
			continue
		}
		currentHash = hash(currentHash + step.hash)
	}

	return currentHash == merkleTree.Root.data
}


