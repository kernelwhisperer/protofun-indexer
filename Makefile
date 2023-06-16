ENDPOINT ?= mainnet.eth.streamingfast.io:443
POSTGRESQL_DSN ?= psql://daniel:toor@localhost:5432/protofun?sslmode=disable

START_BLOCK ?= 17491129
STOP_BLOCK  ?= 17491131
# START_BLOCK ?= 12964995
# STOP_BLOCK  ?= 12965005

IPFS_ENDPOINT ?= http://127.0.0.1:5001
GRAPH_NODE_ENDPOINT ?= http://127.0.0.1:8020
GRAPHMAN_CONFIG ?= ../graph-node-dev/config/graphman.toml

.PHONY: codegen
codegen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: package
package: build
	substreams pack -o protofun.spkg ./substreams.yaml

.PHONY: stream
stream: build
	substreams run -e $(ENDPOINT) substreams.yaml map_block --start-block $(START_BLOCK) --stop-block $(STOP_BLOCK)

.PHONY: gui
gui: build
	substreams gui -e $(ENDPOINT) substreams.yaml map_block --start-block $(START_BLOCK) --stop-block $(STOP_BLOCK)

.PHONY: build-db
build-db:
	docker compose up --detach

.PHONY: remove-db
remove-db:
	docker-compose down
	docker volume rm protofun_pgdata

.PHONY: sync-db
sync-db: package
	substreams-sink-postgres run $(POSTGRESQL_DSN) $(ENDPOINT) "protofun.spkg" db_out $(START_BLOCK):$(STOP_BLOCK) 
	# substreams-sink-postgres run $(POSTGRESQL_DSN) $(ENDPOINT) "protofun.spkg" db_out

.PHONY: stream-graph
stream-graph: build
	substreams run -e $(ENDPOINT) substreams.yaml graph_out  --start-block $(START_BLOCK) --stop-block $(STOP_BLOCK)

.PHONY: gui-graph
gui-graph: build
	substreams gui -e $(ENDPOINT) substreams.yaml graph_out  --start-block $(START_BLOCK) --stop-block $(STOP_BLOCK)

.PHONE: start-graph-node
start-graph-node:
	SUBSTREAMS_ENDPOINT=https://$(ENDPOINT) ./graph-node/up.sh

.PHONY: remove-graph-node
remove-graph-node:
	./graph-node/down.sh

.PHONE: deploy-graph
deploy-graph: package
	graph build --ipfs $(IPFS_ENDPOINT) subgraph.yaml
	graph create protofun_block_meta --node $(GRAPH_NODE_ENDPOINT)
	graph deploy --node $(GRAPH_NODE_ENDPOINT) --ipfs $(IPFS_ENDPOINT) --version-label v0.1.0 protofun_block_meta subgraph.yaml

.PHONE: undeploy-graph
undeploy-graph:
	docker exec graph-node graphman drop --force protofun_block_meta
