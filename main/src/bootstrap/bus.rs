use domain::system::node::bus::{CommandReceiver, CommandSender};
use infrastructure::bus::build_channel;
use std::sync::Arc;

pub(crate) fn build_cmd_bus() -> (Arc<dyn CommandSender>, Box<dyn CommandReceiver>) {
    const CMD_CHANNEL_BUFFER_SIZE: usize = 100;

    let (sender, receiver) = build_channel(CMD_CHANNEL_BUFFER_SIZE);
    (Arc::new(sender), Box::new(receiver))
}
