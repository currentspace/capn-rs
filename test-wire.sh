#!/bin/bash

echo "Testing Cap'n Web Wire Protocol with Rust Server"
echo "================================================="
echo

# Test 1: Simple pipeline call for add method
echo "Test 1: Pipeline call - add(5, 3)"
echo "Request:"
echo '["push",["pipeline",0,["add"],[5,3]]]'
echo '["pull",1]'
echo
echo "Response:"
curl -s -X POST http://localhost:8080/rpc/batch \
  -H "Content-Type: text/plain" \
  -d '["push",["pipeline",0,["add"],[5,3]]]
["pull",1]'
echo
echo

# Test 2: Health check
echo "Test 2: Health check"
curl -s http://localhost:8080/health | jq .
echo