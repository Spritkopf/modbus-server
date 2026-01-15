
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Provided buffer is too small
    BufferTooSmall,
    /// Invalid rResource address
    InvalidAddress,
    /// Invalid value
    InvalidValue,
    /// Request / Function is not supported
    NotSupported,
    /// Application Error
    Application,
}

/// Map crate error codes to modbus exceptions (if applicable)
pub fn map_exception(err: Error) -> modbus_core::Exception {
    match err {
        Error::InvalidAddress => modbus_core::Exception::IllegalDataAddress,
        Error::InvalidValue => modbus_core::Exception::IllegalDataValue,
        Error::NotSupported => modbus_core::Exception::IllegalFunction,
        Error::Application | Error::BufferTooSmall => {
            modbus_core::Exception::ServerDeviceFailure
        }
    }
}
