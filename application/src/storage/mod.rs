use domain::repos::blockchain::BlockchainRepository;
use domain::repos::network::NetworkRepository;
use domain::repos::outbox::OutboxRepository;
use domain::repos::utxo::UtxoRepository;
use std::sync::Arc;

pub trait Storage {
    fn get_blockchain_repo(&self) -> Arc<dyn BlockchainRepository>;

    fn get_utxo_repo(&self) -> Arc<dyn UtxoRepository>;

    fn get_network_repo(&self) -> Arc<dyn NetworkRepository>;

    fn get_outbox_repo(&self) -> Arc<dyn OutboxRepository>;
}
