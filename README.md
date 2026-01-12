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

## Example

```rust
    println!("Coming Soon");
```

## Todo

- [ ] Implement missing requests
    - [X] Read Coils
    - [X] Read Discrete inputs
    - [X] Read Holding Registers
    - [X] Read Input Registers
    - [X] Write Single Coil
    - [ ] Write Single Register
    - [ ] Write Multiple Coils
    - [ ] Write Multiple Registers

- [ ] Crate Error type
- [ ] Error handling
- [ ] Cargo features
- [ ] Documentation
- [ ] Examples
- [ ] Memory usage (internal buffers, maximum message size) (external buffers?)
