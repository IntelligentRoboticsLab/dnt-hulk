use std::net::SocketAddr;

use log::warn;
use spl_network::GameControllerReturnMessage;
use tokio::net::UdpSocket;

pub async fn send_game_controller_return_message(
    game_controller_state_messages: &UdpSocket,
    last_game_controller_address: &Option<SocketAddr>,
    message: GameControllerReturnMessage,
) {
    let game_controller_address = match last_game_controller_address {
        Some(game_controller_address) => game_controller_address,
        None => {
            // Unknown GameController address, silently skipping return message sending
            return;
        }
    };

    let message: Vec<u8> = match message.try_into() {
        Ok(message) => message,
        Err(error) => {
            warn!(
                "Failed to parse GameController return message (will be discarded): {:?}",
                error
            );
            return;
        }
    };

    match game_controller_state_messages
        .send_to(
            message.as_slice(),
            SocketAddr::new(game_controller_address.ip(), 3939),
        )
        .await
    {
        Ok(_) => {}
        Err(error) => warn!("Failed to send GameController return message: {:?}", error),
    }
}
