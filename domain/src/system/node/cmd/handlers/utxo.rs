use super::super::CommandResponder;
use super::CommandHandlerControlFlow;
use crate::entities::transaction::{TransactionOutPoint, Utxo};
use crate::system::utxo::UtxoSetReader;
use common::error::AppError;
use common::log_node_debug;
use std::sync::Arc;

/// Handles UTXO-related commands.
#[derive(Debug, Clone)]
pub(crate) struct UtxoCommandHandler {
    utxo_set_r: Arc<dyn UtxoSetReader>,
}

impl UtxoCommandHandler {
    pub(crate) fn new(utxo_set_r: Arc<dyn UtxoSetReader>) -> Self {
        Self { utxo_set_r }
    }

    /// Get UTXOs by their outpoints.
    pub(in crate::system::node) async fn handle_get_utxos_by_outpoints(
        &self,
        outpoints: Vec<TransactionOutPoint>,
        responder: Box<dyn CommandResponder<Result<Vec<Utxo>, AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!(
            "UtxoCommandHandler: Getting {} UTXO(s) by outpoint",
            outpoints.len()
        );

        let res = self.utxo_set_r.get_multiple_utxos_by_outpoints(&outpoints);

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }

    /// Get all UTXOs.
    pub(in crate::system::node) async fn handle_get_utxos(
        &self,
        responder: Box<dyn CommandResponder<Result<Vec<Utxo>, AppError>> + Send>,
    ) -> Result<CommandHandlerControlFlow, AppError> {
        log_node_debug!("UtxoCommandHandler: Getting all UTXOs");

        let res = self.utxo_set_r.get_multiple_utxos();

        responder.respond(res);
        Ok(CommandHandlerControlFlow::Continue)
    }
}
