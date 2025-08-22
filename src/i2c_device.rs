use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum I2CError {
    Linux(String),
    Mock(String),
}

impl fmt::Display for I2CError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            I2CError::Linux(msg) => write!(f, "Linux I2C Error: {}", msg),
            I2CError::Mock(msg) => write!(f, "Mock I2C Error: {}", msg),
        }
    }
}

impl Error for I2CError {}

impl From<i2cdev::linux::LinuxI2CError> for I2CError {
    fn from(error: i2cdev::linux::LinuxI2CError) -> Self {
        I2CError::Linux(format!("{:?}", error))
    }
}

pub trait I2CDevice: Send {
    fn smbus_write_byte(&mut self, value: u8) -> Result<(), I2CError>;
    fn smbus_write_byte_data(&mut self, register: u8, value: u8) -> Result<(), I2CError>;
    fn smbus_write_i2c_block_data(&mut self, register: u8, data: &[u8]) -> Result<(), I2CError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum I2CCommand {
    WriteByte(u8),
    WriteByteData(u8, u8),
    WriteBlockData(u8, Vec<u8>),
}

pub struct LinuxI2CDeviceWrapper {
    device: i2cdev::linux::LinuxI2CDevice,
}

impl LinuxI2CDeviceWrapper {
    pub fn new(bus: &str, addr: u16) -> Result<Self, I2CError> {
        use i2cdev::linux::LinuxI2CDevice;
        let device = LinuxI2CDevice::new(bus, addr)?;
        Ok(LinuxI2CDeviceWrapper { device })
    }
}

impl I2CDevice for LinuxI2CDeviceWrapper {
    fn smbus_write_byte(&mut self, value: u8) -> Result<(), I2CError> {
        use i2cdev::core::I2CDevice as I2CDeviceTrait;
        self.device.smbus_write_byte(value)?;
        Ok(())
    }

    fn smbus_write_byte_data(&mut self, register: u8, value: u8) -> Result<(), I2CError> {
        use i2cdev::core::I2CDevice as I2CDeviceTrait;
        self.device.smbus_write_byte_data(register, value)?;
        Ok(())
    }

    fn smbus_write_i2c_block_data(&mut self, register: u8, data: &[u8]) -> Result<(), I2CError> {
        use i2cdev::core::I2CDevice as I2CDeviceTrait;
        self.device.smbus_write_i2c_block_data(register, data)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct MockI2CDevice {
    commands: Arc<Mutex<Vec<I2CCommand>>>,
    responses: Arc<Mutex<VecDeque<Result<(), I2CError>>>>,
    fail_on_command: Arc<Mutex<Option<usize>>>,
    always_fail: Arc<Mutex<bool>>,
}

impl MockI2CDevice {
    pub fn new() -> Self {
        MockI2CDevice {
            commands: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(VecDeque::new())),
            fail_on_command: Arc::new(Mutex::new(None)),
            always_fail: Arc::new(Mutex::new(false)),
        }
    }

    pub fn get_commands(&self) -> Vec<I2CCommand> {
        self.commands.lock().unwrap().clone()
    }

    pub fn clear_commands(&self) {
        self.commands.lock().unwrap().clear();
    }

    pub fn add_response(&self, response: Result<(), I2CError>) {
        self.responses.lock().unwrap().push_back(response);
    }

    pub fn set_fail_on_command(&self, command_index: Option<usize>) {
        *self.fail_on_command.lock().unwrap() = command_index;
    }

    pub fn set_always_fail(&self, fail: bool) {
        *self.always_fail.lock().unwrap() = fail;
    }

    pub fn verify_command_sequence(&self, expected: &[I2CCommand]) -> bool {
        let commands = self.commands.lock().unwrap();
        if commands.len() != expected.len() {
            return false;
        }
        commands.iter().zip(expected.iter()).all(|(a, b)| a == b)
    }

    pub fn verify_command_at(&self, index: usize, expected: &I2CCommand) -> bool {
        let commands = self.commands.lock().unwrap();
        commands.get(index).map_or(false, |cmd| cmd == expected)
    }

    pub fn command_count(&self) -> usize {
        self.commands.lock().unwrap().len()
    }

    fn get_response(&self, command_index: usize) -> Result<(), I2CError> {
        if *self.always_fail.lock().unwrap() {
            return Err(I2CError::Mock("Always fail mode enabled".to_string()));
        }

        if let Some(fail_index) = *self.fail_on_command.lock().unwrap() {
            if command_index == fail_index {
                return Err(I2CError::Mock(format!("Configured to fail at command {}", fail_index)));
            }
        }

        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(Ok(()))
    }
}

impl I2CDevice for MockI2CDevice {
    fn smbus_write_byte(&mut self, value: u8) -> Result<(), I2CError> {
        let mut commands = self.commands.lock().unwrap();
        let command_index = commands.len();
        commands.push(I2CCommand::WriteByte(value));
        drop(commands);
        
        self.get_response(command_index)
    }

