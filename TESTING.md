# Testing Guide for QwiicLCD-Rust

This project includes comprehensive testing capabilities using mock I2C devices, allowing tests to run without requiring physical hardware.

## Mock I2C Infrastructure

The testing infrastructure consists of:

### 1. I2CDevice Trait
A trait that abstracts I2C operations, allowing both real and mock implementations:
```rust
pub trait I2CDevice: Send {
    fn smbus_write_byte(&mut self, value: u8) -> Result<(), I2CError>;
    fn smbus_write_byte_data(&mut self, register: u8, value: u8) -> Result<(), I2CError>;
    fn smbus_write_i2c_block_data(&mut self, register: u8, data: &[u8]) -> Result<(), I2CError>;
}
```

### 2. MockI2CDevice
A mock implementation that:
- Records all I2C commands sent to it
- Allows verification of command sequences
- Can simulate errors at specific points
- Supports custom responses for testing error handling

### 3. Generic Screen Implementation
The `Screen` struct is now generic over the I2CDevice trait, allowing it to work with both real hardware and mock devices:
```rust
pub struct Screen<D: I2CDevice> {
    dev: D,
    config: ScreenConfig,
    state: DisplayState,
}
```

## Running Tests

### Basic Test Execution
```bash
# Run all tests
cargo test

# Run only unit tests (no hardware required)
cargo test --lib

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_change_backlight
```

### Test Coverage
To measure test coverage, install and use `cargo-tarpaulin`:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage analysis
cargo tarpaulin --out Html

# Generate coverage report in terminal
cargo tarpaulin

# Generate detailed coverage with ignored code excluded
cargo tarpaulin --ignore-panics --ignore-tests
```

## Writing Tests with Mock Devices

### Basic Test Setup
```rust
use crate::*;
use crate::i2c_device::{MockI2CDevice, I2CCommand};

#[test]
fn test_lcd_operation() {
    // Create mock device
    let mock = MockI2CDevice::new();
    let mock_clone = mock.clone();
    
    // Create screen with mock
    let config = ScreenConfig::default();
    let mut screen = Screen::new_with_device(config, mock);
    
    // Perform operations
    screen.clear().unwrap();
    screen.print("Hello").unwrap();
    
    // Verify commands
    let commands = mock_clone.get_commands();
    assert!(commands.contains(&I2CCommand::WriteByteData(0x7C, 0x2D)));
}
```

### Testing Error Conditions
```rust
#[test]
fn test_error_handling() {
    let mut mock = MockI2CDevice::new();
    
    // Configure mock to fail on the third command
    mock.set_fail_on_command(Some(2));
    
    let config = ScreenConfig::default();
    let mut screen = Screen::new_with_device(config, mock);
    
    assert!(screen.clear().is_ok());  // First two commands succeed
    assert!(screen.print("Hi").is_err());  // Third command fails
}
```

### Custom Error Responses
```rust
#[test]
fn test_specific_errors() {
    let mock = MockI2CDevice::new();
    
    // Add custom responses
    mock.add_response(Ok(()));
    mock.add_response(Err(I2CError::Mock("Device busy".to_string())));
    
    let config = ScreenConfig::default();
    let mut screen = Screen::new_with_device(config, mock.clone());
    
    assert!(screen.write_byte(1).is_ok());
    assert!(screen.write_byte(2).is_err());
}
```

### Command Verification
```rust
#[test]
fn test_command_sequence() {
    let mock = MockI2CDevice::new();
    let mut screen = Screen::new_with_device(ScreenConfig::default(), mock.clone());
    
    screen.change_backlight(255, 0, 0).unwrap();
    
    // Verify exact command sequence
    let expected = vec![
        I2CCommand::WriteBlockData(0x7C, vec![0x2B, 255, 0, 0])
    ];
    assert!(mock.verify_command_sequence(&expected));
    
    // Or verify specific command at index
    assert!(mock.verify_command_at(0, &I2CCommand::WriteBlockData(0x7C, vec![0x2B, 255, 0, 0])));
}
```

## Test Categories

### Unit Tests
Located in `src/screen_tests.rs`, these tests cover:
- Screen initialization and configuration
- Cursor movement and positioning
- Text display operations
- Backlight control
- Display state management
- Custom character creation
- Scrolling and cursor navigation
- Error handling

### Integration Tests
The original hardware test (marked with `#[ignore]`) remains available for testing with real hardware:
```bash
# Run hardware tests when device is connected
cargo test -- --ignored
```

## CI/CD Integration

Tests are automatically run via GitHub Actions on:
- Every push to main/master
- Every pull request
- Can be manually triggered

The CI pipeline:
1. Checks out code
2. Sets up Rust toolchain
3. Runs all tests
4. Reports results

## Coverage Goals

Current test coverage targets:
- **Overall**: >80% code coverage
- **Core functionality**: 100% coverage
- **Error paths**: Comprehensive error simulation
- **Edge cases**: All boundary conditions tested

## Adding New Tests

When adding new functionality:
1. Write the mock test first (TDD approach)
2. Implement the feature
3. Ensure mock tests pass
4. Add hardware test if applicable (with `#[ignore]` attribute)
5. Update this documentation if new patterns are introduced

## Troubleshooting

### Common Issues

1. **Mock not recording commands**: Ensure you're using the cloned mock for verification
2. **Tests timing out**: Mock operations are instant, no need for delays in tests
3. **Command mismatch**: Use `println!("{:?}", mock.get_commands())` to debug

### Debug Helpers

```rust
// Print all recorded commands
let commands = mock.get_commands();
for (i, cmd) in commands.iter().enumerate() {
    println!("{}: {:?}", i, cmd);
}

// Check command count
println!("Total commands: {}", mock.command_count());
```

## Future Enhancements

Potential improvements to the testing infrastructure:
- [ ] Add performance benchmarks
- [ ] Implement property-based testing
- [ ] Add fuzzing for command sequences
- [ ] Create test fixtures for common scenarios
- [ ] Add integration tests with mock I2C bus simulator