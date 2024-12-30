#!/bin/bash

# Load environment variables from .env file
if [ -f .env ]; then
    export $(cat .env | grep -v '#' | awk '/=/ {print $1}')
else
    echo "Error: .env file not found"
    exit 1
fi

# Check if required environment variables are set
if [ -z "$IP_ASSET_REGISTRY_ADDRESS" ] || [ -z "$REGISTRATION_WORKFLOWS_ADDRESS" ] || [ -z "$PRIVATE_KEY" ] || [ -z "$RPC_URL" ] || [ -z "$BLOCKSCOUT_URL" ]; then
    echo "Error: Required environment variables are not set"
    echo "Please ensure your .env file contains:"
    echo "IP_ASSET_REGISTRY_ADDRESS"
    echo "REGISTRATION_WORKFLOWS_ADDRESS"
    echo "PRIVATE_KEY"
    echo "RPC_URL"
    echo "BLOCKSCOUT_URL"
    exit 1
fi

# Build the project
echo "Building project..."
forge build

# Run the deployment script
echo "Deploying IPARegistrar..."
forge script script/IPARegistrar.s.sol:IPARegistrarScript \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    --verify \
    --verifier-url $BLOCKSCOUT_URL/api/ \
    --verifier blockscout \
    -vvv

echo "Deployment completed!"
