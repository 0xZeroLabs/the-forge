-include .env

.PHONY: all test clean

all: clean build deploy

# Clean the repo
clean  :; forge clean

# Update Dependencies
forge-update:; forge update

build:; forge build

deploy :; forge script script/Deploy.s.sol:DeployScript  --fork-url sepolia --broadcast --verify -vvvv --priority-gas-price 1

