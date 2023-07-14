ifndef PRIVATE_KEY
	# This is the address of the default Anvil account deploying it's first contract
	PRIVATE_KEY=ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
endif

ifndef RELAY_CONTRACT_ADDRESS
	# anvil deterministic address from default account
	RELAY_CONTRACT_ADDRESS=0x5FbDB2315678afecb367f032d93F642f64180aa3
endif

ifndef CHAIN_ID
	CHAIN_ID=31337 # anvil chain id
endif

ifndef BONSAI_API_URL
	BONSAI_API_URL=http://localhost:8081
endif

ifndef BONSAI_API_KEY
	BONSAI_API_KEY=None
endif

ifndef ENVIRONMENT
	ENVIRONMENT=local
endif

ifndef ETH_WS_NODE_URL
	ETH_WS_NODE_URL=ws://anvil:8545
endif

ifndef ETH_HTTP_NODE_URL
	ETH_HTTP_NODE_URL=http://anvil:8545
endif

bonsai_relay_contract_path=./relay/contracts/BonsaiRelay.sol:BonsaiRelay
bonsai_starter_contract_path=./contracts/BonsaiStarter.sol:BonsaiStarter
constructor_args=None
guest_binary=FIBONACCI

up: deploy-bonsai-relay-contract
	@BONSAI_API_URL=$(BONSAI_API_URL) \
	BONSAI_API_KEY=$(BONSAI_API_KEY) \
	ETH_WS_NODE_URL=$(ETH_WS_NODE_URL) \
	ETH_HTTP_NODE_URL= \
	RELAY_CONTRACT_ADDRESS=$(relay_address) \
	PRIVATE_KEY=$(PRIVATE_KEY) \
	ETH_CHAIN_ID=$(CHAIN_ID) \
	CONTRACT_PATH= \
	CONSTRUCTOR_ARGS='$(constructor_args)' \
	docker compose --profile $(ENVIRONMENT) up -d

deploy-bonsai-relay-contract:
	$(eval relay_address = $(shell $(MAKE) deploy-contract contract_path=$(bonsai_relay_contract_path) | grep -E -o 'Deployed to: 0x([0-9a-fA-F]{40})' | grep -E -o '0x[0-9a-fA-F]{40}'))
	@echo ------------------------------------------------------------------
	@echo Relay Contract Address: $(relay_address)
	@echo ------------------------------------------------------------------

deploy-bonsai-starter-contract: set-image-id
	$(MAKE) \
	contract_path=$(bonsai_starter_contract_path) \
	constructor_args='$(RELAY_CONTRACT_ADDRESS) $(image_id)' \
	deploy-contract

set-image-id:
	$(eval image_id = $(shell BONSAI_API_URL=$(BONSAI_API_URL) BONSAI_API_KEY=$(BONSAI_API_KEY) cargo run -q -- upload --guest-binary $(guest_binary) | grep -E -o '[0-9a-fA-F]{64}'))
	@echo ---------------------------------------------------------------------------
	@echo Image ID: $(image_id)
	@echo ---------------------------------------------------------------------------

# make deploy-contract contract_path=<path_to_contract_sol>:<contract_name> constructor_args=[optional] PRIVATE_KEY=[optional]
deploy-contract:
	@BONSAI_API_URL= \
	BONSAI_API_KEY= \
	ETH_WS_NODE_URL= \
	ETH_HTTP_NODE_URL=$(ETH_HTTP_NODE_URL) \
	RELAY_CONTRACT_ADDRESS=$(RELAY_CONTRACT_ADDRESS) \
	PRIVATE_KEY=$(PRIVATE_KEY) \
	ETH_CHAIN_ID=$(CHAIN_ID) \
	CONTRACT_PATH=$(contract_path) \
	CONSTRUCTOR_ARGS='$(constructor_args)' \
	docker compose --profile $(ENVIRONMENT) run contract-deployer

down:
	@BONSAI_API_URL= \
	BONSAI_API_KEY= \
	ETH_WS_NODE_URL= \
	ETH_HTTP_NODE_URL= \
	RELAY_CONTRACT_ADDRESS= \
	PRIVATE_KEY= \
	ETH_CHAIN_ID= \
	CONTRACT_PATH= \
	CONSTRUCTOR_ARGS= \
	docker compose --profile local --profile development down
