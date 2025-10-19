#!/bin/bash
# Judge Prompts API Testing with curl

BASE_URL="http://127.0.0.1:8080/api/v1"

echo "=== Judge Prompts API Tests ==="
echo ""

# 1. Get all judge prompts
echo "1️⃣  GET all judge prompts:"
curl -s -X GET "${BASE_URL}/judge-prompts" | jq '.'
echo -e "\n"

# 2. Get the active judge prompt
echo "2️⃣  GET active judge prompt:"
curl -s -X GET "${BASE_URL}/judge-prompts/active" | jq '.'
echo -e "\n"

# 3. Get a specific judge prompt by version (version 1)
echo "3️⃣  GET judge prompt version 1:"
curl -s -X GET "${BASE_URL}/judge-prompts/1" | jq '.'
echo -e "\n"

# 4. Create a new judge prompt (not active)
echo "4️⃣  POST create new judge prompt (not active):"
curl -s -X POST "${BASE_URL}/judge-prompts" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Strict Evaluation",
    "template": "You are a strict evaluator.\n\nExpected: {{expected}}\nActual: {{actual}}\n\nAre they identical? Answer with Verdict: PASS or Verdict: FAIL",
    "description": "A more strict evaluation prompt that requires exact matches",
    "set_active": false
  }' | jq '.'
echo -e "\n"

# 5. Create a new judge prompt (and set as active)
echo "5️⃣  POST create new judge prompt (set as active):"
curl -s -X POST "${BASE_URL}/judge-prompts" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Lenient Evaluation",
    "template": "You are a lenient evaluator.\n\nCriteria: {{criteria}}\nExpected: {{expected}}\nActual: {{actual}}\n\nDo they convey the same meaning? Verdict: PASS or FAIL\nReasoning:",
    "description": "A more lenient evaluation that focuses on semantic meaning",
    "set_active": true
  }' | jq '.'
echo -e "\n"

# 6. Verify the active prompt changed
echo "6️⃣  GET active judge prompt (should be the new one):"
curl -s -X GET "${BASE_URL}/judge-prompts/active" | jq '.'
echo -e "\n"

# 7. Set a different prompt as active (version 2)
echo "7️⃣  PUT set prompt version 2 as active:"
curl -s -X PUT "${BASE_URL}/judge-prompts/active" \
  -H "Content-Type: application/json" \
  -d '{
    "version": 2
  }' | jq '.'
echo -e "\n"

# 8. Get all prompts again to see the active status changed
echo "8️⃣  GET all judge prompts (check is_active field):"
curl -s -X GET "${BASE_URL}/judge-prompts" | jq '.'
echo -e "\n"

# 9. Try to get a non-existent version (should return 404)
echo "9️⃣  GET non-existent version 999 (should fail):"
curl -s -X GET "${BASE_URL}/judge-prompts/999" | jq '.'
echo -e "\n"

# 10. Create a minimal prompt (no description, not active)
echo "🔟 POST create minimal judge prompt:"
curl -s -X POST "${BASE_URL}/judge-prompts" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Minimal Prompt",
    "template": "Compare:\nExpected: {{expected}}\nActual: {{actual}}\nVerdict:",
    "set_active": false
  }' | jq '.'
echo -e "\n"

echo "✅ All tests complete!"
