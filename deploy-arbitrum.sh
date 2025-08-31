#!/bin/bash

# Bitcoin Price Oracle Deployment Script for Arbitrum Testnet
# This script deploys the Bitcoin price feed oracle to Arbitrum Sepolia testnet

set -e

ORACLE_NAME="bitcoin-price-oracle"
NETWORK="arbitrum-sepolia"
CONFIG_FILE="arbitrum-testnet.toml"

echo "🚀 Deploying Bitcoin Price Oracle to Arbitrum Testnet..."

# Build the oracle WASM module
echo "📦 Building WASM module..."
cargo build --target wasm32-wasi --release --example bitcoin_price_feed

# Check if deployment configuration exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Configuration file $CONFIG_FILE not found!"
    exit 1
fi

echo "⚙️  Using configuration: $CONFIG_FILE"

# Deploy to Blocksense network (placeholder - actual deployment command would depend on Blocksense CLI)
echo "🌐 Deploying oracle to Blocksense network..."
echo "Oracle: $ORACLE_NAME"
echo "Target: $NETWORK"
echo "WASM: target/wasm32-wasi/release/examples/bitcoin_price_feed.wasm"

# Note: Replace with actual Blocksense deployment command when available
# blocksense deploy --oracle $ORACLE_NAME --config $CONFIG_FILE --network $NETWORK

echo "✅ Deployment configuration ready!"
echo "📝 Next steps:"
echo "   1. Install Blocksense CLI tool"
echo "   2. Configure Arbitrum testnet credentials"
echo "   3. Run: blocksense deploy --oracle $ORACLE_NAME --config $CONFIG_FILE --network $NETWORK"
echo ""
echo "🔧 Oracle Details:"
echo "   - Feeds: BTC/USD price, 24h volume, market status"
echo "   - Sources: 5 mock exchanges with median aggregation"
echo "   - Update threshold: 0.5% price change"
echo "   - Heartbeat: 5 minutes"