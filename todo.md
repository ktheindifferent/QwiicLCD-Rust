# TODO List for QwiicLCD-Rust

## Recently Completed Tasks
- [x] Fixed ScreenConfig::new() constructor bug
- [x] Fixed invalid range checks in move_cursor()
- [x] Added error handling in clear() method
- [x] Implemented Default trait properly for structs
- [x] Removed unnecessary code complexity
- [x] Added comprehensive documentation
- [x] Applied rustfmt and clippy fixes
- [x] Updated to Rust 2021 edition
- [x] Updated project documentation (project_description.md, overview.md)
- [x] Created comprehensive unit tests for all public methods
- [x] Added unit tests for the `map()` utility function
- [x] Fixed map() function to handle edge cases (division by zero, overflow)
- [x] Added tests for ScreenConfig with various dimensions
- [x] Added tests for DisplayState management
- [x] Verified all enum constant values
- [x] All tests passing (17 unit tests, 1 hardware integration test)

## Immediate Priority Tasks
- [ ] Create mock I2C device for testing without hardware
- [ ] Add GitHub Actions CI/CD pipeline for automated testing
- [ ] Increase test coverage to >80%
- [ ] Implement functionality for unused enums (EntryShift, MoveType, etc.)
- [ ] Add integration tests for different screen sizes
- [ ] Create property-based tests using quickcheck or proptest

## Potential Future Improvements

### Code Enhancements
- [ ] Consider updating i2cdev dependency to latest version (0.6.1)
- [ ] Remove enum_primitive dependency if not actively used
- [ ] Add builder pattern for Screen configuration
- [ ] Implement Display trait for custom formatting support
- [ ] Add async/await support for non-blocking operations

### Testing
- [ ] Add unit tests for individual methods
- [ ] Create mock I2C device for testing without hardware
- [ ] Add integration tests for different screen sizes (16x2, etc.)
- [ ] Add property-based testing for range validations

### Features
- [ ] Add support for custom characters
- [ ] Implement scrolling text functionality
- [ ] Add animation support
- [ ] Support for multiple screens on different I2C addresses
- [ ] Add screen buffer for offline composition

### Documentation
- [ ] Add examples directory with various use cases
- [ ] Create troubleshooting guide
- [ ] Document hardware setup and wiring
- [ ] Add performance benchmarks
- [ ] Create migration guide from other LCD libraries

### Error Handling
- [ ] Create custom error types instead of using LinuxI2CError directly
- [ ] Add retry logic for I2C communication failures
- [ ] Implement graceful degradation for partial failures

### Platform Support
- [ ] Test and document support for other platforms beyond ARM
- [ ] Add Windows/Mac support with I2C adapters
- [ ] Support for other I2C implementations beyond Linux