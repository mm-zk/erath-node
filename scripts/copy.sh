L1_CONTRACT_DIR="contracts/l1-contracts/artifacts/cache/solpp-generated-contracts"

cp ${L1_CONTRACT_DIR}/zksync/facets/Getters.sol/GettersFacet.json src/compiled_contracts
cp ${L1_CONTRACT_DIR}/zksync/facets/Admin.sol/AdminFacet.json src/compiled_contracts
cp ${L1_CONTRACT_DIR}/zksync/facets/Executor.sol/ExecutorFacet.json src/compiled_contracts
cp ${L1_CONTRACT_DIR}/zksync/facets/Mailbox.sol/MailboxFacet.json src/compiled_contracts
cp ${L1_CONTRACT_DIR}/zksync/Verifier.sol/Verifier.json src/compiled_contracts



cp ${L1_CONTRACT_DIR}/zksync/DiamondInit.sol/DiamondInit.json src/compiled_contracts
cp ${L1_CONTRACT_DIR}/zksync/DiamondProxy.sol/DiamondProxy.json src/compiled_contracts
