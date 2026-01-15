#![no_std]

use modbus_core::{
    Coils, Error, Request, Response, ResponsePdu,
    rtu::{
        Header, ResponseAdu,
        server::{decode_request, encode_response},
    },
};
// TODO: add library error type

pub trait ModbusHandler {
    fn read_coils(&mut self, addr: usize, len: usize, out: &mut [bool]) -> Result<usize, Error>;
    fn on_write(&mut self, addr: usize, len: usize, buf: &[bool]) -> Result<usize, Error>;
}

pub struct ModbusRtuServer<H> {
    unit_id: u8,
    handler: H,
}

impl<H> ModbusRtuServer<H>
where
    H: ModbusHandler,
{
    pub fn new(unit_id: u8, handler: H) -> Self {
        Self {
            unit_id,
            handler,
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
                    let handler_result =
                        self.handler
                            .read_coils(addr as usize, len as usize, &mut coils_buf)?;

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
                Request::WriteSingleCoil(addr, value) => {
                    let coils_buf = [value];

                    // call user handler for read_coils
                    let num_written_coils =
                        self.handler.on_write(addr as usize, 1, &coils_buf)?;

                    // workaround for bug in modbus-core crate: not encoding a response because the
                    // Response::WriteSingleCoil enum is not correct. Since the modbus spec states the response is an
                    // echo of the request, we are just doing that
                    tx[..rx.len()].copy_from_slice(rx);

                    if num_written_coils == 1 {
                        return Ok(rx.len());
                    }
                }
                // TODO: modbus-core crate has a bug that breaks mnultiple coil write. PR is open, revisit
                // later...
                // Request::WriteMultipleCoils(addr, coils) => {
                //     let mut coils_buf = [false; 32];
                //     for (i, coil) in coils.into_iter().enumerate() {
                //         coils_buf[i] = coil;
                //     }
                //
                //     // call user handler for read_coils
                //     let num_written_coils =
                //         self.coil_handler
                //             .on_write(addr as usize, coils.len(), &coils_buf)?;
                //
                //     let response = Response::WriteMultipleCoils(addr, num_written_coils as u16);
                //     let response_adu = ResponseAdu {
                //         hdr: Header {
                //             slave: self.unit_id,
                //         },
                //         pdu: ResponsePdu(Ok(response)),
                //     };
                //     let tx_len = encode_response(response_adu, tx).ok().unwrap();
                //     return Ok(tx_len as usize);
                // }
                _ => {}
            }
        }
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct TestData {
        test_coils: [bool; 12],
    }

    static TEST_COILS: [bool; 12] = [
        false, true, true, false, true, false, true, false, true, true, false, false,
    ];

    impl ModbusHandler for TestData {
        fn on_write(&mut self, addr: usize, len: usize, buf: &[bool]) -> Result<usize, Error> {
            // The variant below iterates through the written coils, this way the application
            // could write states to peripherals when the coils are not buffered in memory...
            for (i, slot) in buf.iter().take(len).enumerate() {
                let coil_idx = addr + i;
                self.test_coils[coil_idx] = *slot;
            }

            Ok(len)
        }

        fn read_coils(&mut self, addr: usize, len: usize, out: &mut [bool]) -> Result<usize, Error> {
            // manual memcopy since we have the coils states buffered in memory
            out[..len].copy_from_slice(&TEST_COILS[addr..addr + len]);

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
        let mycoil = TestData {
            test_coils: [false; 12],
        };
        let mut server = ModbusRtuServer::new(1, mycoil);

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
        let mycoil = TestData {
            test_coils: [false; 12],
        };
        let mut server = ModbusRtuServer::new(1, mycoil);

        let frame: [u8; 8] = [
            0x01, // Slave address
            0x01, // Function code: Read Coils
            0x00, 0x03, // Starting address: 3
            0x00, 0x04, // Quantity of coils: 4
            0xCD, 0xC9, // CRC16 (low byte first)
        ];
        let expected_response: [u8; 6] = [0x01, 0x01, 0x01, 0x0A, 0xD1, 0x8F]; // Data byte: 0x0A (0:0 1:1 2:0 3:1)
        let mut tx_buf = [0u8; 32];

        let len = server.process_frame(&frame, &mut tx_buf).unwrap();
        let response = &tx_buf[..len];
        assert_eq!(len, 6);
        assert_eq!(response, expected_response);
    }

    #[test]
    fn write_single_coil() {
        let mycoil = TestData {
            test_coils: [false; 12],
        };
        let mut server = ModbusRtuServer::new(1, mycoil);

        let frame: [u8; 8] = [
            0x01, // Slave address
            0x05, // Function code: Write single coil
            0x00, 0x03, // Starting address: 3
            0xFF, 0x00, // Coil Value: 0xFF (ON)
            0x7C, 0x3A, // CRC16 (low byte first)
        ];
        let expected_response = frame; // repsponse is identical to request frame
        let mut tx_buf = [0u8; 32];

        let len = server.process_frame(&frame, &mut tx_buf).unwrap();
        let response = &tx_buf[..len];
        assert_eq!(response, expected_response);
        assert_eq!(
            server.handler.test_coils,
            [
                false, false, false, true, false, false, false, false, false, false, false, false
            ]
        )
    }

    // TODO: modbus-core crate has a bug that breaks mnultiple coil write. PR is open, revisit
    // later...
    // #[test]
    // fn write_multiple_coils() {
    //     let mycoil = MyCoil {
    //         test_coils: [false; 12],
    //     };
    //     let mut server = ModbusRtuServer::new(1, mycoil);
    //
    //     let frame: [u8; 10] = [
    //         0x01, // Slave address
    //         0x0F, // Function code: Write multiple coils
    //         0x00, 0x03, // Starting address: 3
    //         0x00, 0x04, // Coil count: 4
    //         0x01, // Byte count: 1
    //         0x0D, // Data byte: (3:1 4:0 5:1 6:1)
    //         0xBB, 0x53, // CRC16 (low byte first)
    //     ];
    //     let expected_response: [u8; 8] = [0x01, 0x0F, 0x00, 0x03, 0x00, 0x04, 0xA4, 0x08];
    //     let mut tx_buf = [0u8; 32];
    //
    //     let len = server.process_frame(&frame, &mut tx_buf).unwrap();
    //     let response = &tx_buf[..len];
    //     assert_eq!(len, 8);
    //     assert_eq!(response, expected_response);
    //     assert_eq!(
    //         server.coil_handler.test_coils,
    //         [
    //             false, false, false, true, false, true, true, false, false, false, false, false
    //         ]
    //     )
    // }
}
