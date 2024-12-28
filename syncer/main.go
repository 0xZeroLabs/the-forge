package main

import (
	"fmt"
	"os"
	"os/exec"
	"strings"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/joho/godotenv"
)

func main() {
	// Load environment variables from .env file
	if err := godotenv.Load(); err != nil {
		fmt.Printf("Warning: Error loading .env file: %v\n", err)
	}

	// Get and validate deployer private key
	privateKeyDeployer := os.Getenv("PRIVATE_KEY_DEPLOYER")
	if privateKeyDeployer == "" {
		fmt.Println("PRIVATE_KEY_DEPLOYER is not set.")
		os.Exit(1)
	}

	// Get and validate syncer private key
	privateKeySyncer := os.Getenv("PRIVATE_KEY_SYNCER")
	if privateKeySyncer == "" {
		fmt.Println("PRIVATE_KEY_SYNCER is not set.")
		os.Exit(1)
	}

	// Add '0x' prefix if not present
	if !strings.HasPrefix(privateKeySyncer, "0x") {
		privateKeySyncer = "0x" + privateKeySyncer
	}
	if !strings.HasPrefix(privateKeyDeployer, "0x") {
		privateKeyDeployer = "0x" + privateKeyDeployer
	}

	// Validate deployer key and get address
	_, err := crypto.HexToECDSA(strings.TrimPrefix(privateKeyDeployer, "0x"))
	if err != nil {
		fmt.Println("Invalid PRIVATE_KEY_DEPLOYER")
		os.Exit(1)
	}

	// Validate syncer key and get address
	syncerPrivKey, err := crypto.HexToECDSA(strings.TrimPrefix(privateKeySyncer, "0x"))
	if err != nil {
		fmt.Println("Invalid PRIVATE_KEY_SYNCER")
		os.Exit(1)
	}
	syncerAddress := crypto.PubkeyToAddress(syncerPrivKey.PublicKey)

	// Set deployer private key in environment
	os.Setenv("PRIVATE_KEY", privateKeyDeployer)

	// Execute CLI command
	cmd := exec.Command("othentic-cli", "network", "set-syncer", "--syncer-address", syncerAddress.Hex())
	output, err := cmd.CombinedOutput()
	if err != nil {
		fmt.Printf("Error executing command: %v\n", err)
		os.Exit(1)
	}

	fmt.Println(string(output))
}
