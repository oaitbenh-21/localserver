BASE_URL = http://localhost:8080

# ─── Server ───────────────────────────────────────────────────────────────────

build:
	cargo build

run:
	cargo run

# ─── GET ──────────────────────────────────────────────────────────────────────

test-get-home:
	@echo "\n--- GET / ---"
	curl -i $(BASE_URL)/

test-get-file:
	@echo "\n--- GET /uploads/test.txt ---"
	curl -i $(BASE_URL)/uploads/test.txt

test-get-missing:
	@echo "\n--- GET /does-not-exist (expect 404) ---"
	curl -i $(BASE_URL)/does-not-exist

# ─── POST ─────────────────────────────────────────────────────────────────────

test-post:
	@echo "\n--- POST /uploads/test.txt ---"
	curl -i -X POST $(BASE_URL)/uploads/test.txt \
		--data "hello world"

# ─── DELETE ───────────────────────────────────────────────────────────────────

test-delete:
	@echo "\n--- DELETE /uploads/test.txt ---"
	curl -i -X DELETE $(BASE_URL)/uploads/test.txt

test-delete-missing:
	@echo "\n--- DELETE /does-not-exist (expect 404) ---"
	curl -i -X DELETE $(BASE_URL)/does-not-exist

# ─── Full flow ────────────────────────────────────────────────────────────────

test-all:
	@echo "\n========== 1. Upload file =========="
	curl -i -X POST $(BASE_URL)/uploads/test.txt \
		--data "hello world"

	@echo "\n========== 2. Read it back =========="
	curl -i $(BASE_URL)/uploads/test.txt

	@echo "\n========== 3. Delete it =========="
	curl -i -X DELETE $(BASE_URL)/uploads/test.txt

	@echo "\n========== 4. Confirm it's gone (expect 404) =========="
	curl -i $(BASE_URL)/uploads/test.txt

	@echo "\n========== 5. Bad request =========="
	printf "GARBAGE\r\n\r\n" | nc -q 1 localhost 8080

	@echo "\n========== 6. Unknown method (expect 405) =========="
	curl -i -X PATCH $(BASE_URL)/uploads/test.txt

.PHONY: build run \
	test-get-home test-get-file test-get-missing \
	test-post \
	test-delete test-delete-missing \
	test-all
