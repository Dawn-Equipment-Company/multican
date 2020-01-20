extern crate multican;
use multican::CanConfig;

// to see something, try cansend can2 123#4570

fn main() {
	let cfg = vec![
		CanConfig { id: 0, kind: "socketcan".to_string() },
		CanConfig { id: 1, kind: "socketcan".to_string() },
		CanConfig { id: 2, kind: "socketcan".to_string() },
		// you can also mix network kinds
		CanConfig { id: 3, kind: "udp".to_string() },
	];
	let mut network = multican::from_config(cfg);
	loop {
		for message in network.recv() {
			println!("RX: {:?}", message);
		}
	}
}
