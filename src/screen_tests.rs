#[cfg(test)]
mod tests {
    use crate::*;
    use crate::i2c_device::{MockI2CDevice, I2CCommand, I2CError};

    fn create_test_screen() -> (Screen<MockI2CDevice>, MockI2CDevice) {
        let mock = MockI2CDevice::new();
        let mock_clone = mock.clone();
        let config = ScreenConfig::default();
        let screen = Screen::new_with_device(config, mock);
        (screen, mock_clone)
    }

    fn create_test_screen_with_config(rows: u8, cols: u8) -> (Screen<MockI2CDevice>, MockI2CDevice) {
        let mock = MockI2CDevice::new();
        let mock_clone = mock.clone();
        let config = ScreenConfig::new(rows, cols);
        let screen = Screen::new_with_device(config, mock);
        (screen, mock_clone)
    }

    #[test]
    fn test_screen_init() {
        let (mut screen, mock) = create_test_screen();
        
        screen.init().unwrap();
        
        let commands = mock.get_commands();
        
        // First command should be apply_display_state with defaults (all on)
        assert!(commands.contains(&I2CCommand::WriteByteData(254, 0x0F)));
        
        // Should contain clear command
        assert!(commands.contains(&I2CCommand::WriteByteData(0x7C, 0x2D)));
        
        // Should contain return home
        assert!(commands.contains(&I2CCommand::WriteByteData(254, 0x02)));
        
        // Final state should be display on, cursor off, blink off
        assert!(commands.contains(&I2CCommand::WriteByteData(254, 0x0C)));
    }

