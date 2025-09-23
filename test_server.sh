#!/bin/bash

# Test script for Cap'n Web server

echo "Testing Cap'n Web Server HTTP Batch Endpoint"
echo "============================================="
echo ""

# Check if server is running
SERVER_URL="http://127.0.0.1:8080/rpc/batch"

# Test 1: Basic calculation
echo "Test 1: Basic addition (5 + 3)"
curl -s -X POST $SERVER_URL \
  -H "Content-Type: application/json" \
  -d '[{"type":"call","id":1,"target":1,"member":"add","args":[5,3]}]' | jq .

echo ""
echo "Test 2: Multiple operations in batch"
curl -s -X POST $SERVER_URL \
  -H "Content-Type: application/json" \
  -d '[
    {"type":"call","id":1,"target":1,"member":"add","args":[5,3]},
    {"type":"call","id":2,"target":1,"member":"multiply","args":[4,7]},
    {"type":"call","id":3,"target":1,"member":"divide","args":[10,2]}
  ]' | jq .

echo ""
echo "Test 3: Echo service"
curl -s -X POST $SERVER_URL \
  -H "Content-Type: application/json" \
  -d '[
    {"type":"call","id":1,"target":2,"member":"echo","args":["hello","world"]},
    {"type":"call","id":2,"target":2,"member":"reverse","args":["rust"]}
  ]' | jq .

echo ""
echo "Test 4: Error handling (division by zero)"
curl -s -X POST $SERVER_URL \
  -H "Content-Type: application/json" \
  -d '[{"type":"call","id":1,"target":1,"member":"divide","args":[10,0]}]' | jq .

echo ""
echo "Test 5: Unknown capability"
curl -s -X POST $SERVER_URL \
  -H "Content-Type: application/json" \
  -d '[{"type":"call","id":1,"target":999,"member":"test","args":[]}]' | jq .

echo ""
echo "Tests completed!"