use hula_types::Battery;
use std::sync::{Arc, Mutex};
use zbus::{
    blocking::{Connection, ConnectionBuilder},
    dbus_interface,
    zvariant::Optional,
    Error,
};

use crate::SharedState;
use constants::{HULA_DBUS_PATH, HULA_DBUS_SERVICE};

struct RobotInfo {
    shared_state: Arc<Mutex<SharedState>>,
}

#[dbus_interface(name = "org.hulks.hula")]
impl RobotInfo {
    fn head_id(&self) -> Optional<String> {
        let configuration = self.shared_state.lock().unwrap().configuration;
        Optional::from(configuration.and_then(|configuration| {
            let head_id = configuration.head_id.to_vec();
            String::from_utf8(head_id).ok()
        }))
    }

    fn body_id(&self) -> Optional<String> {
        let configuration = self.shared_state.lock().unwrap().configuration;
        Optional::from(configuration.and_then(|configuration| {
            let body_id = configuration.body_id.to_vec();
            String::from_utf8(body_id).ok()
        }))
    }

    fn battery(&self) -> Optional<Battery> {
        Optional::from(self.shared_state.lock().unwrap().battery)
    }
}

pub fn serve_dbus(shared_state: Arc<Mutex<SharedState>>) -> Result<Connection, Error> {
    let robot_info = RobotInfo { shared_state };
    ConnectionBuilder::system()?
        .name(HULA_DBUS_SERVICE)?
        .serve_at(HULA_DBUS_PATH, robot_info)?
        .build()
}
