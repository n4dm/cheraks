
package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"
)

const (
	difficulty = 4
	chainFile  = "cheraks-chain.json"
)

type Block struct {
	Index        int    `json:"index"`
	Timestamp    int64  `json:"timestamp"`
	Data         string `json:"data"`
	PreviousHash string `json:"previous_hash"`
	Nonce        uint64 `json:"nonce"`
	Hash         string `json:"hash"`
}

func calculateHash(b Block) string {
	record := strconv.Itoa(b.Index) +
		strconv.FormatInt(b.Timestamp, 10) +
		b.Data +
		b.PreviousHash +
		strconv.FormatUint(b.Nonce, 10)

	sum := sha256.Sum256([]byte(record))
	return hex.EncodeToString(sum[:])
}

func mineBlock(b Block) Block {
	target := strings.Repeat("0", difficulty)

	for {
		b.Hash = calculateHash(b)
		if strings.HasPrefix(b.Hash, target) {
			return b
		}
		b.Nonce++
	}
}

func isValidBlock(previous, current Block) bool {
	return current.Index == previous.Index+1 &&
		current.PreviousHash == previous.Hash &&
		current.Hash == calculateHash(current) &&
		strings.HasPrefix(current.Hash, strings.Repeat("0", difficulty))
}

func validChain(chain []Block) bool {
	if len(chain) == 0 {
		return false
	}

	genesis := chain[0]
	if genesis.Index != 0 ||
		genesis.PreviousHash != "0" ||
		genesis.Hash != calculateHash(genesis) {
		return false
	}

	for i := 1; i < len(chain); i++ {
		if !isValidBlock(chain[i-1], chain[i]) {
			return false
		}
	}
	return true
}

func createGenesis() Block {
	b := Block{
		Index:        0,
		Timestamp:    time.Now().Unix(),
		Data:         "Cheraks genesis block",
		PreviousHash: "0",
	}
	b.Hash = calculateHash(b)
	return b
}

func loadChain() ([]Block, error) {
	data, err := os.ReadFile(chainFile)
	if err != nil {
		return nil, err
	}

	var chain []Block
	if err := json.Unmarshal(data, &chain); err != nil {
		return nil, err
	}

	if !validChain(chain) {
		return nil, fmt.Errorf("saved blockchain failed validation")
	}
	return chain, nil
}

func saveChain(chain []Block) error {
	data, err := json.MarshalIndent(chain, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(chainFile, data, 0600)
}

func main() {
	chain, err := loadChain()

	if err != nil {
		if !os.IsNotExist(err) {
			fmt.Printf("Cannot load saved chain: %v\n", err)
			fmt.Println("Preserving the file; no overwrite performed.")
			return
		}

		chain = []Block{createGenesis()}
		if err := saveChain(chain); err != nil {
			fmt.Printf("Cannot save genesis block: %v\n", err)
			return
		}
		fmt.Println("Created new Cheraks chain.")
	} else {
		fmt.Println("Loaded saved Cheraks chain.")
	}

	next := Block{
		Index:        len(chain),
		Timestamp:    time.Now().Unix(),
		Data:         fmt.Sprintf("Cheraks local test block %d", len(chain)),
		PreviousHash: chain[len(chain)-1].Hash,
	}

	fmt.Println("Mining next block...")
	next = mineBlock(next)
	chain = append(chain, next)

	if !validChain(chain) {
		fmt.Println("Chain validation failed.")
		return
	}

	if err := saveChain(chain); err != nil {
		fmt.Printf("Cannot save chain: %v\n", err)
		return
	}

	fmt.Printf("Chain valid: %t\n", validChain(chain))
	fmt.Printf("Blocks saved: %d\n", len(chain))
	fmt.Printf("Latest block hash: %s\n", chain[len(chain)-1].Hash)
	fmt.Printf("Saved to: %s\n", chainFile)
}
