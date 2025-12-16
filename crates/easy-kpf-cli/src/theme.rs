use ratatui::style::{Color, Modifier, Style};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
  Light,
  Dark,
}

impl ThemeMode {
  /// Detect system theme on macOS using AppleScript
  pub fn detect() -> Self {
    if cfg!(target_os = "macos") {
      let output = Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output();

      match output {
        Ok(out) => {
          let stdout = String::from_utf8_lossy(&out.stdout);
          if stdout.trim().eq_ignore_ascii_case("dark") {
            ThemeMode::Dark
          } else {
            ThemeMode::Light
          }
        }
        Err(_) => ThemeMode::Dark, // Default to dark if detection fails
      }
    } else {
      // Default to dark on non-macOS systems
      ThemeMode::Dark
    }
  }
}

/// Apple-inspired color palette for terminal UI
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Theme {
  // Backgrounds
  pub bg_primary: Color,
  pub bg_secondary: Color,
  pub bg_elevated: Color,
  pub bg_selection: Color,
  pub bg_selection_cursor: Color,

  // Text
  pub text_primary: Color,
  pub text_secondary: Color,
  pub text_tertiary: Color,
  pub text_inverted: Color,

  // Borders
  pub border_default: Color,
  pub border_focused: Color,
  pub border_active: Color,

  // Semantic colors
  pub accent: Color,
  pub accent_secondary: Color,
  pub success: Color,
  pub warning: Color,
  pub error: Color,
  pub info: Color,

  // Status bar
  pub status_bg: Color,
  pub status_key_bg: Color,
  pub status_key_fg: Color,

  // Visual mode (purple/violet tones)
  pub visual_border: Color,
  pub visual_bg: Color,
  pub visual_cursor_bg: Color,
}

#[allow(dead_code)]
impl Theme {
  pub fn dark() -> Self {
    Self {
      // Dark backgrounds (macOS dark mode inspired)
      bg_primary: Color::Reset,                    // Terminal default
      bg_secondary: Color::Rgb(44, 44, 46),        // Elevated surfaces
      bg_elevated: Color::Rgb(58, 58, 60),         // Modal backgrounds
      bg_selection: Color::Rgb(50, 50, 55),        // Selection highlight
      bg_selection_cursor: Color::Rgb(72, 72, 74), // Cursor row

      // Text colors
      text_primary: Color::Rgb(245, 245, 247), // Primary text
      text_secondary: Color::Rgb(152, 152, 157), // Secondary text
      text_tertiary: Color::Rgb(99, 99, 102),  // Hints, placeholders
      text_inverted: Color::Rgb(28, 28, 30),   // Text on accent bg

      // Borders
      border_default: Color::Rgb(72, 72, 74), // Inactive borders
      border_focused: Color::Rgb(10, 132, 255), // Focused - iOS blue
      border_active: Color::Rgb(48, 209, 88), // Active/editing - iOS green

      // Semantic colors (iOS/macOS dark mode colors)
      accent: Color::Rgb(10, 132, 255),          // iOS blue
      accent_secondary: Color::Rgb(94, 92, 230), // iOS indigo
      success: Color::Rgb(48, 209, 88),          // iOS green
      warning: Color::Rgb(255, 214, 10),         // iOS yellow
      error: Color::Rgb(255, 69, 58),            // iOS red
      info: Color::Rgb(100, 210, 255),           // iOS cyan

      // Status bar
      status_bg: Color::Rgb(28, 28, 30),
      status_key_bg: Color::Rgb(10, 132, 255),
      status_key_fg: Color::Rgb(255, 255, 255),

      // Visual mode (purple/violet)
      visual_border: Color::Rgb(191, 90, 242), // iOS purple
      visual_bg: Color::Rgb(60, 40, 80),       // Subtle purple bg
      visual_cursor_bg: Color::Rgb(100, 60, 120), // Darker purple cursor
    }
  }

