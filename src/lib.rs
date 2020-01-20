
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
        if net_config.kind == "udp" {
			mc.add_adapter(net_config.id, Box::new(UdpNetwork::new(net_config.id)));
        } else if net_config.kind == "socketcan" {
            #[cfg(unix)]
            {
				// adding a parameter for the prefix would be nice for socketcan
				// otherwise i can't use this for can/vcan
				mc.add_adapter(net_config.id, Box::new(SocketCanNetwork::new(net_config.id, "vcan")));
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
				mc.add_adapter(net_config.id, Box::new(PcanNetwork::new()));
            }
        } else {
            error!("Unknown CAN network type: {}", net_config.kind);
        }
    }
	mc
}
