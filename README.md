# MultiCan

Library that supports different CANBUS scenarios, especially those that require multiple busses to operate at the same time.

Typical usage: 3 busses, using mixed network types
```toml
[[can_networks]]
id = 0
kind = "socketcan"

[[can_networks]]
id = 1
kind = "socketcan"

[[can_networks]]
id = 2
kind = "udp"

```

```rust
// read the config file however you'd like
let network_config = read_config("can.toml");
let network = multican::from_config(network_config);

// receive from all busses:
for rx in network.recv() {
    println!("RX: {:?}", rx);
}

// send a message to bus 2
let m = CanMessage { bus: 2, header: 0x12345678, data: vec![1, 2, 3, 4] };
network.send(m);

```

