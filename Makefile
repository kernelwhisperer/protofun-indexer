ENDPOINT ?= mainnet.eth.streamingfast.io:443
POSTGRESQL_DSN ?= psql://daniel:toor@localhost:5432/protofun?sslmode=disable

START_BLOCK ?= 12965000
STOP_BLOCK  ?= 12966000

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: stream
stream: build
	substreams run -e $(ENDPOINT) substreams.yaml map_block --start-block $(START_BLOCK) --stop-block $(STOP_BLOCK)

.PHONY: codegen
codegen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"

.PHONY: package
package: build
	substreams pack -o protofun.spkg ./substreams.yaml

.PHONY: build-db
build-db:
	docker compose up --detach

.PHONY: remove-db
remove-db:
	docker-compose down
	docker volume rm protofun_pgdata

.PHONY: sync-db
sync-db: package
	# substreams-sink-postgres run $(POSTGRESQL_DSN) $(ENDPOINT) "protofun.spkg" db_out $(START_BLOCK):$(STOP_BLOCK) 
	substreams-sink-postgres run $(POSTGRESQL_DSN) $(ENDPOINT) "protofun.spkg" db_out
