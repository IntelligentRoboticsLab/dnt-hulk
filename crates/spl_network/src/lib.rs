pub mod endpoint;
pub mod message_receiver;

#[derive(Clone, Copy, Debug)]
pub enum CyclerInstance {
    SplNetwork,
}

pub use spl_network_messages::GameControllerReturnMessage;
