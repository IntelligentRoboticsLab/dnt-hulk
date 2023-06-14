use clap::Args;
use color_eyre::Result;

use nao::Nao;

use crate::parsers::NaoAddress;

#[derive(Args)]
pub struct Arguments {
    /// The NAO to update the IP address e.g. 10.0.8.42
    #[arg(required = true)]
    pub nao: NaoAddress,

    /// The third octet of the new ip address for the robot.
    pub first_octet: u8,

    /// The last octet of the new ip address for the robot.
    pub second_octet: u8,
}

pub async fn change_ip(arguments: Arguments) -> Result<()> {
    let nao = Nao::new(arguments.nao.ip);
    println!(
        "{}",
        nao.set_last_ip_octet(arguments.first_octet, arguments.second_octet)
            .await?
    );

    Ok(())
}