    #[test]
    fn test_change_backlight() {
        let (mut screen, mock) = create_test_screen();
        
        screen.change_backlight(128, 64, 32).unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteBlockData(0x7C, vec![0x2B, 128, 64, 32]));
    }

    #[test]
    fn test_change_backlight_full_colors() {
        let (mut screen, mock) = create_test_screen();
        
        screen.change_backlight(255, 255, 255).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteBlockData(0x7C, vec![0x2B, 255, 255, 255]));
        
        mock.clear_commands();
        screen.change_backlight(0, 0, 0).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteBlockData(0x7C, vec![0x2B, 0, 0, 0]));
    }

    #[test]
    fn test_clear() {
        let (mut screen, mock) = create_test_screen();
        
        screen.clear().unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], I2CCommand::WriteByteData(0x7C, 0x2D));
        assert_eq!(commands[1], I2CCommand::WriteByteData(254, 0x02));
    }

    #[test]
    fn test_home() {
        let (mut screen, mock) = create_test_screen();
        
        screen.home().unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x02));
    }

    #[test]
    fn test_move_cursor_valid_positions() {
        let (mut screen, mock) = create_test_screen();
        
        screen.move_cursor(0, 0).unwrap();
        assert_eq!(mock.get_commands()[0], I2CCommand::WriteByteData(254, 0x80));
        
        mock.clear_commands();
        screen.move_cursor(1, 0).unwrap();
        assert_eq!(mock.get_commands()[0], I2CCommand::WriteByteData(254, 0x80 | 0x40));
        
        mock.clear_commands();
        screen.move_cursor(2, 0).unwrap();
        assert_eq!(mock.get_commands()[0], I2CCommand::WriteByteData(254, 0x80 | 0x14));
        
        mock.clear_commands();
        screen.move_cursor(3, 0).unwrap();
        assert_eq!(mock.get_commands()[0], I2CCommand::WriteByteData(254, 0x80 | 0x54));
        
        mock.clear_commands();
        screen.move_cursor(0, 5).unwrap();
        assert_eq!(mock.get_commands()[0], I2CCommand::WriteByteData(254, 0x80 | 0x05));
        
        mock.clear_commands();
        screen.move_cursor(1, 10).unwrap();
        assert_eq!(mock.get_commands()[0], I2CCommand::WriteByteData(254, 0x80 | (0x40 + 10)));
    }

    #[test]
    fn test_move_cursor_out_of_bounds() {
        let (mut screen, mock) = create_test_screen();
        
        screen.move_cursor(4, 0).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0F));
        
        mock.clear_commands();
        screen.move_cursor(0, 20).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0F));
        
        mock.clear_commands();
        screen.move_cursor(10, 30).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0F));
    }

    #[test]
    fn test_move_cursor_different_screen_sizes() {
        let (mut screen, mock) = create_test_screen_with_config(2, 16);
        
        screen.move_cursor(1, 15).unwrap();
        assert_eq!(mock.get_commands()[0], I2CCommand::WriteByteData(254, 0x80 | (0x40 + 15)));
        
        mock.clear_commands();
        screen.move_cursor(2, 0).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0F));
        
        mock.clear_commands();
        screen.move_cursor(0, 16).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0F));
    }

    #[test]
    fn test_enable_cursor() {
        let (mut screen, mock) = create_test_screen();
        
        screen.enable_cursor(true).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0F));
        
        mock.clear_commands();
        screen.enable_cursor(false).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0D));
    }

    #[test]
    fn test_enable_display() {
        let (mut screen, mock) = create_test_screen();
        
        screen.enable_display(false).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0B));
        
        mock.clear_commands();
        screen.enable_display(true).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0F));
    }

    #[test]
    fn test_enable_blink() {
        let (mut screen, mock) = create_test_screen();
        
        screen.enable_blink(false).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0E));
        
        mock.clear_commands();
        screen.enable_blink(true).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0F));
    }

    #[test]
    fn test_display_state_combinations() {
        let (mut screen, mock) = create_test_screen();
        
        screen.enable_display(true).unwrap();
        screen.enable_cursor(false).unwrap();
        screen.enable_blink(false).unwrap();
        mock.clear_commands();
        screen.apply_display_state().unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0C));
        
        screen.enable_display(false).unwrap();
        screen.enable_cursor(true).unwrap();
        screen.enable_blink(false).unwrap();
        mock.clear_commands();
        screen.apply_display_state().unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x0A));
        
        screen.enable_display(false).unwrap();
        screen.enable_cursor(false).unwrap();
        screen.enable_blink(true).unwrap();
        mock.clear_commands();
        screen.apply_display_state().unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x09));
    }

    #[test]
    fn test_print() {
        let (mut screen, mock) = create_test_screen();
        
        screen.print("Hello").unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 5);
        assert_eq!(commands[0], I2CCommand::WriteByte(b'H'));
        assert_eq!(commands[1], I2CCommand::WriteByte(b'e'));
        assert_eq!(commands[2], I2CCommand::WriteByte(b'l'));
        assert_eq!(commands[3], I2CCommand::WriteByte(b'l'));
        assert_eq!(commands[4], I2CCommand::WriteByte(b'o'));
    }

    #[test]
    fn test_print_empty_string() {
        let (mut screen, mock) = create_test_screen();
        
        screen.print("").unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_print_special_characters() {
        let (mut screen, mock) = create_test_screen();
        
        screen.print("!@#$%").unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 5);
        assert_eq!(commands[0], I2CCommand::WriteByte(b'!'));
        assert_eq!(commands[1], I2CCommand::WriteByte(b'@'));
        assert_eq!(commands[2], I2CCommand::WriteByte(b'#'));
        assert_eq!(commands[3], I2CCommand::WriteByte(b'$'));
        assert_eq!(commands[4], I2CCommand::WriteByte(b'%'));
    }

    #[test]
    fn test_print_with_spaces() {
        let (mut screen, mock) = create_test_screen();
        
        screen.print("Hello World").unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 11);
        assert_eq!(commands[5], I2CCommand::WriteByte(b' '));
    }

    #[test]
    fn test_write_byte() {
        let (mut screen, mock) = create_test_screen();
        
        screen.write_byte(0x42).unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByte(0x42));
    }

    #[test]
    fn test_write_block() {
        let (mut screen, mock) = create_test_screen();
        
        screen.write_block(0x10, vec![0x01, 0x02, 0x03]).unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteBlockData(0x10, vec![0x01, 0x02, 0x03]));
    }

    #[test]
    fn test_write_setting_cmd() {
        let (mut screen, mock) = create_test_screen();
        
        screen.write_setting_cmd(0x42).unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(0x7C, 0x42));
    }

    #[test]
    fn test_write_special_cmd() {
        let (mut screen, mock) = create_test_screen();
        
        screen.write_special_cmd(0x42).unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x42));
    }

    #[test]
    fn test_complex_sequence() {
        let (mut screen, mock) = create_test_screen();
        
        screen.init().unwrap();
        screen.change_backlight(255, 0, 0).unwrap();
        screen.clear().unwrap();
        screen.move_cursor(1, 5).unwrap();
        screen.print("Test").unwrap();
        screen.enable_cursor(false).unwrap();
        
        let commands = mock.get_commands();
        
        assert!(commands.contains(&I2CCommand::WriteBlockData(0x7C, vec![0x2B, 255, 0, 0])));
        
        assert!(commands.contains(&I2CCommand::WriteByteData(0x7C, 0x2D)));
        
        assert!(commands.contains(&I2CCommand::WriteByteData(254, 0x80 | (0x40 + 5))));
        
        assert!(commands.contains(&I2CCommand::WriteByte(b'T')));
        assert!(commands.contains(&I2CCommand::WriteByte(b'e')));
        assert!(commands.contains(&I2CCommand::WriteByte(b's')));
        assert!(commands.contains(&I2CCommand::WriteByte(b't')));
    }

    #[test]
    fn test_error_handling() {
        let mut mock = MockI2CDevice::new();
        mock.set_always_fail(true);
        let config = ScreenConfig::default();
        let mut screen = Screen::new_with_device(config, mock);
        
        assert!(screen.clear().is_err());
        assert!(screen.home().is_err());
        assert!(screen.move_cursor(0, 0).is_err());
        assert!(screen.print("test").is_err());
        assert!(screen.change_backlight(255, 255, 255).is_err());
    }

    #[test]
    fn test_error_at_specific_command() {
        let (mut screen, mock) = create_test_screen();
        mock.set_fail_on_command(Some(2));
        
        assert!(screen.clear().is_ok());
        
        assert!(screen.print("Hi").is_err());
    }

    #[test]
    fn test_custom_error_responses() {
        let (mut screen, mock) = create_test_screen();
        
        mock.add_response(Ok(()));
        mock.add_response(Err(I2CError::Mock("Device busy".to_string())));
        mock.add_response(Ok(()));
        
        assert!(screen.write_byte(1).is_ok());
        
        let result = screen.write_byte(2);
        assert!(result.is_err());
        if let Err(I2CError::Mock(msg)) = result {
            assert_eq!(msg, "Device busy");
        }
        
        assert!(screen.write_byte(3).is_ok());
    }

    #[test]
    fn test_screen_with_device_creation() {
        let mock = MockI2CDevice::new();
        let config = ScreenConfig::new(2, 16);
        let screen = Screen::new_with_device(config, mock);
        
        assert_eq!(screen.config.max_rows, 2);
        assert_eq!(screen.config.max_columns, 16);
    }

    #[test]
    fn test_verify_init_command_sequence() {
        let (mut screen, mock) = create_test_screen();
        
        screen.init().unwrap();
        
        let has_display_on = mock.verify_command_at(0, &I2CCommand::WriteByteData(254, 0x0F));
        let has_clear = mock.get_commands().iter().any(|cmd| {
            *cmd == I2CCommand::WriteByteData(0x7C, 0x2D)
        });
        let has_blink_off = mock.get_commands().iter().any(|cmd| {
            *cmd == I2CCommand::WriteByteData(254, 0x0C)
        });
        let has_cursor_off = mock.get_commands().iter().any(|cmd| {
            *cmd == I2CCommand::WriteByteData(254, 0x0D)
        });
        
        assert!(has_display_on || has_clear || has_blink_off || has_cursor_off);
    }

    #[test]
    fn test_set_contrast() {
        let (mut screen, mock) = create_test_screen();
        
        screen.set_contrast(128).unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], I2CCommand::WriteByteData(0x7C, 0x18));
        assert_eq!(commands[1], I2CCommand::WriteByteData(0x7C, 128));
        
        mock.clear_commands();
        screen.set_contrast(255).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[1], I2CCommand::WriteByteData(0x7C, 255));
        
        mock.clear_commands();
        screen.set_contrast(0).unwrap();
        let commands = mock.get_commands();
        assert_eq!(commands[1], I2CCommand::WriteByteData(0x7C, 0));
    }

    #[test]
    fn test_create_character() {
        let (mut screen, mock) = create_test_screen();
        
        let heart = [0x00, 0x0A, 0x1F, 0x1F, 0x0E, 0x04, 0x00, 0x00];
        screen.create_character(0, &heart).unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x40));
        assert_eq!(commands[1], I2CCommand::WriteByte(0x00));
        assert_eq!(commands[2], I2CCommand::WriteByte(0x0A));
        assert_eq!(commands[3], I2CCommand::WriteByte(0x1F));
        assert_eq!(commands[4], I2CCommand::WriteByte(0x1F));
        assert_eq!(commands[5], I2CCommand::WriteByte(0x0E));
        assert_eq!(commands[6], I2CCommand::WriteByte(0x04));
        assert_eq!(commands[7], I2CCommand::WriteByte(0x00));
        assert_eq!(commands[8], I2CCommand::WriteByte(0x00));
        assert_eq!(commands[9], I2CCommand::WriteByteData(254, 0x02));
    }

    #[test]
    fn test_create_character_different_locations() {
        let (mut screen, mock) = create_test_screen();
        
        let pattern = [0xFF; 8];
        
        screen.create_character(3, &pattern).unwrap();
        assert_eq!(mock.get_commands()[0], I2CCommand::WriteByteData(254, 0x40 | (3 << 3)));
        
        mock.clear_commands();
        screen.create_character(7, &pattern).unwrap();
        assert_eq!(mock.get_commands()[0], I2CCommand::WriteByteData(254, 0x40 | (7 << 3)));
    }

    #[test]
    fn test_create_character_invalid_location() {
        let (mut screen, mock) = create_test_screen();
        
        let pattern = [0xFF; 8];
        screen.create_character(8, &pattern).unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_create_character_short_pattern() {
        let (mut screen, mock) = create_test_screen();
        
        let pattern = [0xFF; 5];
        screen.create_character(0, &pattern).unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_scroll_display_left() {
        let (mut screen, mock) = create_test_screen();
        
        screen.scroll_display_left().unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x18));
    }

    #[test]
    fn test_scroll_display_right() {
        let (mut screen, mock) = create_test_screen();
        
        screen.scroll_display_right().unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x1C));
    }

    #[test]
    fn test_cursor_left() {
        let (mut screen, mock) = create_test_screen();
        
        screen.cursor_left().unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x10));
    }

    #[test]
    fn test_cursor_right() {
        let (mut screen, mock) = create_test_screen();
        
        screen.cursor_right().unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x14));
    }

    #[test]
    fn test_autoscroll_on() {
        let (mut screen, mock) = create_test_screen();
        
        screen.autoscroll_on().unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x07));
    }

    #[test]
    fn test_autoscroll_off() {
        let (mut screen, mock) = create_test_screen();
        
        screen.autoscroll_off().unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x06));
    }

    #[test]
    fn test_left_to_right() {
        let (mut screen, mock) = create_test_screen();
        
        screen.left_to_right().unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x06));
    }

    #[test]
    fn test_right_to_left() {
        let (mut screen, mock) = create_test_screen();
        
        screen.right_to_left().unwrap();
        
        let commands = mock.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], I2CCommand::WriteByteData(254, 0x04));
    }

    #[test]
    fn test_advanced_display_operations() {
        let (mut screen, mock) = create_test_screen();
        
        screen.clear().unwrap();
        screen.print("Scrolling test").unwrap();
        
        for _ in 0..5 {
            screen.scroll_display_left().unwrap();
        }
        
        let commands = mock.get_commands();
        let scroll_count = commands.iter().filter(|cmd| {
            **cmd == I2CCommand::WriteByteData(254, 0x18)
        }).count();
        assert_eq!(scroll_count, 5);
        
        mock.clear_commands();
        
        for _ in 0..3 {
            screen.scroll_display_right().unwrap();
        }
        
        let commands = mock.get_commands();
        let scroll_count = commands.iter().filter(|cmd| {
            **cmd == I2CCommand::WriteByteData(254, 0x1C)
        }).count();
        assert_eq!(scroll_count, 3);
    }

    #[test]
    fn test_cursor_navigation() {
        let (mut screen, mock) = create_test_screen();
        
        screen.move_cursor(0, 10).unwrap();
        
        for _ in 0..5 {
            screen.cursor_left().unwrap();
        }
        
        for _ in 0..2 {
            screen.cursor_right().unwrap();
        }
        
        let commands = mock.get_commands();
        
        let left_count = commands.iter().filter(|cmd| {
            **cmd == I2CCommand::WriteByteData(254, 0x10)
        }).count();
        assert_eq!(left_count, 5);
        
        let right_count = commands.iter().filter(|cmd| {
            **cmd == I2CCommand::WriteByteData(254, 0x14)
        }).count();
        assert_eq!(right_count, 2);
    }
}