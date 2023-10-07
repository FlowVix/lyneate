# Lyneate

[![crates.io](https://img.shields.io/crates/v/lyneate)](https://crates.io/crates/lyneate)
[![license](https://img.shields.io/crates/l/lyneate)](https://github.com/FlowVix/lyneate/blob/master/LICENSE)
[![docs.rs](https://img.shields.io/docsrs/lyneate)](https://docs.rs/lyneate)

Display beautiful code reports in the terminal with
support for single-line and multi-line highlighting.

## [Example](https://github.com/FlowVix/lyneate/blob/master/examples/basic.rs)

<img src="https://github.com/FlowVix/lyneate/blob/master/images/example.png?raw=true" alt="test"/>

```rust
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
```

## Usage

This crate provides the `Report` struct which takes the source code and an iterator over the span, text, and color of all messages.

Code spans can be byte-aligned or char-aligned. Different methods for constructing a `Report` for either are provided.

The API is kept simple in order to allow as much flexibility
as possible to the user. It does not make any assumptions or care
about the provenance of the source code.

## Planned Features

-   More customizability of the shape and style of the displayed reports.
-   Colorless support.