    fn smbus_write_byte_data(&mut self, register: u8, value: u8) -> Result<(), I2CError> {
        let mut commands = self.commands.lock().unwrap();
        let command_index = commands.len();
        commands.push(I2CCommand::WriteByteData(register, value));
        drop(commands);
        
        self.get_response(command_index)
    }

    fn smbus_write_i2c_block_data(&mut self, register: u8, data: &[u8]) -> Result<(), I2CError> {
        let mut commands = self.commands.lock().unwrap();
        let command_index = commands.len();
        commands.push(I2CCommand::WriteBlockData(register, data.to_vec()));
        drop(commands);
        
        self.get_response(command_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_device_records_commands() {
        let mut device = MockI2CDevice::new();
        
        device.smbus_write_byte(0x42).unwrap();
        device.smbus_write_byte_data(0x10, 0x20).unwrap();
        device.smbus_write_i2c_block_data(0x30, &[0x40, 0x50]).unwrap();
        
        let commands = device.get_commands();
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], I2CCommand::WriteByte(0x42));
        assert_eq!(commands[1], I2CCommand::WriteByteData(0x10, 0x20));
        assert_eq!(commands[2], I2CCommand::WriteBlockData(0x30, vec![0x40, 0x50]));
    }

    #[test]
    fn test_mock_device_configured_failures() {
        let mut device = MockI2CDevice::new();
        device.set_fail_on_command(Some(1));
        
        assert!(device.smbus_write_byte(0x42).is_ok());
        assert!(device.smbus_write_byte(0x43).is_err());
        assert!(device.smbus_write_byte(0x44).is_ok());
    }

    #[test]
    fn test_mock_device_always_fail() {
        let mut device = MockI2CDevice::new();
        device.set_always_fail(true);
        
        assert!(device.smbus_write_byte(0x42).is_err());
        assert!(device.smbus_write_byte_data(0x10, 0x20).is_err());
    }

    #[test]
    fn test_mock_device_custom_responses() {
        let mut device = MockI2CDevice::new();
        device.add_response(Ok(()));
        device.add_response(Err(I2CError::Mock("Custom error".to_string())));
        device.add_response(Ok(()));
        
        assert!(device.smbus_write_byte(0x42).is_ok());
        assert!(device.smbus_write_byte(0x43).is_err());
        assert!(device.smbus_write_byte(0x44).is_ok());
    }

    #[test]
    fn test_verify_command_sequence() {
        let mut device = MockI2CDevice::new();
        
        device.smbus_write_byte(0x42).unwrap();
        device.smbus_write_byte_data(0x10, 0x20).unwrap();
        
        let expected = vec![
            I2CCommand::WriteByte(0x42),
            I2CCommand::WriteByteData(0x10, 0x20),
        ];
        
        assert!(device.verify_command_sequence(&expected));
        
        let wrong_sequence = vec![
            I2CCommand::WriteByte(0x43),
            I2CCommand::WriteByteData(0x10, 0x20),
        ];
        
        assert!(!device.verify_command_sequence(&wrong_sequence));
    }
}