#!/bin/bash
# Server Mode Demo Script
# Tests the basic functionality of the server daemon

set -e

echo "=========================================="
echo "Serial CLI Server Mode Demo"
echo "=========================================="
echo ""

# Clean up any existing server
echo "1. Cleaning up any existing server..."
./target/debug/serial-cli server stop 2>/dev/null || true
sleep 1
echo ""

# Start the server
echo "2. Starting server daemon..."
./target/debug/serial-cli server start
echo ""
sleep 2

# Check server status
echo "3. Checking server status..."
./target/debug/serial-cli server status
echo ""
sleep 1

# Test port_list
echo "4. Testing port_list RPC call..."
./target/debug/serial-cli server call port_list '{}'
echo ""
sleep 1

# Test protocol_list
echo "5. Testing protocol_list RPC call..."
./target/debug/serial-cli server call protocol_list '{}'
echo ""
sleep 1

# Test connection_list (should be empty)
echo "6. Testing connection_list RPC call..."
./target/debug/serial-cli server call connection_list '{}'
echo ""
sleep 1

# Test server_stats
echo "7. Testing server_stats RPC call..."
./target/debug/serial-cli server call server_stats '{}'
echo ""
sleep 1

# Stop the server
echo "8. Stopping server daemon..."
./target/debug/serial-cli server stop
echo ""
sleep 1

# Verify stopped
echo "9. Verifying server stopped..."
./target/debug/serial-cli server status
echo ""

echo "=========================================="
echo "Demo completed successfully!"
echo "=========================================="
