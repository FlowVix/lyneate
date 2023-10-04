use std::collections::BTreeMap;

use colored::Colorize;

use crate::span::MessageSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report<S> {
    pub code: String,
    pub messages: Vec<(S, String)>,
}

impl<S> Report<S>
where
    S: MessageSpan + std::fmt::Debug,
{
    pub fn display<C>(self, colors: C)
    where
        C: Iterator<Item = (u8, u8, u8)>,
    {
        let colors = colors.take(self.messages.len()).collect::<Vec<_>>();
        println!("Goolors {:?}", colors);

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
        struct MsgID(usize);

        #[derive(Debug, Clone, Copy)]
        struct LineInfo<'a> {
            line: &'a str,
            start: usize,
            end: usize,
        }

        let lines = {
            let mut out = vec![];
            let mut size = 0;

            for s in self.code.split_inclusive('\n') {
                out.push(LineInfo {
                    line: s,
                    start: size,
                    end: size + s.len(),
                });
                size += s.len();
            }

            out
        };

        println!("{:#?}", lines);

        let get_line = |c: usize| {
            lines
                .iter()
                .position(|line| line.start <= c && c < line.end)
                .unwrap_or(lines.len() - 1)
        };

        // stage 1

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
                linear.entry(start_line).or_insert(vec![]).push(LinearMsg {
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

        // println!("linear {:#?}", linear_sorted);
        // println!("multiline {:#?}", multiline_groups);
        // println!("space left {}", extra_space_left);

        #[derive(Debug, Clone)]
        struct FinalLine<S> {
            highlights: Vec<(S, MsgID)>,
            spacing: usize,
        }
        impl<S> FinalLine<S> {
            pub fn new() -> Self {
                Self {
                    highlights: vec![],
                    spacing: 0,
                }
            }
        }
        let mut final_lines = linear
            .iter()
            .map(|(l, msgs)| (*l, FinalLine::<S>::new()))
            .collect::<BTreeMap<_, _>>();

        for group in &multiline_groups {
            for msg in &group.msgs {
                {
                    let line = final_lines
                        .entry(msg.start_line)
                        .or_insert(FinalLine::<S>::new());

                    line.highlights.push((
                        S::from_range(msg.pre_len..lines[msg.start_line].line.trim_end().len()),
                        msg.id,
                    ));
                    // line.spacing += 2;
                }
                {
                    let line = final_lines
                        .entry(msg.end_line)
                        .or_insert(FinalLine::<S>::new());

                    line.highlights
                        .push((S::from_range(0..msg.end_len), msg.id));
                    line.spacing += 2;
                }
            }
        }
        for (line, msgs) in linear {
            for msg in msgs {
                let line = final_lines.get_mut(&line).unwrap();

                line.highlights.push((msg.span, msg.id));
                line.spacing += if line.spacing == 0 { 3 } else { 2 };
            }
        }
        let extra_space_left = multiline_groups
            .iter()
            .map(|g| g.msgs.len())
            .max()
            .map(|v| v * 2 + 1)
            .unwrap_or(0);

        let max_line = lines.iter().map(|l| l.line.len()).max().unwrap_or(0) + 4 + extra_space_left;

        #[derive(Debug, Clone, Copy)]
        struct BoardCell {
            id: Option<MsgID>,
            b: u8,
        }
        #[derive(Debug, Clone)]
        struct BoardRow {
            line: Option<usize>,
            cells: Vec<BoardCell>,
        }
        impl BoardRow {
            pub fn recolor<S: MessageSpan>(&mut self, span: S, id: Option<MsgID>) {
                for i in span.start()..span.end() {
                    self.cells[i].id = id
                }
            }
        }

        let mut board: Vec<BoardRow> = vec![];

        for (line, info) in &final_lines {
            let s = lines[*line].line.trim_end();

            board.push(BoardRow {
                line: Some(*line),
                cells: (" ".repeat(extra_space_left)
                    + s
                    + &" ".repeat(max_line - extra_space_left - s.len()))
                    .as_bytes()
                    .iter()
                    .map(|v| BoardCell { id: None, b: *v })
                    .collect::<Vec<_>>(),
            });

            for _ in 0..(info.spacing) {
                board.push(BoardRow {
                    line: None,
                    cells: vec![BoardCell { id: None, b: b' ' }; max_line],
                });
            }
        }

        // struct LineInfo<S> {
        //     line: usize,
        //     msgs: Vec<LinearMsg<S>>,
        // }
        println!("final lines {:#?}\n-------------------", final_lines);
        // println!("max {:#?}", max_line);
        // println!("extra_space_left {:#?}", extra_space_left);

        for row in board {
            println!(
                "{}",
                // line,
                row.cells
                    .iter()
                    .map(|c| {
                        if let Some(id) = c.id {
                            let (r, g, b) = colors[id.0];
                            (c.b as char).to_string().truecolor(r, g, b).to_string()
                        } else {
                            (c.b as char).to_string()
                        }
                    })
                    .collect::<String>()
            )
            // println!("{:?}", i);
        }

        struct DrawInfo {}

        // println!(
        //     "aa{}b",
        //     format!("gge{}azzzzz", format!("br{}h", "lol".bright_green()).bright_red()).bright_green()
        // );
    }
}
