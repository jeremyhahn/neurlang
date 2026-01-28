# Model Generation Test Specification

## Objective
Validate that the trained model can generate a production-grade multi-service REST API system with:
- User Service (authentication, CRUD)
- Inventory Service (stock management, CRUD)
- 12+ endpoints across services
- SQLite persistence layer
- Proper error handling

## Test Prompts

### User Service (Port 8080)

1. **GET /users** - List all users
   ```
   Prompt: "create a REST API endpoint GET /users that returns all users from SQLite database as JSON array"
   Expected: Server init, accept loop, GET check, sqlite query, JSON response, send 200
   ```

2. **GET /users/{id}** - Get user by ID
   ```
   Prompt: "implement GET /users/{id} endpoint that extracts ID from path and returns single user JSON"
   Expected: Path parsing, sqlite query with bind, JSON response, 404 handling
   ```

3. **POST /users** - Create user
   ```
   Prompt: "create POST /users endpoint that parses JSON body, generates UUID, inserts into database"
   Expected: JSON parse, UUID generation, sqlite insert, return 201 with created user
   ```

4. **PUT /users/{id}** - Update user
   ```
   Prompt: "implement PUT /users/{id} endpoint to update user fields from JSON body"
   Expected: Path parsing, JSON parse, sqlite update, return updated user
   ```

5. **DELETE /users/{id}** - Delete user
   ```
   Prompt: "create DELETE /users/{id} endpoint to remove user from database"
   Expected: Path parsing, sqlite delete, return 204 No Content
   ```

6. **POST /auth/login** - User authentication
   ```
   Prompt: "implement POST /auth/login endpoint that validates username/password against database"
   Expected: JSON parse, sqlite query, password compare, return token or 401
   ```

7. **POST /auth/register** - User registration
   ```
   Prompt: "create POST /auth/register endpoint for new user signup with password hashing"
   Expected: JSON parse, validation, sqlite insert, return 201
   ```

### Inventory Service (Port 8081)

8. **GET /items** - List all items
   ```
   Prompt: "create GET /items endpoint for inventory service on port 8081"
   Expected: Server on 8081, sqlite query items table, JSON array response
   ```

9. **GET /items/{id}** - Get item by ID
   ```
   Prompt: "implement GET /items/{id} endpoint with SKU validation"
   Expected: Path parsing, sqlite query, JSON response, 404 handling
   ```

10. **POST /items** - Create item
    ```
    Prompt: "create POST /items endpoint to add new inventory item with name, price, quantity"
    Expected: JSON parse, UUID generation, sqlite insert, return 201
    ```

11. **PUT /items/{id}/stock** - Update stock level
    ```
    Prompt: "implement PUT /items/{id}/stock endpoint to adjust inventory quantity"
    Expected: Path parsing, JSON parse quantity, sqlite update, return updated item
    ```

12. **GET /items/low-stock** - Low stock report
    ```
    Prompt: "create GET /items/low-stock endpoint to find items with quantity below threshold"
    Expected: sqlite query with WHERE quantity < ?, JSON array response
    ```

13. **DELETE /items/{id}** - Delete item
    ```
    Prompt: "implement DELETE /items/{id} endpoint to remove item from inventory"
    Expected: Path parsing, sqlite delete, return 204
    ```

### Cross-Service Communication

14. **Validate user from inventory service**
    ```
    Prompt: "in inventory service, validate user by calling GET http://localhost:8080/users/{id}"
    Expected: http_get, check status, parse response, handle 404
    ```

## Success Criteria

### Minimum Requirements
- [ ] All 14 prompts generate compilable code
- [ ] Generated code uses correct opcodes and extensions
- [ ] HTTP server patterns are correct (socket, bind, listen, accept)
- [ ] SQLite operations use correct extension IDs
- [ ] JSON handling is properly structured

### Quality Metrics
- [ ] Instruction efficiency: avg < 40 instructions per endpoint
- [ ] No invalid opcodes or register references
- [ ] Proper error handling paths
- [ ] Consistent register conventions (r10=server, r11=client, r14=db)

### Generation Performance
- [ ] Time to generate single endpoint: < 50ms
- [ ] Time to generate full service (7 endpoints): < 500ms
- [ ] Batch generation (14 endpoints): < 1 second

## Test Execution

```bash
# Generate REST API training data
python train/generate_rest_api_data.py

# Merge with base training data
cat training_data.jsonl train/rest_api_training_data.jsonl > train/combined_training_data.jsonl

# Train model (if needed)
python train/train.py --data train/combined_training_data.jsonl

# Run generation tests
cargo test --test generation_tests -- --nocapture

# Or use the CLI
nl generate "create GET /users endpoint returning all users from SQLite as JSON"
```

## RAG Knowledge Base Usage

The model should retrieve from `docs/rag/knowledge_base.md` for:
- Extension ID lookups (e.g., "sqlite_open" → 260)
- Register conventions
- HTTP parsing patterns
- Response building templates

RAG should be preferred over memorization for:
- Extension IDs (400+ possibilities)
- Error response templates
- SQL query patterns
- HTTP protocol details

## Validation Script

```python
def validate_generation(prompt: str, generated_ir: bytes) -> bool:
    """Validate generated code meets requirements."""

    # Decode instructions
    instrs = decode_instructions(generated_ir)

    # Check for required patterns
    checks = {
        'has_server_init': any(i.opcode == NET and i.mode == SOCKET for i in instrs),
        'has_accept': any(i.opcode == NET and i.mode == ACCEPT for i in instrs),
        'has_sqlite': any(i.opcode == EXT and 260 <= i.imm <= 279 for i in instrs),
        'has_json': any(i.opcode == EXT and 200 <= i.imm <= 211 for i in instrs),
        'has_response': any(i.opcode == NET and i.mode == SEND for i in instrs),
        'has_halt_or_loop': any(i.opcode == HALT or (i.opcode == BRANCH and i.imm < 0) for i in instrs),
    }

    # Endpoint-specific validation
    if 'GET' in prompt:
        checks['correct_method'] = True  # Would check method parsing

    if 'POST' in prompt or 'PUT' in prompt:
        checks['has_body_parse'] = any(i.opcode == EXT and i.imm == 200 for i in instrs)

    if '/users/' in prompt or '/items/' in prompt:
        checks['has_path_parse'] = True  # Would check path extraction

    return all(checks.values())
```

## Expected Output Format

For prompt: "create GET /users endpoint returning all users as JSON"

```json
{
  "context": "create GET /users endpoint returning all users as JSON",
  "instructions": [
    {"valid": 1, "opcode": 1, "mode": 0, "rd": 1, "rs1": 0, "rs2": 0, "has_imm": 1, "imm_bin": 2},
    {"valid": 1, "opcode": 1, "mode": 0, "rd": 2, "rs1": 0, "rs2": 0, "has_imm": 1, "imm_bin": 1},
    {"valid": 1, "opcode": 17, "mode": 0, "rd": 10, "rs1": 1, "rs2": 2, "has_imm": 0, "imm_bin": 0},
    // ... more instructions
  ],
  "generated_assembly": "mov r1, 2\nmov r2, 1\nnet.socket r10, r1, r2\n..."
}
```
