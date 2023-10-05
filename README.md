# Lyneate

Display beautiful code reports in the terminal with
support for single-line and multi-line highlighting.

## Example

_todo_

## Usage

This crate provides the [`Report`] struct which takes the source code and an iterator over the span, text, and color of all messages.

The API is kept simple in order to allow as much flexibility
as possible to the user. It does not make any assumptions or care
about the provenance of the source code.

## Planned Features

-   More customizability of the shape and style of the displayed reports.
-   Colorless support.
