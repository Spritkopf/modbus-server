#![no_std]

use modbus_core::{
    rtu::{decode, server::decode_request},
    DecoderType, 
    Error,
    Request
};

pub trait CoilHandler {
    fn on_write(&mut self, value: bool);
    fn on_read(&mut self, value: bool);
}

pub struct ModbusRtuServer<H> {
    unit_id: u8,
    coil: bool,
    handler: H,
}

impl<H> ModbusRtuServer<H>
where
    H: CoilHandler,
{
    pub fn new(unit_id: u8, handler: H) -> Self {
        Self {
            unit_id,
            coil: false,
            handler,
        }
    }

    pub fn process_frame(&mut self, rx: &[u8], tx: &mut [u8]) -> Result<usize, Error> {
        // let frame_option = decode(DecoderType::Request, rx)?;
        //
        // let (frame, location) = match frame_option {
        //     Some((frame, location)) => (frame, location),
        //     None => return Err(Error::ProtocolNotModbus(0)),
        // };
        let request = decode_request(rx).unwrap_or_default();

        if let Some(adu) = request {
            match adu.pdu.0 {
                Request::ReadCoils(addr, len) => { 
                    tx[0] = len as u8;
                    return Ok(len as usize);
                },
                _ => {}
            }
        }
        Ok(0)
        // // TODO: return None! (option return type)
        // if frame.slave != self.unit_id {
        //     return Err(Error::BufferSize);
        // }
        // let request = RtuRequest::parse(frame.pdu()).map_err(|_| Error::InvalidFrame)?;
        //
        // match request.function_code() {
        //     FunctionCode::ReadCoils => {
        //         // address == 0, quantity == 1
        //         let value = self.coil;
        //         self.handler.on_read(value);
        //
        //         let response = RtuFrame::new_read_coils_response(self.unit_id, &[value], tx);
        //
        //         Ok(response)
        //     }
        //
        //     FunctionCode::WriteSingleCoil => {
        //         let value = request.coil_value().unwrap();
        //         self.coil = value;
        //         self.handler.on_write(value);
        //
        //         let response = RtuFrame::new_write_single_coil_response(
        //             self.unit_id,
        //             request.address(),
        //             value,
        //             tx,
        //         );
        //
        //         Ok(response)
        //     }
        //
        //     _ => Err(Error::UnsupportedFunction),
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MyCoil;

    impl CoilHandler for MyCoil {
        fn on_write(&mut self, value: bool) {
            // toggle GPIO, drive relay, etc.
            if value {
                // set pin high
            } else {
                // set pin low
            }
        }

        fn on_read(&mut self, _value: bool) {
            // optional: sync state from hardware
        }
    }
    #[test]
    fn it_works() {
        let mut server = ModbusRtuServer::new(1, MyCoil);

        let frame: [u8; 8] = [
            0x01, // Slave address
            0x01, // Function code: Read Coils
            0x00, 0x00, // Starting address: 0
            0x00, 0x01, // Quantity of coils: 1
            0xFD, 0xCA, // CRC16 (low byte first)
        ];
        let mut tx_buf = [0u8; 32];

        match server.process_frame(&frame, &mut tx_buf) {
            Ok(len) => {
                let response = &tx_buf[..len];
                assert_eq!(len, 1);
                // send via UART
            }
            Err(e) => {
                // assert_eq!(e, Error::);
            }
        }
    }
}
