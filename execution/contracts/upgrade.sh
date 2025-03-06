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
    "PRIVATE_KEY"
    "RPC_URL"
    "BLOCKSCOUT_URL"
    "PROXY_ADDRESS"
    "IP_ASSET_REGISTRY_ADDRESS"
    "REGISTRATION_WORKFLOWS_ADDRESS"
    "PILICENSE_TEMPLATE_ADDRESS"
    "ROYALTY_POLICY_LAP_ADDRESS"
    "LICENSING_MODULE_ADDRESS"
    "BATCHER_ADDRESS"
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
echo "🚀 Building project..."
forge clean && forge build || {
    echo "❌ Build failed"
    exit 1
}

# Run the upgrade script
echo "🔄 Upgrading ForgeRegistry..."
forge script script/ForgeRegistry.s.sol:UpgradeForgeRegistry \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --broadcast \
    --verify \
    --verifier-url $BLOCKSCOUT_URL/api/ \
    --verifier blockscout \
    -vvv || {
    echo "❌ Upgrade failed"
    exit 1
}

echo "✅ Upgrade completed successfully!"

# Save upgrade timestamp and information
TIMESTAMP=$(date '+%Y-%m-%d_%H-%M-%S')
UPGRADE_LOG="deployments/upgrade_$TIMESTAMP.log"

# Create deployments directory if it doesn't exist
mkdir -p deployments

# Save upgrade information
{
    echo "Upgrade Timestamp: $TIMESTAMP"
    echo "Network RPC: $RPC_URL"
    echo "Blockscout URL: $BLOCKSCOUT_URL"
    echo "Proxy Address: $PROXY_ADDRESS"
    echo "IP Asset Registry Address: $IP_ASSET_REGISTRY_ADDRESS"
    echo "Registration Workflows Address: $REGISTRATION_WORKFLOWS_ADDRESS"
    echo "PI License Template Address: $PILICENSE_TEMPLATE_ADDRESS"
    echo "Royalty Policy LAP Address: $ROYALTY_POLICY_LAP_ADDRESS"
    echo "Licensing Module Address: $LICENSING_MODULE_ADDRESS"
    echo "Batcher Address: $BATCHER_ADDRESS"
} > "$UPGRADE_LOG"

echo "📋 Upgrade information saved to $UPGRADE_LOG"

# Verify the upgrade
echo "🔍 Verifying upgrade..."
sleep 10  # Wait for blockchain to process the upgrade

# You might want to add additional verification steps here
# For example, calling a version function if you have one

echo "🎉 Upgrade process completed!"
