use std::collections::BTreeMap;

use colored::Colorize;
use widestring::{Utf32Str, Utf32String};

use crate::{span::MessageSpan, util::byte_span_to_char_span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report<S> {
    pub code: Utf32String,
    pub messages: Vec<(S, String)>,
}

impl<S> Report<S>
where
    S: MessageSpan + std::fmt::Debug,
{
    pub fn new(code: &str, mut messages: Vec<(S, String)>) -> Self {
        let code_utf32 = Utf32String::from_str(code);

        for (s, _) in &mut messages {
            *s = byte_span_to_char_span(code, *s);
        }

        Self {
            code: code_utf32,
            messages,
        }
    }

    pub fn display<C>(mut self, colors: C)
    where
        C: Iterator<Item = (u8, u8, u8)>,
    {
        let colors = colors.take(self.messages.len()).collect::<Vec<_>>();

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        struct MsgID(usize);

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
        struct LinearMsg<S: std::fmt::Debug> {
            id: MsgID,
            span: S,
            msg: String,
        }
        #[derive(Debug, Clone)]
        struct MultilineMsg {
            id: MsgID,

            start_line: usize,
            end_line: usize,

            pre_len: usize,
            end_len: usize,

            msg: String,
        }

        let mut linear: BTreeMap<usize, Vec<LinearMsg<S>>> = BTreeMap::new();
        let mut multiline: Vec<MultilineMsg> = vec![];

        for (id, (span, msg)) in self.messages.into_iter().enumerate() {
            let start_line = get_line(span.start());
            let end_line = get_line(span.end());

            if start_line == end_line {
                linear.entry(start_line).or_default().push(LinearMsg {
                    id: MsgID(id),
                    span: span.sub(lines[start_line].start),
                    msg,
                })
            } else {
                multiline.push(MultilineMsg {
                    id: MsgID(id),
                    start_line,
                    end_line,
                    pre_len: span.start() - lines[start_line].start,
                    end_len: span.end() - lines[end_line].start,
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
            underline_highlights: Vec<(S, MsgID)>,
            multiline_highlights: Vec<(S, MsgID)>,
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
            .map(|l| (*l, FinalLine::<S>::new()))
            .collect::<BTreeMap<_, _>>();

        #[derive(Debug, Clone)]
        struct UnderlineCommand<S> {
            line: usize,
            span: S,
            msg: String,
            id: MsgID,
            depth: usize,
            connector_pos: usize,
        }
        #[derive(Debug, Clone)]
        struct MultilineCommand {
            start_line: usize,
            end_line: usize,

            msg: String,

            id: MsgID,

            depth: usize,
            side_height: usize,
        }

        let mut underline_commands: Vec<UnderlineCommand<S>> = vec![];
        let mut multiline_commands: Vec<MultilineCommand> = vec![];

        let side_space = multiline_groups
            .iter()
            .map(|g| g.msgs.len())
            .max()
            .map(|v| v * 2 + 1)
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

                fline.underline_highlights.push((msg.span, msg.id));
                fline.spacing += if fline.spacing == 0 { 3 } else { 2 };

                let middle = msg.span.start() + msg.span.len() / 2;
                let connector_pos = 'outer: {
                    let mut max_span = None;
                    for span in spans {
                        let diff = if (span.start()..span.end()).contains(&middle) {
                            break 'outer span.start() + span.len() / 2;
                        } else if span.end() <= middle {
                            middle - span.end()
                        } else {
                            span.start() - middle - 1
                        };
                        if max_span.is_none() || max_span.is_some_and(|(_, v)| diff < v) {
                            max_span = Some((span, diff))
                        }
                    }
                    max_span
                        .map(|(s, _)| s.start() + s.len() / 2)
                        .unwrap_or(middle)
                };

                underline_commands.push(UnderlineCommand {
                    line,
                    span: msg.span,
                    msg: msg.msg,
                    id: msg.id,
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
                        .or_insert(FinalLine::<S>::new());

                    line.multiline_highlights.push((
                        S::from_range(msg.pre_len..lines[msg.start_line].line.trim_end().len()),
                        msg.id,
                    ));
                }
                let depth = {
                    let line = final_lines
                        .entry(msg.end_line)
                        .or_insert(FinalLine::<S>::new());

                    line.multiline_highlights
                        .push((S::from_range(0..msg.end_len), msg.id));
                    line.spacing += 2;
                    line.spacing
                };
                multiline_commands.push(MultilineCommand {
                    start_line: msg.start_line,
                    end_line: msg.end_line,
                    msg: msg.msg,
                    id: msg.id,
                    depth: depth - 1,
                    side_height: side,
                })
            }
        }

        let max_line = lines.iter().map(|l| l.line.len()).max().unwrap_or(0) + 4 + side_space;

        #[derive(Debug, Clone, Copy)]
        struct BoardCell {
            id: Option<MsgID>,
            ch: char,
        }
        #[derive(Debug, Clone)]
        struct BoardRow {
            line: Option<usize>,
            cells: Vec<BoardCell>,
            end_str: Option<String>,
        }
        impl BoardRow {
            pub fn recolor<S: MessageSpan>(&mut self, span: S, id: Option<MsgID>) {
                for i in span.start()..span.end() {
                    if let Some(c) = self.cells.get_mut(i) {
                        c.id = id;
                    }
                }
            }
            pub fn write(&mut self, text: &str, start: usize) {
                for (i, ch) in text.chars().enumerate() {
                    if let Some(c) = self.cells.get_mut(i + start) {
                        c.ch = ch;
                    }
                }
            }
            pub fn write_colored(&mut self, text: &str, start: usize, id: Option<MsgID>) {
                for (i, ch) in text.chars().enumerate() {
                    if let Some(c) = self.cells.get_mut(i + start) {
                        c.ch = ch;
                        c.id = id;
                    }
                }
            }
        }

        let mut board: Vec<BoardRow> = vec![];

        for (line, info) in &final_lines {
            let s = lines[*line].line.trim_end();

            board.push(BoardRow {
                line: Some(*line),
                cells: (Utf32String::from(" ").repeat(side_space)
                    + s
                    + Utf32String::from(" ")
                        .repeat(max_line - side_space - s.len())
                        .as_utfstr())
                .chars()
                .map(|v| BoardCell { id: None, ch: v })
                .collect::<Vec<_>>(),
                end_str: None,
            });

            for _ in 0..(info.spacing) {
                board.push(BoardRow {
                    line: None,
                    cells: vec![BoardCell { id: None, ch: ' ' }; max_line],
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
            for &(span, id) in info
                .multiline_highlights
                .iter()
                .chain(&info.underline_highlights)
            {
                board[shifted_line(*line)].recolor(span.plus(side_space), Some(id));
            }
        }

        for MultilineCommand {
            start_line,
            end_line,
            msg,
            id,
            depth,
            side_height,
        } in multiline_commands
        {
            let horiz = side_space - side_height * 2 - 3;
            let start_line = shifted_line(start_line);
            let end_line = shifted_line(end_line);

            #[allow(clippy::needless_range_loop)]
            for i in (start_line + 1)..end_line {
                let spacing = board[i].line.is_none();
                board[i].write_colored(if spacing { "┆" } else { "│" }, horiz, Some(id));
            }

            board[start_line].write_colored("╭▶", horiz, Some(id));
            board[end_line].write_colored("├▶", horiz, Some(id));

            for i in 0..depth {
                board[end_line + i + 1].write_colored("│", horiz, Some(id))
            }
            {
                let line = &mut board[end_line + depth + 1];
                line.write_colored("╰──", horiz, Some(id));
                line.cells.truncate(horiz + 3);
                line.end_str = Some(msg)
            }
        }

        for UnderlineCommand {
            line,
            span,
            msg,
            id,
            depth,
            connector_pos,
        } in underline_commands
        {
            let line = shifted_line(line) + 1;
            board[line].write_colored(&"─".repeat(span.len()), span.start() + side_space, Some(id));
            board[line].write("┬", connector_pos + side_space);
            for i in 0..(depth - 1) {
                board[line + i + 1].write_colored("│", connector_pos + side_space, Some(id))
            }
            let arm = connector_pos + side_space;
            {
                let line = &mut board[line + depth];
                line.write_colored("╰──", arm, Some(id));
                line.cells.truncate(arm + 3);
                line.end_str = Some(msg)
            }
        }

        let max_line_num_len = (final_lines.last_key_value().unwrap().0 + 1).ilog10() as usize + 1;
        let empty_pad = " ".repeat(max_line_num_len + 3);

        for row in board {
            println!(
                "   {}{} {}",
                row.line
                    .map(|v| format!("{:>max_line_num_len$}.  ", v + 1)
                        .dimmed()
                        .to_string())
                    .unwrap_or(empty_pad.clone()),
                row.cells
                    .iter()
                    .map(|c| {
                        if let Some(id) = c.id {
                            let (r, g, b) = colors[id.0];
                            c.ch.to_string().truecolor(r, g, b).to_string()
                        } else {
                            c.ch.to_string()
                        }
                    })
                    .collect::<String>(),
                row.end_str.unwrap_or("".into()),
            )
        }
    }
}
