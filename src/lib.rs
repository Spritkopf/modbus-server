#![no_std]

use modbus_core::{
    Coils, Error, Request, Response, ResponsePdu, rtu::{
        Header, ResponseAdu, server::{decode_request, encode_response},
    },
};
// TODO: add library error type


pub trait CoilHandler {
    fn on_write(&mut self, value: bool);
    fn on_read(&mut self, addr: usize, len: usize, buf: &mut [bool]) -> Result<usize, Error>;
}

pub struct ModbusRtuServer<H> {
    unit_id: u8,
    coil_handler: H,
}

impl<H> ModbusRtuServer<H>
where
    H: CoilHandler,
{
    pub fn new(unit_id: u8, coil_handler: H) -> Self {
        Self {
            unit_id,
            coil_handler,
        }
    }

    pub fn process_frame(&mut self, rx: &[u8], tx: &mut [u8]) -> Result<usize, Error> {
        let request = decode_request(rx).unwrap_or_default();

        if let Some(adu) = request {
            match adu.pdu.0 {
                Request::ReadCoils(addr, len) => {
                    let mut buf = [0u8; 255];
                    let mut coils_buf = [false; 255];

                    // call user handler for read_coils
                    let handler_result = self.coil_handler.on_read(addr as usize, len as usize, &mut coils_buf)?;

                    let coils = Coils::from_bools(&coils_buf[..len as usize], &mut buf)?;
                    let response = Response::ReadCoils(coils);
                    let response_adu = ResponseAdu {
                        hdr: Header {
                            slave: self.unit_id,
                        },
                        pdu: ResponsePdu(Ok(response)),
                    };
                    let tx_len = encode_response(response_adu, tx).ok().unwrap();
                    return Ok(tx_len as usize);
                }
                _ => {}
            }
        }
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MyCoil;

    static TEST_COILS: [bool; 12] = [
        false, true, true, false, true, false, true, false, true, true, false, false,
    ];

    impl CoilHandler for MyCoil {
        fn on_write(&mut self, value: bool) {
            // toggle GPIO, drive relay, etc.
            if value {
                // set pin high
            } else {
                // set pin low
            }
        }

        fn on_read(&mut self, addr: usize, len: usize, buf: &mut [bool]) -> Result<usize, Error> {
            // manual memcopy since we have the coils states buffered in memory
            buf[..len].copy_from_slice(&TEST_COILS[addr..addr + len]);

            // The variant below iterates through the requested coils, this way the application
            // could read individual states from peripherals when the coils are not buffered in memory...
            // for (i, slot) in buf.iter_mut().take(len).enumerate() {
            //     let coil_idx = addr + i;
            //     *slot = TEST_COILS[coil_idx];
            // }
            Ok(len)
        }
    }
    #[test]
    fn read_single_coil() {
        let mut server = ModbusRtuServer::new(1, MyCoil);

        let frame: [u8; 8] = [
            0x01, // Slave address
            0x01, // Function code: Read Coils
            0x00, 0x00, // Starting address: 0
            0x00, 0x01, // Quantity of coils: 1
            0xFD, 0xCA, // CRC16 (low byte first)
        ];
        let expected_response: [u8; 6] = [0x01, 0x01, 0x01, 0x00, 0x51, 0x88];
        let mut tx_buf = [0u8; 32];

        let len = server.process_frame(&frame, &mut tx_buf).unwrap();
                let response = &tx_buf[..len];
                assert_eq!(len, 6);
                assert_eq!(response, expected_response);
    }

    #[test]
    fn read_multiple_coils() {
        let mut server = ModbusRtuServer::new(1, MyCoil);

        let frame: [u8; 8] = [
            0x01, // Slave address
            0x01, // Function code: Read Coils
            0x00, 0x03, // Starting address: 3
            0x00, 0x04, // Quantity of coils: 4
            0xCD, 0xC9, // CRC16 (low byte first)
        ];
        let expected_response: [u8; 6] = [0x01, 0x01, 0x01, 0x0A, 0xD1, 0x8F];
        let mut tx_buf = [0u8; 32];

        let len = server.process_frame(&frame, &mut tx_buf).unwrap();
                let response = &tx_buf[..len];
                assert_eq!(len, 6);
                assert_eq!(response, expected_response);
    }
}
