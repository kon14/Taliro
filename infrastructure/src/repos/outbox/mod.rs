use crate::ext::{AppErrorExtInfrastructure, TransactionContextExtInfrastructure};
use crate::storage::SledStorage;
use common::error::AppError;
use common::tx::ctx::AtomicTransactionContext;
use domain::encode::{TryDecode, TryEncode};
use domain::repos::outbox::OutboxRepository;
use domain::types::outbox::OutboxEntry;
use sled::Transactional;
use sled::Tree;
use sled::transaction::ConflictableTransactionError;
use std::fmt::{Debug, Formatter};

pub struct SledOutboxRepository {
    outbox_unprocessed_tree: Tree,
    outbox_processed_tree: Tree,
}

impl Debug for SledOutboxRepository {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SledBlockchainRepository")
            .field(
                "outbox_unprocessed_tree",
                &SledStorage::OUTBOX_UNPROCESSED_TREE,
            )
            .field("outbox_processed_tree", &SledStorage::OUTBOX_PROCESSED_TREE)
            .finish()
    }
}

impl SledOutboxRepository {
    pub fn open(
        outbox_unprocessed_tree: Tree,
        outbox_processed_tree: Tree,
    ) -> Result<Self, AppError> {
        let repo = Self {
            outbox_unprocessed_tree,
            outbox_processed_tree,
        };
        Ok(repo)
    }
}

impl OutboxRepository for SledOutboxRepository {
    fn insert_entry(
        &self,
        tx_ctx: Option<&dyn AtomicTransactionContext>,
        entry: OutboxEntry,
    ) -> Result<(), AppError> {
        let key = Self::get_key(&entry);
        let data = entry.try_encode()?;
        if let Some(tx_ctx) = tx_ctx {
            let outbox_unprocessed_tree = tx_ctx.get_outbox_unprocessed_tree()?;
            outbox_unprocessed_tree.insert(&*key, data).to_app_error()?;
        } else {
            self.outbox_unprocessed_tree
                .insert(&key, data)
                .to_app_error()?;
        }
        Ok(())
    }

    fn get_unprocessed_entries(&self) -> Result<Vec<OutboxEntry>, AppError> {
        self.outbox_unprocessed_tree
            .iter()
            .map(|res| {
                let (_key, value) = res.to_app_error()?;
                OutboxEntry::try_decode(&value)
            })
            .collect()
    }

    fn mark_entry_as_processed(&self, entry: &OutboxEntry) -> Result<(), AppError> {
        let key = Self::get_key(entry);

        let tx_res = (&self.outbox_unprocessed_tree, &self.outbox_processed_tree).transaction(
            |(unprocessed_tree, processed_tree)| {
                let entry_data = unprocessed_tree.get(&key)?;
                let Some(entry_data) = entry_data else {
                    return Err(ConflictableTransactionError::Abort(AppError::not_found(
                        format!("Outbox entry ({}) not found!", entry.get_id()),
                    )));
                };

                let mut entry = OutboxEntry::try_decode(&entry_data)
                    .map_err(ConflictableTransactionError::Abort)?;
                entry.mark_processed();

                let data = entry
                    .try_encode()
                    .map_err(ConflictableTransactionError::Abort)?;
                processed_tree.insert(&*key, data.clone()).map_err(|err| {
                    ConflictableTransactionError::Abort(AppError::internal(err.to_string()))
                })?;
                unprocessed_tree.remove(&*key).map_err(|err| {
                    ConflictableTransactionError::Abort(AppError::internal(err.to_string()))
                })?;
                Ok(())
            },
        );
        tx_res.to_app_error()
    }
}

impl SledOutboxRepository {
    fn get_key(entry: &OutboxEntry) -> Vec<u8> {
        format!("{}:{}", entry.get_event().get_event_type(), entry.get_id()).into_bytes()
    }
}
