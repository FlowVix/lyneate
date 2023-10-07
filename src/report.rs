use std::{collections::BTreeMap, ops::Range};

use colored::Colorize;
use widestring::{Utf32Str, Utf32String};

use crate::{
    span::{byte_span_to_char_span, MessageSpan},
    Theme,
};

type Color = (u8, u8, u8);

/// A code report containing the source code in UTF32 and the spans,
/// text, and colors of all messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report<'a, I> {
    code: Utf32String,
    messages: I,
    realign: Option<&'a str>,
    pub theme: Theme,
}

impl<'a, I> Report<'a, I>
where
    I: IntoIterator<Item = (Range<usize>, String, Color)>,
{
    /// Creates a new report from source code and messages with byte-aligned spans.
    pub fn new_byte_spanned(code: &'a str, messages: I) -> Self {
        let code_utf32 = Utf32String::from_str(code);

        Self {
            code: code_utf32,
            messages,
            realign: Some(code),
            theme: Theme::default(),
        }
    }
    /// Creates a new report from source code and messages with char-aligned spans.
    pub fn new_char_spanned(code: &str, messages: I) -> Self {
        let code_utf32 = Utf32String::from_str(code);

        Self {
            code: code_utf32,
            messages,
            realign: None,
            theme: Theme::default(),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Prettily displays the code report.
    pub fn display(self) {
        #[derive(Debug, Clone, Copy)]
        struct LineInfo<'a> {
            line: &'a Utf32Str,
            start: usize,
            end: usize,
        }

        let lines = {
            let mut out = vec![];

            let mut start = 0;

            macro_rules! s {
                ($s:expr) => {
                    #[allow(unused_assignments)]
                    {
                        out.push(LineInfo {
                            line: $s,
                            start,
                            end: start + $s.len(),
                        });
                        start += $s.len();
                    }
                };
            }

            for (i, c) in self.code.as_char_slice().iter().enumerate() {
                if *c == '\n' {
                    s!(&self.code[start..(i + 1)]);
                }
            }
            s!(&self.code[start..self.code.len()]);

            out
        };

        let get_line = |c: usize| {
            lines
                .iter()
                .position(|line| line.start <= c && c < line.end)
                .unwrap_or(lines.len() - 1)
        };

        #[derive(Debug, Clone)]
        struct LinearMsg {
            color: Color,
            span: MessageSpan,
            msg: String,
        }
        #[derive(Debug, Clone)]
        struct MultilineMsg {
            color: Color,

            start_line: usize,
            end_line: usize,

            pre_len: usize,
            end_len: usize,

            msg: String,
        }

        let mut linear: BTreeMap<usize, Vec<LinearMsg>> = BTreeMap::new();
        let mut multiline: Vec<MultilineMsg> = vec![];

        for (span, msg, color) in self.messages {
            let span = MessageSpan {
                start: span.start,
                end: span.end,
            };
            let span = if let Some(code) = self.realign {
                byte_span_to_char_span(code, span)
            } else {
                span
            };

            let start_line = get_line(span.start);
            let end_line = get_line(span.end);

            if start_line == end_line {
                linear.entry(start_line).or_default().push(LinearMsg {
                    color,
                    span: span.sub(lines[start_line].start),
                    msg,
                })
            } else {
                multiline.push(MultilineMsg {
                    color,
                    start_line,
                    end_line,
                    pre_len: span.start - lines[start_line].start,
                    end_len: span.end - lines[end_line].start,
                    msg,
                })
            }
        }

        #[derive(Debug, Clone)]
        struct MultilineGroup {
            first_line: usize,
            last_line: usize,
            msgs: Vec<MultilineMsg>,
        }

        let mut multiline_groups: Vec<MultilineGroup> = vec![];

        'outer: for msg in multiline {
            for group in &mut multiline_groups {
                if group.first_line <= msg.end_line && msg.start_line <= group.last_line {
                    if msg.start_line < group.first_line {
                        group.first_line = msg.start_line;
                    }
                    if msg.end_line > group.last_line {
                        group.last_line = msg.end_line;
                    }
                    group.msgs.push(msg);
                    continue 'outer;
                }
            }
            multiline_groups.push(MultilineGroup {
                first_line: msg.start_line,
                last_line: msg.end_line,
                msgs: vec![msg],
            })
        }

        #[derive(Debug, Clone)]
        struct FinalLine<S> {
            underline_highlights: Vec<(S, Color)>,
            multiline_highlights: Vec<(S, Color)>,
            spacing: usize,
        }
        impl<S> FinalLine<S> {
            pub fn new() -> Self {
                Self {
                    underline_highlights: vec![],
                    multiline_highlights: vec![],
                    spacing: 0,
                }
            }
        }
        let mut final_lines = linear
            .keys()
            .map(|l| (*l, FinalLine::new()))
            .collect::<BTreeMap<_, _>>();

        #[derive(Debug, Clone)]
        struct UnderlineCommand {
            line: usize,
            span: MessageSpan,
            msg: String,
            color: Color,
            depth: usize,
            connector_pos: usize,
        }
        #[derive(Debug, Clone)]
        struct MultilineCommand {
            start_line: usize,
            end_line: usize,

            spacing_end: usize,

            msg: String,

            color: Color,

            depth: usize,
            side_height: usize,
        }

        let mut underline_commands: Vec<UnderlineCommand> = vec![];
        let mut multiline_commands: Vec<MultilineCommand> = vec![];

        let side_space = multiline_groups
            .iter()
            .map(|g| g.msgs.len())
            .max()
            .map(|v| v * (self.theme.sizing.side_pointer_length + 1) + 1)
            .unwrap_or(0);

        for (line, msgs) in linear {
            let mut visible_spans = msgs.iter().map(|l| vec![l.span]).collect::<Vec<_>>();

            for i in 0..(visible_spans.len() - 1) {
                for j in (i + 1)..visible_spans.len() {
                    visible_spans[i] = visible_spans[i]
                        .iter()
                        .flat_map(|s| s.overlay(visible_spans[j][0]))
                        .collect();
                }
            }

            for (msg, spans) in msgs.into_iter().zip(visible_spans) {
                let fline = final_lines.get_mut(&line).unwrap();

                fline.underline_highlights.push((msg.span, msg.color));
                fline.spacing +=
                    if fline.spacing == 0 { 2 } else { 1 } + self.theme.sizing.underline_spacing;

                let middle = msg.span.start + msg.span.size() / 2;
                let connector_pos = 'outer: {
                    let mut max_span = None;
                    for span in spans {
                        let diff = if (span.start..span.end).contains(&middle) {
                            break 'outer span.start + span.size() / 2;
                        } else if span.end <= middle {
                            middle - span.end
                        } else {
                            span.start - middle - 1
                        };
                        if max_span.is_none() || max_span.is_some_and(|(_, v)| diff < v) {
                            max_span = Some((span, diff))
                        }
                    }
                    max_span
                        .map(|(s, _)| s.start + s.size() / 2)
                        .unwrap_or(middle)
                };

                underline_commands.push(UnderlineCommand {
                    line,
                    span: msg.span,
                    msg: msg.msg,
                    color: msg.color,
                    depth: fline.spacing - 1,
                    connector_pos,
                })
            }
        }
        for group in multiline_groups {
            for (side, msg) in group.msgs.into_iter().enumerate() {
                {
                    let line = final_lines
                        .entry(msg.start_line)
                        .or_insert(FinalLine::new());

                    line.multiline_highlights.push((
                        MessageSpan {
                            start: msg.pre_len,
                            end: lines[msg.start_line].line.trim_end().len(),
                        },
                        msg.color,
                    ));
                }
                let (spacing_end, depth) = {
                    let hl_line = final_lines.entry(msg.end_line).or_insert(FinalLine::new());

                    hl_line.multiline_highlights.push((
                        MessageSpan {
                            start: 0,
                            end: msg.end_len,
                        },
                        msg.color,
                    ));

                    let spacing_end = msg.end_line.max(group.last_line);

                    let spacing_line = final_lines.entry(spacing_end).or_insert(FinalLine::new());

                    spacing_line.spacing += 2;
                    (spacing_end, spacing_line.spacing)
                };

                multiline_commands.push(MultilineCommand {
                    start_line: msg.start_line,
                    end_line: msg.end_line,
                    msg: msg.msg,
                    color: msg.color,
                    depth,
                    side_height: side,
                    spacing_end,
                })
            }
        }

        {
            let keys = final_lines.keys().copied().collect::<Vec<_>>();
            for idx in keys {
                if final_lines[&idx].spacing == 0 && !final_lines.contains_key(&(idx + 1)) {
                    final_lines.get_mut(&idx).unwrap().spacing += 1;
                }
            }
        }

        #[derive(Debug, Clone, Copy)]
        struct BoardCell {
            color: Option<Color>,
            ch: char,
        }
        #[derive(Debug, Clone)]
        struct BoardRow {
            line: Option<usize>,
            cells: Vec<BoardCell>,
            end_str: Option<String>,
        }
        impl BoardRow {
            pub fn recolor(&mut self, span: MessageSpan, color: Option<Color>) {
                for i in span.start..span.end {
                    if let Some(c) = self.get_cell(i) {
                        c.color = color;
                    }
                }
            }
            pub fn _write(&mut self, text: &str, start: usize) {
                for (i, ch) in text.chars().enumerate() {
                    self.write_char(ch, i + start);
                }
            }
            pub fn write_colored(&mut self, text: &str, start: usize, color: Option<Color>) {
                for (i, ch) in text.chars().enumerate() {
                    self.write_colored_char(ch, i + start, color);
                }
            }
            pub fn write_char(&mut self, ch: char, idx: usize) {
                if let Some(c) = self.get_cell(idx) {
                    c.ch = ch;
                }
            }
            pub fn write_colored_char(&mut self, ch: char, idx: usize, color: Option<Color>) {
                if let Some(c) = self.get_cell(idx) {
                    c.ch = ch;
                    c.color = color;
                }
            }
            pub fn get_cell(&mut self, idx: usize) -> Option<&mut BoardCell> {
                if self.end_str.is_some() {
                    return self.cells.get_mut(idx);
                }

                if idx >= self.cells.len() {
                    self.cells.resize(
                        idx + 1,
                        BoardCell {
                            color: None,
                            ch: ' ',
                        },
                    );
                }

                self.cells.get_mut(idx)
            }
        }

        let mut board: Vec<BoardRow> = vec![];

        for (line, info) in &final_lines {
            let s = lines[*line].line.trim_end();

            board.push(BoardRow {
                line: Some(*line),
                cells: (Utf32String::from(" ").repeat(side_space) + s)
                    .chars()
                    .map(|v| BoardCell { color: None, ch: v })
                    .collect::<Vec<_>>(),
                end_str: None,
            });

            for _ in 0..(info.spacing) {
                board.push(BoardRow {
                    line: None,
                    cells: vec![],
                    end_str: None,
                });
            }
        }

        let shifted_line = |l: usize| {
            final_lines
                .iter()
                .take_while(|(v, _)| **v != l)
                .map(|(_, l)| l.spacing + 1)
                .sum::<usize>()
        };

        for (line, info) in &final_lines {
            for &(span, color) in info
                .multiline_highlights
                .iter()
                .chain(&info.underline_highlights)
            {
                board[shifted_line(*line)].recolor(span.plus(side_space), Some(color));
            }
        }

        for MultilineCommand {
            start_line,
            end_line,
            spacing_end,
            msg,
            color,
            depth,
            side_height,
        } in multiline_commands
        {
            let horiz = side_space
                - side_height * (self.theme.sizing.side_pointer_length + 1)
                - 2
                - self.theme.sizing.side_pointer_length;
            let start_line = shifted_line(start_line);
            let end_line = shifted_line(end_line);
            let spacing_end = shifted_line(spacing_end);

            #[allow(clippy::needless_range_loop)]
            for i in (start_line + 1)..end_line {
                let spacing = board[i].line.is_none();
                board[i].write_colored_char(
                    if spacing {
                        self.theme.chars.side_vertical_dotted
                    } else {
                        self.theme.chars.side_vertical
                    },
                    horiz,
                    Some(color),
                );
            }

            {
                let arm = match self.theme.sizing.side_pointer_length {
                    0 => "".into(),
                    _ => format!(
                        "{}{}",
                        self.theme
                            .chars
                            .side_pointer_line
                            .to_string()
                            .repeat(self.theme.sizing.side_pointer_length - 1),
                        self.theme.chars.side_pointer
                    ),
                };

                board[start_line].write_colored(
                    &format!("{}{}", self.theme.chars.top_curve, arm),
                    horiz,
                    Some(color),
                );
                board[end_line].write_colored(
                    &format!("{}{}", self.theme.chars.side_junction, arm),
                    horiz,
                    Some(color),
                );
            }

            #[allow(clippy::needless_range_loop)]
            for i in (end_line + 1)..(spacing_end + depth) {
                board[i].write_colored_char(self.theme.chars.side_vertical, horiz, Some(color))
            }
            {
                let line = &mut board[spacing_end + depth];

                let arm = match self.theme.sizing.side_arm_length {
                    0 => "".into(),
                    _ => format!(
                        "{}{}",
                        self.theme
                            .chars
                            .msg_line
                            .to_string()
                            .repeat(self.theme.sizing.side_arm_length - 1),
                        self.theme.chars.msg_pointer
                    ),
                };

                line.write_colored(
                    &format!("{}{}", self.theme.chars.bottom_curve, arm),
                    horiz,
                    Some(color),
                );
                line.cells
                    .truncate(horiz + self.theme.sizing.side_arm_length + 1);
                line.end_str = Some(msg)
            }
        }

        for UnderlineCommand {
            line,
            span,
            msg,
            color,
            depth,
            connector_pos,
        } in underline_commands
        {
            let line = shifted_line(line) + 1;
            board[line].write_colored(
                &self.theme.chars.underline.to_string().repeat(span.size()),
                span.start + side_space,
                Some(color),
            );
            board[line].write_char(
                self.theme.chars.underline_junction,
                connector_pos + side_space,
            );
            for i in 0..(depth - 1) {
                board[line + i + 1].write_colored_char(
                    self.theme.chars.underline_vertical,
                    connector_pos + side_space,
                    Some(color),
                )
            }
            let arm_start = connector_pos + side_space;
            {
                let line = &mut board[line + depth];

                let arm = match self.theme.sizing.underline_arm_length {
                    0 => "".into(),
                    _ => format!(
                        "{}{}",
                        self.theme
                            .chars
                            .msg_line
                            .to_string()
                            .repeat(self.theme.sizing.underline_arm_length - 1),
                        self.theme.chars.msg_pointer
                    ),
                };

                line.write_colored(
                    &format!("{}{}", self.theme.chars.bottom_curve, arm),
                    arm_start,
                    Some(color),
                );
                line.cells
                    .truncate(arm_start + self.theme.sizing.underline_arm_length + 1);
                line.end_str = Some(msg)
            }
        }

        let max_line_num_len = (final_lines.last_key_value().unwrap().0 + 1).ilog10() as usize + 1;
        let empty_pad = format!("{} ", " ".repeat(max_line_num_len));

        let pre_pad = " ".repeat(self.theme.sizing.pre_line_number_padding);

        for row in board {
            println!(
                "{}{}  {} {}",
                pre_pad,
                row.line
                    .map(|v| (self.theme.effects.line_numbers)(&format!(
                        "{:>max_line_num_len$}.",
                        v + 1
                    )))
                    .unwrap_or((self.theme.effects.line_numbers)(&empty_pad)),
                row.cells
                    .iter()
                    .map(|c| {
                        if let Some((r, g, b)) = c.color {
                            c.ch.to_string().truecolor(r, g, b).to_string()
                        } else {
                            (self.theme.effects.unhighlighted)(&c.ch.to_string())
                        }
                    })
                    .collect::<String>(),
                row.end_str.unwrap_or("".into()),
            )
        }
    }
}
