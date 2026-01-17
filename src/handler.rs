use crate::error::Error;

/// Trait for defining handlers for access to the Modbus data
///
/// The user application holds all data and defines the access including application side effects.
/// This trait implements defaults (not supported) so the user can choose to only implement the ones
/// actually needed
pub trait ModbusHandler {
    /// Read Coils
    /// # Arguments
    /// - `addr`: Data adress from Modbus request
    /// - `len`: Number of Data from Modbus request
    /// - `out`: output of the requested Coil values. The output buffer is guaranteed to hold the maximum number of coils in one Modbus request (2000)
    fn read_coils(&mut self, _addr: usize, _len: usize, _out: &mut [bool]) -> Result<usize, Error> {
        Err(Error::NotSupported)
    }

    /// Read Discrete Inputs
    /// # Arguments
    /// - `addr`: Data adress from Modbus request
    /// - `len`: Number of Data from Modbus request
    /// - `out`: output of the requested Coil values. The output buffer is guaranteed to hold the maximum number of coils in one Modbus request (2000)
    fn read_discrete_input(
        &mut self,
        _addr: usize,
        _len: usize,
        _out: &mut [bool],
    ) -> Result<usize, Error> {
        Err(Error::NotSupported)
    }

    /// Read Holding Registers
    /// # Arguments
    /// - `addr`: Data adress from Modbus request
    /// - `len`: Number of Data from Modbus request
    /// - `out`: output of the requested register values. The output buffer is guaranteed to hold the maximum number of Registers in one Modbus request (125)
    fn read_holding_registers(
        &mut self,
        _addr: usize,
        _len: usize,
        _out: &mut [u16],
    ) -> Result<usize, Error> {
        Err(Error::NotSupported)
    }

    /// Read Input Registers
    /// # Arguments
    /// - `addr`: Data adress from Modbus request
    /// - `len`: Number of Data from Modbus request
    /// - `out`: output of the requested register values. The output buffer is guaranteed to hold the maximum number of Registers in one Modbus request (125)
    fn read_input_registers(
        &mut self,
        _addr: usize,
        _len: usize,
        _out: &mut [u16],
    ) -> Result<usize, Error> {
        Err(Error::NotSupported)
    }

    /// Write Coils
    /// # Arguments
    /// - `addr`: Data adress (from Modbus request)
    /// - `len`: Number of Coils to write (from Modbus request)
    /// - `buf`: Slice holding the coils to be written
    fn write_coils(&mut self, _addr: usize, _len: usize, _buf: &[bool]) -> Result<usize, Error> {
        Err(Error::NotSupported)
    }

    /// Write Registers
    /// # Arguments
    /// - `addr`: Data adress (from Modbus request)
    /// - `len`: Number of Registers to write (from Modbus request)
    /// - `buf`: Slice holding the registers to be written
    fn write_registers(&mut self, _addr: usize, _len: usize, _buf: &[u16]) -> Result<usize, Error> {
        Err(Error::NotSupported)
    }
}
