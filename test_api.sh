#!/bin/bash

echo "Testing validation with zero days_limit..."
curl -X POST http://localhost:3000/scrape \
  -H "Content-Type: application/json" \
  -d '{"item_id": 12345, "days_limit": 0, "frequency_hours": 24}' \
  -w "\nHTTP Status: %{http_code}\n" \
  -s

echo "Testing validation with zero frequency_hours..."
curl -X POST http://localhost:3000/scrape \
  -H "Content-Type: application/json" \
  -d '{"item_id": 12345, "days_limit": 7, "frequency_hours": 0}' \
  -w "\nHTTP Status: %{http_code}\n" \
  -s

echo "Testing valid request with defaults..."
curl -X POST http://localhost:3000/scrape \
  -H "Content-Type: application/json" \
  -d '{"item_id": 12345}' \
  -w "\nHTTP Status: %{http_code}\n" \
  -s