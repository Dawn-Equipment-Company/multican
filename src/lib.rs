
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[cfg(unix)]
extern crate socketcan;
#[cfg(windows)]
extern crate pcan_basic_sys;

mod can_message;
mod can_network;
mod multican;
mod can_udp;
mod can_socketcan;
#[cfg(windows)]
mod can_pcan;

pub use self::can_message::CanMessage;
pub use self::can_network::CanNetwork;
pub use self::multican::MultiCan;
pub use self::can_udp::UdpNetwork;
pub use self::can_socketcan::SocketCanNetwork;
#[cfg(windows)]
pub use self::can_pcan::PcanNetwork;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanConfig {
    /// Bus identifier, can be set to any value.  Use to specify the tx/rs bus for a CanMessage
    pub id: u8,
    // can i make this an enum?
    /// Bus type - udp, socketcan, pcan
    pub kind: String,
}

/// Initializes a group of CAN adapters from the specified configuration
///
/// Typical entry point for this library.  CanConfig can be read from a config file
/// or created manually.  Not required to create a multican instance, but useful.
///
/// # Example
///
/// ```
/// // Set up adapters
/// let adapters = setup_can(config);
/// let network = MultiCan::new(&mut adapters);
/// for m in network.recv() {
///     println!("RX: {:?}", m);
/// }
/// ```
pub fn setup_can(config: Vec<CanConfig>) -> Vec<Box<dyn CanNetwork + Send>> {
    let mut adapters: Vec<Box<dyn CanNetwork + Send>> = Vec::new();
    for net_config in config {
        if net_config.kind == "udp" {
            adapters.push(Box::new(UdpNetwork::new(net_config.id)));
        } else if net_config.kind == "socketcan" {
            #[cfg(unix)]
            {
                adapters.push(Box::new(SocketCanNetwork::new(net_config.id, "can")));
            }
            #[cfg(windows)]
            {
                panic!("Can't use SocketCAN on Windows");
            }
        } else if net_config.kind == "pcan" {
            #[cfg(unix)]
            {
                panic!("Can't use PCAN on unix");
            }
            #[cfg(windows)]
            {
                adapters.push(Box::new(PcanNetwork::new()));
            }
        } else {
            error!("Unknown CAN network type: {}", net_config.kind);
        }
    }
    adapters
}
