pub trait AtomicTransactionContext {
    /// Generic method for type identification.
    fn type_id(&self) -> std::any::TypeId;

    fn as_any(&self) -> Box<dyn std::any::Any>;
}