  pub fn light() -> Self {
    Self {
      // Light backgrounds (macOS light mode inspired)
      bg_primary: Color::Reset,                       // Terminal default
      bg_secondary: Color::Rgb(242, 242, 247),        // Elevated surfaces
      bg_elevated: Color::Rgb(255, 255, 255),         // Modal backgrounds
      bg_selection: Color::Rgb(220, 220, 225),        // Selection highlight
      bg_selection_cursor: Color::Rgb(200, 200, 210), // Cursor row

      // Text colors
      text_primary: Color::Rgb(29, 29, 31),     // Primary text
      text_secondary: Color::Rgb(99, 99, 102),  // Secondary text
      text_tertiary: Color::Rgb(142, 142, 147), // Hints, placeholders
      text_inverted: Color::Rgb(255, 255, 255), // Text on accent bg

      // Borders
      border_default: Color::Rgb(199, 199, 204), // Inactive borders
      border_focused: Color::Rgb(0, 122, 255),   // Focused - iOS blue (light)
      border_active: Color::Rgb(52, 199, 89),    // Active/editing - iOS green (light)

      // Semantic colors (iOS/macOS light mode colors)
      accent: Color::Rgb(0, 122, 255),           // iOS blue (light)
      accent_secondary: Color::Rgb(88, 86, 214), // iOS indigo (light)
      success: Color::Rgb(52, 199, 89),          // iOS green (light)
      warning: Color::Rgb(255, 204, 0),          // iOS yellow (light)
      error: Color::Rgb(255, 59, 48),            // iOS red (light)
      info: Color::Rgb(50, 173, 230),            // iOS cyan (light)

      // Status bar
      status_bg: Color::Rgb(229, 229, 234),
      status_key_bg: Color::Rgb(0, 122, 255),
      status_key_fg: Color::Rgb(255, 255, 255),

      // Visual mode (purple/violet)
      visual_border: Color::Rgb(175, 82, 222), // iOS purple (light)
      visual_bg: Color::Rgb(230, 215, 245),    // Subtle purple bg
      visual_cursor_bg: Color::Rgb(210, 190, 230), // Lighter purple cursor
    }
  }

  pub fn for_mode(mode: ThemeMode) -> Self {
    match mode {
      ThemeMode::Light => Self::light(),
      ThemeMode::Dark => Self::dark(),
    }
  }

  // Common style builders
  pub fn title_style(&self) -> Style {
    Style::default()
      .fg(self.accent)
      .add_modifier(Modifier::BOLD)
  }

  pub fn text(&self) -> Style {
    Style::default().fg(self.text_primary)
  }

  pub fn text_secondary(&self) -> Style {
    Style::default().fg(self.text_secondary)
  }

  pub fn text_tertiary(&self) -> Style {
    Style::default().fg(self.text_tertiary)
  }

  pub fn border(&self) -> Style {
    Style::default().fg(self.border_default)
  }

  pub fn border_focused(&self) -> Style {
    Style::default().fg(self.border_focused)
  }

  pub fn border_active(&self) -> Style {
    Style::default().fg(self.border_active)
  }

  pub fn border_visual(&self) -> Style {
    Style::default().fg(self.visual_border)
  }

  pub fn success(&self) -> Style {
    Style::default().fg(self.success)
  }

  pub fn warning(&self) -> Style {
    Style::default().fg(self.warning)
  }

  pub fn error(&self) -> Style {
    Style::default().fg(self.error)
  }

  pub fn accent(&self) -> Style {
    Style::default().fg(self.accent)
  }

  pub fn status_key(&self) -> Style {
    Style::default()
      .fg(self.status_key_fg)
      .bg(self.status_key_bg)
      .add_modifier(Modifier::BOLD)
  }

  pub fn selection(&self) -> Style {
    Style::default().bg(self.bg_selection)
  }

  pub fn cursor(&self) -> Style {
    Style::default()
      .bg(self.bg_selection_cursor)
      .add_modifier(Modifier::BOLD)
  }

  pub fn visual_selection(&self) -> Style {
    Style::default().bg(self.visual_bg)
  }

  pub fn visual_cursor(&self) -> Style {
    Style::default()
      .bg(self.visual_cursor_bg)
      .add_modifier(Modifier::BOLD)
  }

  pub fn info(&self) -> Style {
    Style::default().fg(self.info)
  }

  pub fn accent_secondary(&self) -> Style {
    Style::default().fg(self.accent_secondary)
  }

  pub fn key_badge(&self, color: Color) -> Style {
    Style::default().fg(self.text_inverted).bg(color)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_theme_detection() {
    // Just ensure it doesn't panic
    let _mode = ThemeMode::detect();
  }

  #[test]
  fn test_theme_for_mode() {
    let dark = Theme::for_mode(ThemeMode::Dark);
    let light = Theme::for_mode(ThemeMode::Light);

    // Dark theme should have darker selection cursor
    assert_ne!(dark.bg_selection_cursor, light.bg_selection_cursor);
  }
}
