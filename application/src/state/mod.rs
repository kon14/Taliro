use crate::auth::master_key::MasterKeyAuthenticator;
use crate::usecases::dev;
use domain::system::node::bus::{CommandResponderFactory, CommandSender};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    // Authentication
    pub master_key_authenticator: Arc<dyn MasterKeyAuthenticator + Send + Sync>,
    // Development Use Cases
    pub generate_wallet_use_case: dev::GenerateWalletUseCase,
    pub init_genesis_use_case: dev::InitiateGenesisUseCase,
    pub get_blockchain_tip_info_use_case: dev::blockchain::GetBlockchainTipInfoUseCase,
    pub get_blockchain_block_use_case: dev::blockchain::blocks::GetBlockchainBlockUseCase,
    pub get_blockchain_blocks_by_height_range_use_case:
        dev::blockchain::blocks::GetBlockchainBlocksByHeightRangeUseCase,
    pub adhoc_mine_block_use_case: dev::blockchain::blocks::AdHocMineBlockUseCase,
    pub get_network_self_info_use_case: dev::network::GetNetworkSelfInfoUseCase,
    pub get_network_peers_use_case: dev::network::GetNetworkPeersUseCase,
    pub add_network_peer_use_case: dev::network::AddNetworkPeerUseCase,
    pub get_mempool_transactions_use_case:
        dev::transactions::mempool::GetMempoolTransactionsUseCase,
    pub place_unconfirmed_transaction_use_case:
        dev::transactions::PlaceUnconfirmedTransactionUseCase,
    pub get_utxos_use_case: dev::transactions::utxo::GetUtxosUseCase,
}

impl AppState {
    pub fn new(
        bus_tx: Arc<dyn CommandSender>,
        bus_tx_res_factory: Arc<dyn CommandResponderFactory>,
        master_key_authenticator: Arc<dyn MasterKeyAuthenticator + Send + Sync>,
    ) -> Self {
        let generate_wallet_use_case = dev::GenerateWalletUseCase::new();
        let init_genesis_use_case =
            dev::InitiateGenesisUseCase::new(bus_tx.clone(), bus_tx_res_factory.clone());
        let get_blockchain_tip_info_use_case = dev::blockchain::GetBlockchainTipInfoUseCase::new(
            bus_tx.clone(),
            bus_tx_res_factory.clone(),
        );
        let get_blockchain_block_use_case = dev::blockchain::blocks::GetBlockchainBlockUseCase::new(
            bus_tx.clone(),
            bus_tx_res_factory.clone(),
        );
        let get_blockchain_blocks_by_height_range_use_case =
            dev::blockchain::blocks::GetBlockchainBlocksByHeightRangeUseCase::new(
                bus_tx.clone(),
                bus_tx_res_factory.clone(),
            );
        let adhoc_mine_block_use_case = dev::blockchain::blocks::AdHocMineBlockUseCase::new(
            bus_tx.clone(),
            bus_tx_res_factory.clone(),
        );
        let get_network_self_info_use_case = dev::network::GetNetworkSelfInfoUseCase::new(
            bus_tx.clone(),
            bus_tx_res_factory.clone(),
        );
        let get_network_peers_use_case =
            dev::network::GetNetworkPeersUseCase::new(bus_tx.clone(), bus_tx_res_factory.clone());
        let add_network_peer_use_case =
            dev::network::AddNetworkPeerUseCase::new(bus_tx.clone(), bus_tx_res_factory.clone());
        let get_mempool_transactions_use_case =
            dev::transactions::mempool::GetMempoolTransactionsUseCase::new(
                bus_tx.clone(),
                bus_tx_res_factory.clone(),
            );
        let place_unconfirmed_transaction_use_case =
            dev::transactions::PlaceUnconfirmedTransactionUseCase::new(
                bus_tx.clone(),
                bus_tx_res_factory.clone(),
            );
        let get_utxos_use_case = dev::transactions::utxo::GetUtxosUseCase::new(
            bus_tx.clone(),
            bus_tx_res_factory.clone(),
        );

        Self {
            // Authentication
            master_key_authenticator,
            // Development Use Cases
            generate_wallet_use_case,
            init_genesis_use_case,
            get_blockchain_tip_info_use_case,
            get_blockchain_block_use_case,
            get_blockchain_blocks_by_height_range_use_case,
            adhoc_mine_block_use_case,
            get_network_self_info_use_case,
            get_network_peers_use_case,
            add_network_peer_use_case,
            get_mempool_transactions_use_case,
            place_unconfirmed_transaction_use_case,
            get_utxos_use_case,
        }
    }
}
