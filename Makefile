# This is the address of the default Anvil account deploying it's first contract
anvil_private_key=ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
relay_contract_address=0x5FbDB2315678afecb367f032d93F642f64180aa3
bonsai_api_url=http://localhost:8081
bonsai_api_key=None

up:
	BONSAI_API_URL=$(bonsai_api_url) RELAY_CONTRACT_ADDRESS=$(relay_contract_address) PRIVATE_KEY=$(anvil_private_key) docker compose --profile setup run contract-deployer && \
	BONSAI_API_URL=$(bonsai_api_url) BONSAI_API_KEY=$(bonsai_api_key) RELAY_CONTRACT_ADDRESS=$(relay_contract_address) PRIVATE_KEY=$(anvil_private_key) docker compose --profile main up -d

down:
	docker compose down
