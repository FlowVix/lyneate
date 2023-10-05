use colored::Colorize;
use lyneate::report::Report;

fn main() {
    println!(
        "{} Mismatched match expression branch return types\n",
        "Error:".bright_red()
    );

    let report = Report::new_char_spanned(
        include_str!("basic.pseudo"),
        [
            (
                22..104,
                format!("{}", "In this match expression".dimmed()),
                (255, 64, 112),
            ),
            (
                52..58,
                format!("{} {}", "This is of type".dimmed(), "int".bright_white()),
                (255, 159, 64),
            ),
            (
                95..100,
                format!("{} {}", "This is of type".dimmed(), "string".bright_white()),
                (207, 255, 64),
            ),
        ],
    );

    report.display();
}
