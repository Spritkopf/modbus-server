#![no_std]

pub mod error;
pub mod handler;

use error::Error;
use handler::ModbusHandler;
use modbus_core::{
    Coils, Data, ExceptionResponse, FunctionCode, Request, Response, ResponsePdu,
    rtu::{
        Header, ResponseAdu,
        server::{decode_request, encode_response},
    },
};

use crate::error::map_exception;

pub struct ModbusServer<H> {
    unit_id: u8,
    handler: H,
}

impl<H> ModbusServer<H>
where
    H: ModbusHandler,
{
    pub fn new(unit_id: u8, handler: H) -> Self {
        Self { unit_id, handler }
    }

    pub fn process_frame(&mut self, rx: &[u8], tx: &mut [u8]) -> Result<usize, Error> {
        let request = decode_request(rx).unwrap_or_default();

        if let Some(adu) = request {
            let mut buf = [0u8; 250];
            let response: Result<Response, Error> = match adu.pdu.0 {
                Request::ReadCoils(addr, len) => {
                    let mut coils_buf = [false; 2000];

                    // call user handler for read_coils
                    match self
                        .handler
                        .read_coils(addr as usize, len as usize, &mut coils_buf)
                    {
                        Ok(_) => {
                            let coils = Coils::from_bools(&coils_buf[..len as usize], &mut buf).map_err(|_|Error::BufferTooSmall)?;
                            Ok(Response::ReadCoils(coils))
                        }
                        Err(e) => Err(e),
                    }
                }
                Request::ReadDiscreteInputs(addr, len) => {
                    let mut coils_buf = [false; 2000];

                    // call user handler for read_discrete_inputs
                    match self.handler.read_discrete_input(
                        addr as usize,
                        len as usize,
                        &mut coils_buf,
                    ) {
                        Ok(_) => {
                            let coils = Coils::from_bools(&coils_buf[..len as usize], &mut buf).map_err(|_|Error::BufferTooSmall)?;
                            Ok(Response::ReadDiscreteInputs(coils))
                        }
                        Err(e) => Err(e),
                    }
                }
                Request::ReadHoldingRegisters(addr, len) => {
                    let mut reg_buf = [0u16; 125];

                    // call user handler for read_holding_registers
                    match self.handler.read_holding_registers(
                        addr as usize,
                        len as usize,
                        &mut reg_buf,
                    ) {
                        Ok(_) => {
                            let data = Data::from_words(&reg_buf[..len as usize], &mut buf).map_err(|_|Error::BufferTooSmall)?;
                            Ok(Response::ReadHoldingRegisters(data))
                        }
                        Err(e) => Err(e),
                    }
                }
                Request::ReadInputRegisters(addr, len) => {
                    let mut reg_buf = [0u16; 125];

                    // call user handler for read_holding_registers
                    match self.handler.read_input_registers(
                        addr as usize,
                        len as usize,
                        &mut reg_buf,
                    ) {
                        Ok(_) => {
                            let data = Data::from_words(&reg_buf[..len as usize], &mut buf).map_err(|_|Error::BufferTooSmall)?;
                            Ok(Response::ReadInputRegisters(data))
                        }
                        Err(e) => Err(e),
                    }
                }
                Request::WriteSingleCoil(addr, value) => {
                    let coils_buf = [value];

                    // call user handler for read_coils
                    match self.handler.write_coils(addr as usize, 1, &coils_buf) {
                        Ok(num_written_coils) => {
                            // workaround for bug in modbus-core crate: not encoding a response because the
                            // Response::WriteSingleCoil enum is not correct. Since the modbus spec states the response is an
                            // echo of the request, we are just doing that
                            tx[..rx.len()].copy_from_slice(rx);

                            if num_written_coils == 1 {
                                // We return here since we can't build a valid Response for now
                                return Ok(rx.len());
                            } else {
                                Err(Error::Application)
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
                Request::WriteSingleRegister(addr, value) => {
                    let reg_buf = [value];

                    // call user handler for read_coils
                    match self.handler.write_registers(addr as usize, 1, &reg_buf) {
                        Ok(_) => Ok(Response::WriteSingleRegister(addr, value)),
                        Err(e) => Err(e),
                    }
                }
                _ => Err(Error::NotSupported),
            };

            let function_code = match response {
                Ok(ref r) => match r {
                    Response::ReadCoils(_) => FunctionCode::ReadCoils,
                    Response::ReadDiscreteInputs(_) => FunctionCode::ReadDiscreteInputs,
                    Response::ReadHoldingRegisters(_) => FunctionCode::ReadHoldingRegisters,
                    Response::ReadInputRegisters(_) => FunctionCode::ReadInputRegisters,
                    Response::WriteSingleCoil(_) => FunctionCode::WriteSingleCoil,
                    Response::WriteSingleRegister(_, _) => FunctionCode::WriteSingleRegister,
                    _ => FunctionCode::ReadCoils, // Fallback, though should match request
                },
                Err(_) => match adu.pdu.0 {
                    Request::ReadCoils(_, _) => FunctionCode::ReadCoils,
                    Request::ReadDiscreteInputs(_, _) => FunctionCode::ReadDiscreteInputs,
                    Request::ReadHoldingRegisters(_, _) => FunctionCode::ReadHoldingRegisters,
                    Request::ReadInputRegisters(_, _) => FunctionCode::ReadInputRegisters,
                    Request::WriteSingleCoil(_, _) => FunctionCode::WriteSingleCoil,
                    Request::WriteSingleRegister(_, _) => FunctionCode::WriteSingleRegister,
                    _ => FunctionCode::ReadCoils, // Fallback
                },
            };

            let response_pdu = match response {
                Ok(r) => ResponsePdu(Ok(r)),
                Err(e) => ResponsePdu(Err(ExceptionResponse {
                    function: function_code,
                    exception: map_exception(e)
                })),
            };

            let response_adu = ResponseAdu {
                hdr: Header {
                    slave: self.unit_id,
                },
                pdu: response_pdu,
            };

            let tx_len = encode_response(response_adu, tx).map_err(|_| Error::BufferTooSmall)?;
            return Ok(tx_len as usize);
        }
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct TestData {
        test_coils: [bool; 12],
        test_registers: [u16; 12],
    }

    static TEST_COILS: [bool; 12] = [
        false, true, true, false, true, false, true, false, true, true, false, false,
    ];

    static TEST_REGISTERS: [u16; 12] = [
        0, 2, 100, 1000, 3456, 10000, 11111, 11222, 33333, 40987, 55678, 65535,
    ];

    impl ModbusHandler for TestData {
        fn read_coils(
            &mut self,
            addr: usize,
            len: usize,
            out: &mut [bool],
        ) -> Result<usize, Error> {
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

        fn read_discrete_input(
            &mut self,
            addr: usize,
            len: usize,
            out: &mut [bool],
        ) -> Result<usize, Error> {
            // manual memcopy since we have the coils states buffered in memory
            out[..len].copy_from_slice(&TEST_COILS[addr..addr + len]);

            Ok(len)
        }

        fn read_holding_registers(
            &mut self,
            addr: usize,
            len: usize,
            out: &mut [u16],
        ) -> Result<usize, Error> {
            // manual memcopy since we have the registers buffered in memory
            out[..len].copy_from_slice(&TEST_REGISTERS[addr..addr + len]);

            Ok(len)
        }

        fn read_input_registers(
            &mut self,
            addr: usize,
            len: usize,
            out: &mut [u16],
        ) -> Result<usize, Error> {
            // manual memcopy since we have the registers buffered in memory
            out[..len].copy_from_slice(&TEST_REGISTERS[addr..addr + len]);

            Ok(len)
        }

        fn write_coils(&mut self, addr: usize, len: usize, buf: &[bool]) -> Result<usize, Error> {
            // The variant below iterates through the written coils, this way the application
            // could write states to peripherals when the coils are not buffered in memory...
            for (i, slot) in buf.iter().take(len).enumerate() {
                let coil_idx = addr + i;
                self.test_coils[coil_idx] = *slot;
            }

            Ok(len)
        }

        fn write_registers(
            &mut self,
            addr: usize,
            len: usize,
            buf: &[u16],
        ) -> Result<usize, Error> {
            // The variant below iterates through the written regs, this way the application
            // could write states to peripherals when the registers are not buffered in memory...
            for (i, slot) in buf.iter().take(len).enumerate() {
                let reg_idx = addr + i;
                self.test_registers[reg_idx] = *slot;
            }

            Ok(len)
        }
    }

    #[test]
    fn read_single_coil() {
        let mycoil = TestData {
            test_coils: [false; 12],
            test_registers: [0; 12],
        };
        let mut server = ModbusServer::new(1, mycoil);

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
        assert_eq!(len, expected_response.len());
        assert_eq!(response, expected_response);
    }

    #[test]
    fn read_discrete_input() {
        let mycoil = TestData {
            test_coils: [false; 12],
            test_registers: [0; 12],
        };
        let mut server = ModbusServer::new(1, mycoil);

        let frame: [u8; 8] = [
            0x01, // Slave address
            0x02, // Function code: Read Coils
            0x00, 0x02, // Starting address: 2
            0x00, 0x08, // Quantity of coils: 8
            0xD8, 0x0C, // CRC16 (low byte first)
        ];
        let expected_response: [u8; 6] = [0x01, 0x02, 0x01, 0xD5, 0x60, 0x17];
        let mut tx_buf = [0u8; 32];

        let len = server.process_frame(&frame, &mut tx_buf).unwrap();
        let response = &tx_buf[..len];
        assert_eq!(len, expected_response.len());
        assert_eq!(response, expected_response);
    }

    #[test]
    fn read_multiple_coils() {
        let mycoil = TestData {
            test_coils: [false; 12],
            test_registers: [0; 12],
        };
        let mut server = ModbusServer::new(1, mycoil);

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
        assert_eq!(len, expected_response.len());
        assert_eq!(response, expected_response);
    }

    #[test]
    fn read_holding_registers() {
        let mycoil = TestData {
            test_coils: [false; 12],
            test_registers: [0; 12],
        };
        let mut server = ModbusServer::new(1, mycoil);

        let frame: [u8; 8] = [
            0x01, // Slave address
            0x03, // Function code: Read Holding Registers
            0x00, 0x05, // Starting address: 5
            0x00, 0x04, // Quantity of coils: 4
            0x54, 0x08, // CRC16 (low byte first)
        ];
        let expected_response: [u8; 13] = [
            0x01, // Slave Address
            0x03, // Function Code
            0x08, // Byte Count
            0x27, 0x10, // Data byte 0: 10000
            0x2B, 0x67, // Data byte 1: 11111
            0x2B, 0xD6, // Data byte 2: 11222
            0x82, 0x35, // Data byte 3: 33333
            0xBC, 0x90, //CRC
        ];
        let mut tx_buf = [0u8; 32];

        let len = server.process_frame(&frame, &mut tx_buf).unwrap();
        let response = &tx_buf[..len];
        assert_eq!(len, expected_response.len());
        assert_eq!(response, expected_response);
    }

    #[test]
    fn read_input_registers() {
        let mycoil = TestData {
            test_coils: [false; 12],
            test_registers: [0; 12],
        };
        let mut server = ModbusServer::new(1, mycoil);

        let frame: [u8; 8] = [
            0x01, // Slave address
            0x04, // Function code: Read Input Registers
            0x00, 0x00, // Starting address: 0
            0x00, 0x0C, // Quantity of coils: 12
            0xF0, 0x0F, // CRC16 (low byte first)
        ];
        let expected_response: [u8; 29] = [
            0x01, 0x04, 0x18, 0x00, 0x00, 0x00, 0x02, 0x00, 0x64, 0x03, 0xE8, 0x0D, 0x80, 0x27,
            0x10, 0x2B, 0x67, 0x2B, 0xD6, 0x82, 0x35, 0xA0, 0x1B, 0xD9, 0x7E, 0xFF, 0xFF, 0x21,
            0x74,
        ];
        let mut tx_buf = [0u8; 32];

        let len = server.process_frame(&frame, &mut tx_buf).unwrap();
        let response = &tx_buf[..len];
        assert_eq!(len, expected_response.len());
        assert_eq!(response, expected_response);
    }

    #[test]
    fn write_single_coil() {
        let mycoil = TestData {
            test_coils: [false; 12],
            test_registers: [0; 12],
        };
        let mut server = ModbusServer::new(1, mycoil);

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

    #[test]
    fn write_single_register() {
        let mycoil = TestData {
            test_coils: [false; 12],
            test_registers: [0; 12],
        };
        let mut server = ModbusServer::new(1, mycoil);

        let frame: [u8; 8] = [
            0x01, // Slave address
            0x06, // Function code: Write single register
            0x00, 0x08, // Starting address: 3
            0x12, 0x34, // Register Value: 0x1234
            0x05, 0x7F, // CRC16 (low byte first)
        ];
        let expected_response = frame; // repsponse is identical to request frame
        let mut tx_buf = [0u8; 32];

        let len = server.process_frame(&frame, &mut tx_buf).unwrap();
        let response = &tx_buf[..len];
        assert_eq!(response, expected_response);
        assert_eq!(
            server.handler.test_registers,
            [0, 0, 0, 0, 0, 0, 0, 0, 0x1234, 0, 0, 0,]
        )
    }

    // Test exception handling
    struct ExceptionHandler;
    impl ModbusHandler for ExceptionHandler {
        fn read_coils(
            &mut self,
            _addr: usize,
            _len: usize,
            _out: &mut [bool],
        ) -> Result<usize, Error> {
            Err(Error::InvalidAddress)
        }
    }

    #[test]
    fn return_exception() {
        let mut server = ModbusServer::new(1, ExceptionHandler);

        let frame: [u8; 8] = [
            0x01, // Slave address
            0x01, // Function code: Read Coils
            0x00, 0x00, // Starting address: 0
            0x00, 0x01, // Quantity of coils: 1
            0xFD, 0xCA, // CRC16
        ];

        let mut tx_buf = [0u8; 32];
        let len = server.process_frame(&frame, &mut tx_buf).unwrap();
        let response = &tx_buf[..len];

        // Expected: [0x01, 0x81, 0x02, CRC_LO, CRC_HI]
        assert_eq!(len, 5);
        assert_eq!(response[0], 0x01); // Unit ID
        assert_eq!(response[1], 0x81); // Exception Function Code (0x01 | 0x80)
        assert_eq!(response[2], 0x02); // Exception Code (IllegalDataAddress)
    }
}
