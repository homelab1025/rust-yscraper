#!/bin/bash

echo "Starting server in background..."
cargo run --bin web_server > server.log 2>&1 &
SERVER_PID=$!

echo "Server PID: $SERVER_PID"
echo "Waiting for server to start..."
sleep 5

echo "Testing API endpoints..."

# Test 1: Validation error for zero days_limit
echo "Test 1: Validation error for zero days_limit"
RESPONSE=$(curl -X POST http://localhost:3000/scrape \
  -H "Content-Type: application/json" \
  -d '{"item_id": 12345, "days_limit": 0, "frequency_hours": 24}' \
  -s -w "\nHTTP_CODE:%{http_code}")

if echo "$RESPONSE" | grep -q "HTTP_CODE:400"; then
    echo "✅ Validation for zero days_limit works"
else
    echo "❌ Validation for zero days_limit failed"
fi

# Test 2: Validation error for zero frequency_hours  
echo "Test 2: Validation error for zero frequency_hours"
RESPONSE=$(curl -X POST http://localhost:3000/scrape \
  -H "Content-Type: application/json" \
  -d '{"item_id": 12345, "days_limit": 7, "frequency_hours": 0}' \
  -s -w "\nHTTP_CODE:%{http_code}")

if echo "$RESPONSE" | grep -q "HTTP_CODE:400"; then
    echo "✅ Validation for zero frequency_hours works"
else
    echo "❌ Validation for zero frequency_hours failed"
fi

# Test 3: Valid request with defaults
echo "Test 3: Valid request with defaults"
RESPONSE=$(curl -X POST http://localhost:3000/scrape \
  -H "Content-Type: application/json" \
  -d '{"item_id": 12345}' \
  -s -w "\nHTTP_CODE:%{http_code}")

if echo "$RESPONSE" | grep -q "HTTP_CODE:200"; then
    echo "✅ Valid request with defaults works"
else
    echo "❌ Valid request with defaults failed"
fi

# Test 4: Valid request with custom values
echo "Test 4: Valid request with custom values"
RESPONSE=$(curl -X POST http://localhost:3000/scrape \
  -H "Content-Type: application/json" \
  -d '{"item_id": 12345, "days_limit": 14, "frequency_hours": 12}' \
  -s -w "\nHTTP_CODE:%{http_code}")

if echo "$RESPONSE" | grep -q "HTTP_CODE:200"; then
    echo "✅ Valid request with custom values works"
else
    echo "❌ Valid request with custom values failed"
fi

# Test 5: Check existing scheduling metadata preserved
echo "Test 5: Verify scheduling metadata stored in database"
# We'll check database directly for this
docker exec -i yscraper-postgres psql -U postgres -d yscraper -c "SELECT id, frequency_hours, days_limit FROM urls WHERE id = 12345;"

echo "Cleaning up..."
kill $SERVER_PID
echo "Server stopped."