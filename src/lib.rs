#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[cfg(windows)]
extern crate pcan_basic_sys;
#[cfg(unix)]
extern crate socketcan;

mod can_message;
mod can_network;
#[cfg(windows)]
mod can_pcan;
mod can_socketcan;
mod can_udp;
mod multican;
#[cfg(feature = "async-tokio")]
pub mod message_codec;
#[cfg(feature = "async-tokio")]
pub mod async_can_udp;

pub use self::can_message::CanMessage;
pub use self::can_network::CanNetwork;
#[cfg(windows)]
pub use self::can_pcan::PcanNetwork;
pub use self::can_socketcan::SocketCanNetwork;
pub use self::can_udp::UdpNetwork;
pub use self::multican::MultiCan;

#[cfg(feature = "async-tokio")]
pub use self::message_codec::CanCodec;
#[cfg(feature = "async-tokio")]
pub use self::async_can_udp::AsyncUdpNetwork;
#[cfg(feature = "async-tokio")]
pub use self::can_network::AsyncCanNetwork;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanConfig {
    /// Bus identifier, can be set to any value.  Use to specify the tx/rs bus for a CanMessage
    pub id: u8,
    /// Bus type - udp, socketcan, pcan
    pub kind: CanBusType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CanBusType {
    SocketCan,
    VirtualSocketCan,
    Pcan,
    Udp,
}

/// Initializes a group of CAN adapters from the specified configuration
///
/// Typical entry point for this library.  CanConfig can be read from a config file
/// or created manually.  Not required to create a multican instance, but useful.
///
/// Note that this function contains some sane defaults, but you may need to
/// customize things like the can network prefix.  If this is the case, construct
/// your adaters manuall and use add_adapter
///
/// # Example
///
/// ```
/// // Set up adapters
/// let cfg = read_config("myfile.toml");
/// let mut network = multican::from_config(cfg);
/// for m in network.recv() {
///     println!("RX: {:?}", m);
/// }
/// ```
pub fn from_config(config: Vec<CanConfig>) -> MultiCan {
    let mut mc = MultiCan::new();
    for net_config in config {
        match net_config.kind {
            CanBusType::SocketCan => {
                #[cfg(unix)]
                {
                    mc.add_adapter(
                        net_config.id,
                        Box::new(SocketCanNetwork::new(net_config.id, "can")),
                    );
                }
                #[cfg(windows)]
                {
                    panic!("Can't use SocketCAN on Windows");
                }
            }
            CanBusType::VirtualSocketCan => {
                #[cfg(unix)]
                {
                    mc.add_adapter(
                        net_config.id,
                        Box::new(SocketCanNetwork::new(net_config.id, "vcan")),
                    );
                }
                #[cfg(windows)]
                {
                    panic!("Can't use SocketCAN on Windows");
                }
            }
            CanBusType::Pcan => {
                #[cfg(unix)]
                {
                    panic!("Can't use PCAN on unix");
                }
                #[cfg(windows)]
                {
                    mc.add_adapter(net_config.id, Box::new(PcanNetwork::new()));
                }
            }
            CanBusType::Udp => {
                mc.add_adapter(net_config.id, Box::new(UdpNetwork::new(net_config.id)));
            }
        };
    }
    mc
}
