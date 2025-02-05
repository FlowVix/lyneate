use colored::Colorize;
use lyneate::{Report, Theme, ThemeChars, ThemeSizing};

fn main() {
    println!(
        "{} Mismatched match expression branch return types\n",
        "Error:".bright_red()
    );

    let report = Report::new_char_spanned(
        include_str!("basic.pseudo"),
        [
            (
                23..91,
                format!("{}", "In this match expression".dimmed()),
                (255, 64, 112),
            ),
            (
                56..67,
                format!("{} {}", "This is of type".dimmed(), "int".bright_white()),
                (255, 159, 64),
            ),
            (
                78..83,
                format!("{} {}", "This is of type".dimmed(), "string".bright_white()),
                (207, 255, 64),
            ),
        ],
    )
    .with_theme(Theme {
        sizing: ThemeSizing {
            pre_line_number_padding: 5,
            underline_arm_length: 10,
            ..Default::default()
        },
        chars: ThemeChars::ascii(),
        ..Default::default()
    });

    report.display();
}
