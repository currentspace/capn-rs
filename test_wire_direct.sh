#!/bin/bash

echo "Testing Cap'n Web Wire Protocol Format"
echo "======================================"

# Test the exact format the TypeScript client sends
echo ""
echo "Test 1: TypeScript client format (newline-delimited arrays)"
echo "Request body:"
echo '["push",["pipeline",0,["add"],[5,3]]]'
echo '["pull",1]'

curl -X POST http://localhost:8080/rpc/batch \
  -H "Content-Type: text/plain" \
  -d $'["push",["pipeline",0,["add"],[5,3]]]\n["pull",1]' \
  -v

echo -e "\n\nTest 2: Single message format"
echo "Request body:"
echo '["push",["pipeline",0,["add"],[5,3]]]'

curl -X POST http://localhost:8080/rpc/batch \
  -H "Content-Type: text/plain" \
  -d '["push",["pipeline",0,["add"],[5,3]]]' \
  -v

echo -e "\n\nTest 3: Health check"
curl -X GET http://localhost:8080/health