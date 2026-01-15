use modbus_core::Error as ModbusError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Modbus(ModbusError),
    BufferTooSmall,
    Exception(modbus_core::Exception),
}

impl From<ModbusError> for Error {
    fn from(e: ModbusError) -> Self {
        Error::Modbus(e)
    }
}
