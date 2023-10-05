use colored::Colorize;
use lyneate::Report;

fn main() {
    println!(
        "{} Mismatched match expression branch return types\n",
        "Error:".bright_red()
    );

    let report = Report::new_char_spanned(
        include_str!("basic.pseudo"),
        [
            (
                29..102,
                format!("{}", "In this match expression".dimmed()),
                (255, 64, 112),
            ),
            (
                64..75,
                format!("{} {}", "This is of type".dimmed(), "int".bright_white()),
                (255, 159, 64),
            ),
            (
                87..92,
                format!("{} {}", "This is of type".dimmed(), "string".bright_white()),
                (207, 255, 64),
            ),
            // (13..84, format!("{}", "agaga".dimmed()), (255, 0, 255)),
        ],
    );

    report.display();
}
