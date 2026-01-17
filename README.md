# modbus-server

A `no-std` Mobus server library. It is built on top of `modbus-core` and offers a convenience layer for parsing RTU
frames and defining callbacks for accessing various data. The crate does not hold any data like coils or holding
registers. The user application handles this data via the callbacks defined in the `ModbusHandler` trait.

The crate has no dependency of any HAL components and only operates on raw byte buffers. The user application is
responsible to handle the serial communication part.

## Features

* RTU only
* Supports Coils, Discrete Inputs, Registers (Input / Holding)
* Individual callbacks for each data type
* Data types can be (de-)selected by cargo features (default=all)

### Support Request Types

- [X] Read Coils
- [X] Read Discrete inputs
- [X] Read Holding Registers
- [X] Read Input Registers
- [X] Write Single Coil
- [X] Write Single Register
- [ ] Write Multiple Coils
- [ ] Write Multiple Registers

## Example

```rust
use modbus_server::{ModbusServer, handler::ModbusHandler};

let rx_frame = uart.read; // placeholder, get an RTU frame 

struct MyHandler;
impl ModbusHandler for MyHandler {
// [...] implement handler functions you need
}
let handler = MyHandler {};
let mut server = ModbusServer::new(1, handler); // 1 is the device slave ID
let mut tx_buf = [0u8; 256]; // make sure a full modbus message can fit into the buffer

if let Ok(len) = server.process_frame(&rx_frame, &mut tx_buf) {
    // handle your uart transmission 
    // uart.write(&tx_buf[..len]);
}
```

## Todo

- [ ] Implement missing requests (wait for modbus-core crate update)
- [X] Crate Error type
- [X] Error handling
- [X] Documentation
- [X] Examples
- [X] Memory usage (external buffers?)
- [ ] Satisfy Modbus Spec state diagrams

### Ideas

- Feature flag that changes the default implementation of the handler trait (no-op or not supported)
- Optional feature: helper module to handling uart timing, dependency to embedded-hal (timer)
