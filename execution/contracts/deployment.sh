#!/bin/bash

# Load environment variables from .env file
if [ -f .env ]; then
    export $(cat .env | grep -v '#' | awk '/=/ {print $1}')
else
    echo "❌ Error: .env file not found"
    exit 1
fi

# Required environment variables
REQUIRED_VARS=(
    "IP_ASSET_REGISTRY_ADDRESS"
    "REGISTRATION_WORKFLOWS_ADDRESS"
    "PRIVATE_KEY"
    "RPC_URL"
    "BLOCKSCOUT_URL"
)

# Check each required variable
missing_vars=()
for var in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!var}" ]; then
        missing_vars+=($var)
    fi
done

# If any variables are missing, print error and exit
if [ ${#missing_vars[@]} -ne 0 ]; then
    echo "❌ Error: Missing required environment variables:"
    printf '%s\n' "${missing_vars[@]}"
    echo "Please ensure these variables are set in your .env file"
    exit 1
fi

# Clean and build the project
echo "🔨 Building project..."
forge clean && forge build || {
    echo "❌ Build failed"
    exit 1
}

# Run the deployment script
echo "🚀 Deploying ForgeRegistry..."
forge script script/ForgeRegistry.s.sol:DeployForgeRegistry \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    --verify \
    --verifier-url $BLOCKSCOUT_URL/api/ \
    --verifier blockscout \
    -vvv || {
    echo "❌ Deployment failed"
    exit 1
}

# Save deployment information
TIMESTAMP=$(date '+%Y-%m-%d_%H-%M-%S')
DEPLOYMENT_DIR="deployments"
DEPLOYMENT_LOG="$DEPLOYMENT_DIR/deployment_$TIMESTAMP.log"

mkdir -p $DEPLOYMENT_DIR

{
    echo "Deployment Information"
    echo "====================="
    echo "Timestamp: $TIMESTAMP"
    echo "Network RPC: $RPC_URL"
    echo "Blockscout URL: $BLOCKSCOUT_URL"
    echo "IP Asset Registry: $IP_ASSET_REGISTRY_ADDRESS"
    echo "Registration Workflows: $REGISTRATION_WORKFLOWS_ADDRESS"
} > "$DEPLOYMENT_LOG"

echo "✅ Deployment completed successfully!"
echo "📝 Deployment log saved to: $DEPLOYMENT_LOG"
