#!/bin/bash

echo "Testing Cap'n Web protocol formats"
echo "==================================="

echo -e "\nTest 1: Empty JSON array (what server expects)"
curl -X POST http://localhost:8080/rpc/batch \
  -H "Content-Type: application/json" \
  -d '[]' \
  -v 2>&1 | grep -E "< HTTP|< |> " | head -20

echo -e "\nTest 2: Newline-delimited format (what TypeScript client sends)"
echo 'Sending: ["push",["pipeline",0,["add"],[5,3]]]\n["pull",1]'
curl -X POST http://localhost:8080/rpc/batch \
  -H "Content-Type: text/plain" \
  -d $'["push",["pipeline",0,["add"],[5,3]]]\n["pull",1]' \
  -v 2>&1 | grep -E "< HTTP|< |> " | head -20

echo -e "\nTest 3: Standard JSON message format"
curl -X POST http://localhost:8080/rpc/batch \
  -H "Content-Type: application/json" \
  -d '[{"call":{"id":1,"target":{"cap":{"id":1}},"member":"add","args":[5,3]}}]' \
  -v 2>&1 | grep -E "< HTTP|< |> " | head -20