use crate::ext::AppErrorExtInfrastructure;
use crate::repos::blockchain::SledBlockchainRepository;
use crate::repos::network::SledNetworkRepository;
use crate::repos::outbox::SledOutboxRepository;
use crate::repos::utxo::SledUtxoRepository;
use application::storage::Storage;
use common::config::storage::StorageConfig;
use common::error::AppError;
use domain::repos::blockchain::BlockchainRepository;
use domain::repos::network::NetworkRepository;
use domain::repos::outbox::OutboxRepository;
use domain::repos::utxo::UtxoRepository;
use std::sync::Arc;

pub struct SledStorage {
    #[allow(unused)]
    cfg: StorageConfig,
    #[allow(unused)]
    db: Arc<sled::Db>,
    blockchain_repo: Arc<dyn BlockchainRepository>,
    utxo_repo: Arc<dyn UtxoRepository>,
    network_repo: Arc<dyn NetworkRepository>,
    outbox_repo: Arc<dyn OutboxRepository>,
}

impl SledStorage {
    pub(crate) const OUTBOX_PROCESSED_TREE: &'static str = "outbox_processed";
    pub(crate) const OUTBOX_UNPROCESSED_TREE: &'static str = "outbox_unprocessed";
    pub(crate) const BLOCKCHAIN_BLOCKS_TREE: &'static str = "blockchain_blocks";
    pub(crate) const BLOCKCHAIN_HEIGHTS_TREE: &'static str = "blockchain_heights";
    pub(crate) const BLOCKCHAIN_META_TREE: &'static str = "blockchain_meta";
    pub(crate) const BLOCKCHAIN_META_TREE_TIP_KEY: &'static str = "chain_tip";
    pub(crate) const UTXO_TREE: &'static str = "utxo";
    pub(crate) const NETWORK_PEER_ADDRESS_TREE: &'static str = "network_peers";
    pub(crate) const NETWORK_META_TREE: &'static str = "network_meta";
    pub(crate) const NETWORK_META_TREE_IDENTITY_KEY_PAIR_KEY: &'static str = "identity_key_pair";

    pub fn open(cfg: StorageConfig) -> Result<Self, AppError> {
        let db = sled::open(&cfg.db_path).to_app_error()?;
        let db = Arc::new(db);

        let outbox_processed_tree = db.open_tree(Self::OUTBOX_PROCESSED_TREE).to_app_error()?;
        let outbox_unprocessed_tree = db.open_tree(Self::OUTBOX_UNPROCESSED_TREE).to_app_error()?;
        let blockchain_blocks_tree = db.open_tree(Self::BLOCKCHAIN_BLOCKS_TREE).to_app_error()?;
        let blockchain_heights_tree = db.open_tree(Self::BLOCKCHAIN_HEIGHTS_TREE).to_app_error()?;
        let blockchain_meta_tree = db.open_tree(Self::BLOCKCHAIN_META_TREE).to_app_error()?;
        let utxo_tree = db.open_tree(Self::UTXO_TREE).to_app_error()?;
        let peer_address_tree = db
            .open_tree(Self::NETWORK_PEER_ADDRESS_TREE)
            .to_app_error()?;
        let network_meta_tree = db.open_tree(Self::NETWORK_META_TREE).to_app_error()?;

        let blockchain_repo = SledBlockchainRepository::open(
            blockchain_blocks_tree,
            blockchain_heights_tree,
            blockchain_meta_tree,
            outbox_unprocessed_tree.clone(),
        )?;
        let utxo_repo = SledUtxoRepository::open(utxo_tree)?;
        let network_repo = SledNetworkRepository::open(peer_address_tree, network_meta_tree)?;
        let outbox_repo =
            SledOutboxRepository::open(outbox_unprocessed_tree, outbox_processed_tree)?;
        let storage = Self {
            cfg,
            db,
            blockchain_repo: Arc::new(blockchain_repo),
            utxo_repo: Arc::new(utxo_repo),
            network_repo: Arc::new(network_repo),
            outbox_repo: Arc::new(outbox_repo),
        };
        Ok(storage)
    }
}

impl Storage for SledStorage {
    fn get_blockchain_repo(&self) -> Arc<dyn BlockchainRepository> {
        self.blockchain_repo.clone()
    }

    fn get_utxo_repo(&self) -> Arc<dyn UtxoRepository> {
        self.utxo_repo.clone()
    }

    fn get_network_repo(&self) -> Arc<dyn NetworkRepository> {
        self.network_repo.clone()
    }

    fn get_outbox_repo(&self) -> Arc<dyn OutboxRepository> {
        self.outbox_repo.clone()
    }
}
