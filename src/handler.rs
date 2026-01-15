use crate::error::Error;

pub trait ModbusHandler {
    fn read_coils(&mut self, _addr: usize, _len: usize, _out: &mut [bool]) -> Result<usize, Error> {
        Ok(0)
    }
    fn read_discrete_input(
        &mut self,
        _addr: usize,
        _len: usize,
        _out: &mut [bool],
    ) -> Result<usize, Error> {
        Ok(0)
    }
    fn read_holding_registers(
        &mut self,
        _addr: usize,
        _len: usize,
        _out: &mut [u16],
    ) -> Result<usize, Error> {
        Ok(0)
    }
    fn read_input_registers(
        &mut self,
        _addr: usize,
        _len: usize,
        _out: &mut [u16],
    ) -> Result<usize, Error> {
        Ok(0)
    }
    fn write_coils(&mut self, _addr: usize, _len: usize, _buf: &[bool]) -> Result<usize, Error> {
        Ok(0)
    }
    fn write_registers(&mut self, _addr: usize, _len: usize, _buf: &[u16]) -> Result<usize, Error> {
        Ok(0)
    }
}
