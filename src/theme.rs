use colored::Colorize;

/// Theme defining the characters used different components of the report display.
///
/// ```rust
/// ThemeChars {
///     underline: 'α',
///     underline_junction: 'β',
///     underline_vertical: 'γ',
///     side_vertical: 'δ',
///     side_vertical_dotted: 'ε',
///     side_pointer: 'ζ',
///     side_pointer_line: 'η',
///     side_junction: 'θ',
///     bottom_curve: 'κ',
///     top_curve: 'λ',
///     msg_pointer: 'μ',
///     msg_line: 'ν',
/// }
/// ```
/// <img src="https://github.com/FlowVix/lyneate/blob/master/images/chars.png?raw=true" alt="test"/>
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeChars {
    pub underline: char,
    pub underline_junction: char,
    pub underline_vertical: char,

    pub side_vertical: char,
    pub side_vertical_dotted: char,
    pub side_pointer: char,
    pub side_pointer_line: char,
    pub side_junction: char,

    pub bottom_curve: char,
    pub top_curve: char,

    pub msg_pointer: char,
    pub msg_line: char,
}

/// Theme defining string callbacks applied to different parts of the report display.
///
/// For example, you can use this in conjuction with terminal color crates
/// to make line numbers display with color or other effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeEffects {
    pub line_numbers: fn(&str) -> String,
    pub unhighlighted: fn(&str) -> String,
}

/// Theme defining the different lengths and paddings of the report display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeSizing {
    pub pre_line_number_padding: usize,

    pub underline_spacing: usize,
    pub underline_arm_length: usize,

    pub side_arm_length: usize,
    pub side_pointer_length: usize,
}

/// A collection of the themes to be used when displaying a report.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub chars: ThemeChars,
    pub effects: ThemeEffects,
    pub sizing: ThemeSizing,
}

impl Default for ThemeChars {
    fn default() -> Self {
        Self::box_drawing_chars()
    }
}
impl ThemeChars {
    pub fn box_drawing_chars() -> Self {
        Self {
            underline: '─',
            underline_junction: '┬',
            underline_vertical: '│',
            side_vertical: '│',
            side_vertical_dotted: '╵',
            side_pointer: '▶',
            side_pointer_line: '─',
            side_junction: '├',
            bottom_curve: '╰',
            top_curve: '╭',
            msg_pointer: '─',
            msg_line: '─',
        }
    }
    pub fn ascii() -> Self {
        Self {
            underline: '-',
            underline_junction: '-',
            underline_vertical: '|',
            side_vertical: '|',
            side_vertical_dotted: ':',
            side_pointer: '>',
            side_pointer_line: '-',
            side_junction: '|',
            bottom_curve: '\\',
            top_curve: '/',
            msg_pointer: '-',
            msg_line: '-',
        }
    }
}

impl Default for ThemeEffects {
    fn default() -> Self {
        Self {
            line_numbers: |s| s.dimmed().to_string(),
            unhighlighted: |s| s.to_string(),
        }
        // Self::box_drawing_chars()
    }
}
impl ThemeEffects {
    pub fn none() -> Self {
        Self {
            line_numbers: |s| s.to_string(),
            unhighlighted: |s| s.to_string(),
        }
    }
}

impl Default for ThemeSizing {
    fn default() -> Self {
        Self {
            pre_line_number_padding: 3,
            underline_spacing: 1,
            underline_arm_length: 2,
            side_arm_length: 2,
            side_pointer_length: 2,
        }
    }
}
