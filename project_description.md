# QwiicLCD Rust Library - Bug Fixes and Code Improvements

## Summary of Changes

### Bugs Fixed
1. **ScreenConfig::new() bug** - The constructor was ignoring its parameters and always creating a 4x20 config
2. **Invalid range checks in move_cursor()** - Comparing usize values with < 0 which is impossible
3. **Missing error handling** - The clear() method wasn't propagating errors from write_setting_cmd
4. **Unnecessary parentheses** - Removed extra parentheses in smbus_write_byte_data calls

### Code Quality Improvements
1. **Implemented Default trait properly** - Replaced custom default() methods with proper Default trait implementation
2. **Simplified code** - Removed unnecessary mutable variable in change_backlight, simplified bitwise operations
3. **Removed unused code** - Eliminated unused variable in test_init
4. **Added Rust 2021 edition** - Updated Cargo.toml to use Rust 2021 edition
5. **Code formatting** - Applied cargo fmt for consistent formatting

### Documentation Enhancements
1. **Added comprehensive doc comments** for all public:
   - Enums (Command, EntryMode, DisplayStatus, etc.)
   - Structs (ScreenConfig, DisplayState, Screen)
   - Methods (new, init, clear, home, move_cursor, etc.)
   - Functions (map)

### Verification
- All changes compile without warnings
- Code passes cargo clippy checks
- Code is properly formatted with cargo fmt
- Release build completes successfully

## Impact
These changes improve code reliability, maintainability, and usability of the library while fixing critical bugs that could cause unexpected behavior.