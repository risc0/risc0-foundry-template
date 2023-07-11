# This is the address of the default Anvil account deploying it's first contract
anvil_private_key=ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
relay_contract_address=0x5FbDB2315678afecb367f032d93F642f64180aa3
anvil_chain_id=31337
ifndef BONSAI_API_URL
	BONSAI_API_URL=http://localhost:8081
endif
ifndef BONSAI_API_KEY
	BONSAI_API_KEY=None
endif

bonsai_test_relay_path=./lib/bonsai-lib-sol/src/BonsaiRelay.sol:BonsaiRelay
bonsai_starter_contract_path=./contracts/BonsaiStarter.sol:BonsaiStarter
constructor_args=None

up:
	$(MAKE) deploy-contract contract_path=$(bonsai_test_relay_path)
	@BONSAI_API_URL=$(BONSAI_API_URL) \
	BONSAI_API_KEY=$(BONSAI_API_KEY) \
	RELAY_CONTRACT_ADDRESS=$(relay_contract_address) \
	PRIVATE_KEY=$(anvil_private_key) \
	ETH_CHAIN_ID=$(anvil_chain_id) \
	CONTRACT_PATH= \
	CONSTRUCTOR_ARGS='$(constructor_args)' \
	docker compose --profile main up -d

deploy-bonsai-starter-contract: set-image-id
	$(MAKE) \
	contract_path=$(bonsai_starter_contract_path) \
	constructor_args='$(relay_contract_address) $(image_id)' \
	deploy-contract

set-image-id:
	$(eval image_id = $(shell BONSAI_API_URL=$(BONSAI_API_URL) BONSAI_API_KEY=$(BONSAI_API_KEY) cargo run -q -- upload | grep -E -o '[0-9a-fA-F]{64}'))

# make deploy-contract contract_path=<path_to_contract_sol>:<contract_name> constructor_args=[optional] anvil_private_key=[optional]
deploy-contract:
	@BONSAI_API_URL= \
	BONSAI_API_KEY= \
	RELAY_CONTRACT_ADDRESS=$(relay_contract_address) \
	PRIVATE_KEY=$(anvil_private_key) \
	ETH_CHAIN_ID=$(anvil_chain_id) \
	CONTRACT_PATH=$(contract_path) \
	CONSTRUCTOR_ARGS='$(constructor_args)' \
	docker compose --profile setup run contract-deployer

down:
	@BONSAI_API_URL= \
	BONSAI_API_KEY= \
	RELAY_CONTRACT_ADDRESS= \
	PRIVATE_KEY= \
	ETH_CHAIN_ID= \
	CONTRACT_PATH= \
	CONSTRUCTOR_ARGS= \
	docker compose --profile setup --profile main down
