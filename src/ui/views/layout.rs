use crate::models::ProcessState;
use crate::theme::*;
use crate::ui::app::{App, Focus, Tab};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

const MAX_LINES: usize = 100;

pub fn render_app<B: ratatui::backend::Backend>(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, app, chunks[0]);
    render_body(f, app, chunks[1]);
    render_input(f, app, chunks[2]);
    render_status(f, app, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    f.render_widget(
        Block::bordered()
            .title(" ckptmini-tui ")
            .title_alignment(Alignment::Center),
        area,
    );

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .margin(1)
        .split(area);

    let tabs = Tabs::new(Tab::get_headers().iter().map(|v| Line::from(*v)))
        .select(match app.tab {
            Tab::Processes => 0,
            Tab::Memory => 1,
            Tab::Checkpoints => 2,
        })
        .style(Style::default().fg(CYAN))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(ACCENT));
    f.render_widget(tabs, chunks[0]);

    let info = if let Some(proc) = app.selected_process() {
        format!(" ● PID: {}  {}", proc.pid, proc.name)
    } else {
        " ○ No process selected ".to_string()
    };
    f.render_widget(
        Paragraph::new(info)
            .style(Style::default().fg(SUCCESS))
            .alignment(Alignment::Right),
        chunks[1],
    );
}

fn render_body(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    match app.tab {
        Tab::Processes => render_process_list(f, app, chunks[0]),
        Tab::Memory => render_memory_list(f, app, chunks[0]),
        Tab::Checkpoints => render_checkpoint_list(f, app, chunks[0]),
    }

    render_output(f, app, chunks[1]);
}

fn render_process_list(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focus == Focus::List {
        ACCENT
    } else {
        ACCENT_DIM
    };
    let block = Block::bordered()
        .title(" ⌘ Processes ")
        .border_style(Style::default().fg(border_color));
    f.render_widget(block, area);

    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    if app.processes.is_empty() {
        f.render_widget(
            Paragraph::new("No processes")
                .centered()
                .style(Style::default().fg(TEXT_MUTED)),
            inner,
        );
        return;
    }

    let visible_height = inner.height.saturating_sub(2) as usize;
    let total_items = app.processes.len();
    let scroll = app.process_scroll.min(total_items.saturating_sub(1));

    let header_style = Style::default().fg(CYAN).add_modifier(Modifier::BOLD);
    let rows: Vec<Row> = app
        .processes
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(i, p)| {
            let is_selected = i == scroll && app.focus == Focus::List;
            let state_str = match p.state {
                ProcessState::Running => "Running",
                ProcessState::Sleeping => "Sleeping",
                ProcessState::Stopped => "Stopped",
                ProcessState::Zombie => "Zombie",
                ProcessState::Unknown => "Unknown",
            };
            let name = if p.name.len() > 24 {
                format!("{}...", &p.name[..21])
            } else {
                p.name.clone()
            };
            let cells = vec![
                Cell::from(format!(" {}", if is_selected { "▶" } else { " " })),
                Cell::from(format!("{:>5}", p.pid)),
                Cell::from(format!("{:<26}", name)),
                Cell::from(format!("{:>10}", format_size(p.memory_total))),
                Cell::from(format!("{:>8}", p.threads)),
                Cell::from(state_str),
            ];
            let style = if is_selected {
                Style::default()
                    .bg(HIGHLIGHT_BG)
                    .fg(ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(TEXT_PRIMARY)
            };
            Row::new(cells).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(6),
            Constraint::Length(27),
            Constraint::Length(11),
            Constraint::Length(9),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec![" ", "PID", "Name", "Memory", "Threads", "State"])
            .style(header_style)
            .bottom_margin(0),
    )
    .block(Block::default())
    .widths([
        Constraint::Length(2),
        Constraint::Length(6),
        Constraint::Length(27),
        Constraint::Length(11),
        Constraint::Length(9),
        Constraint::Length(8),
    ]);
    f.render_widget(table, inner);

    render_scrollbar(f, inner, app.processes.len(), app.process_scroll);
}

fn render_memory_list(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focus == Focus::List {
        ACCENT
    } else {
        ACCENT_DIM
    };
    let title = if let Some(proc) = app.selected_process() {
        format!(" ⌬ Memory: {} ({}) ", proc.name, proc.pid)
    } else {
        " ⌬ Memory Map ".to_string()
    };
    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(border_color));
    f.render_widget(block, area);

    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    if app.memory_regions.is_empty() {
        f.render_widget(
            Paragraph::new("No memory regions\nSelect a process first")
                .centered()
                .style(Style::default().fg(TEXT_MUTED)),
            inner,
        );
        return;
    }

    let visible_height = inner.height.saturating_sub(2) as usize;
    let total_items = app.memory_regions.len();
    let scroll = app.memory_scroll.min(total_items.saturating_sub(1));

    let header_style = Style::default().fg(CYAN).add_modifier(Modifier::BOLD);
    let rows: Vec<Row> = app
        .memory_regions
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(i, r)| {
            let is_selected = i == scroll && app.focus == Focus::List;
            let path = r.path.clone().unwrap_or_else(|| "[anon]".to_string());
            let path = if path.len() > 20 {
                format!("{}...", &path[..17])
            } else {
                path
            };
            let cells = vec![
                Cell::from(format!(" {}", if is_selected { "▶" } else { " " })),
                Cell::from(format!("0x{:016x}", r.start)),
                Cell::from(format!("0x{:016x}", r.end)),
                Cell::from(r.perms.as_string()),
                Cell::from(format!("{:>8}", r.human_size())),
                Cell::from(path),
            ];
            let style = if is_selected {
                Style::default()
                    .bg(HIGHLIGHT_BG)
                    .fg(ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(TEXT_PRIMARY)
            };
            Row::new(cells).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(18),
            Constraint::Length(18),
            Constraint::Length(5),
            Constraint::Length(9),
            Constraint::Length(24),
        ],
    )
    .header(
        Row::new(vec![" ", "Start", "End", "Perms", "Size", "Path"])
            .style(header_style)
            .bottom_margin(0),
    )
    .block(Block::default())
    .widths([
        Constraint::Length(2),
        Constraint::Length(18),
        Constraint::Length(18),
        Constraint::Length(5),
        Constraint::Length(9),
        Constraint::Length(24),
    ]);
    f.render_widget(table, inner);

    render_scrollbar(f, inner, app.memory_regions.len(), app.memory_scroll);
}

fn render_checkpoint_list(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focus == Focus::List {
        ACCENT
    } else {
        ACCENT_DIM
    };
    let block = Block::bordered()
        .title(" ⚡ Checkpoints ")
        .border_style(Style::default().fg(border_color));
    f.render_widget(block, area);

    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    if app.checkpoints.is_empty() {
        f.render_widget(
            Paragraph::new("No checkpoints\nPress 'c' to create one")
                .centered()
                .style(Style::default().fg(TEXT_MUTED)),
            inner,
        );
        return;
    }

    let visible_height = inner.height.saturating_sub(2) as usize;
    let total_items = app.checkpoints.len();
    let scroll = app.checkpoint_scroll.min(total_items.saturating_sub(1));

    let header_style = Style::default().fg(CYAN).add_modifier(Modifier::BOLD);
    let rows: Vec<Row> = app
        .checkpoints
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(i, c)| {
            let is_selected = i == scroll && app.focus == Focus::List;
            let name = c
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let cells = vec![
                Cell::from(format!(" {}", if is_selected { "▶" } else { " " })),
                Cell::from(format!("{:<24}", name)),
                Cell::from(format!("{:>5}", c.pid)),
                Cell::from(format!("{:>12}", c.age_string())),
                Cell::from(format!("{:>6}", c.regions)),
                Cell::from(c.human_size()),
            ];
            let style = if is_selected {
                Style::default()
                    .bg(HIGHLIGHT_BG)
                    .fg(ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(TEXT_PRIMARY)
            };
            Row::new(cells).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(25),
            Constraint::Length(6),
            Constraint::Length(13),
            Constraint::Length(7),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec![" ", "Name", "PID", "Created", "Regions", "Size"])
            .style(header_style)
            .bottom_margin(0),
    )
    .block(Block::default())
    .widths([
        Constraint::Length(2),
        Constraint::Length(25),
        Constraint::Length(6),
        Constraint::Length(13),
        Constraint::Length(7),
        Constraint::Length(10),
    ]);
    f.render_widget(table, inner);

    render_scrollbar(f, inner, app.checkpoints.len(), app.checkpoint_scroll);
}

fn render_output(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.focus == Focus::Output {
        ACCENT
    } else {
        ACCENT_DIM
    };
    let title = if app.show_hex_view {
        if app.is_hex_searching {
            format!(" ◈ Hex Dump [{}] ", app.hex_search)
        } else {
            " ◈ Hex Dump ".to_string()
        }
    } else if matches!(app.tab, Tab::Checkpoints) {
        " ◈ Checkpoint Info ".to_string()
    } else if matches!(app.tab, Tab::Memory) {
        " ◈ Memory Map ".to_string()
    } else {
        " ◈ Process Info ".to_string()
    };

    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(border_color));
    f.render_widget(block, area);

    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });

    let lines: Vec<Line> = if app.show_hex_view {
        let total_lines = app.hex_data.lines().count();
        let visible_height = inner.height as usize;
        let start = app.hex_scroll.min(total_lines.saturating_sub(1));
        let search_lower = app.hex_search.to_lowercase();
        let has_hex_search = !search_lower.is_empty();

        app.hex_data
            .lines()
            .skip(start)
            .take(visible_height)
            .map(|l| {
                if has_hex_search && l.to_lowercase().contains(&search_lower) {
                    Line::from(l)
                        .style(Style::default().fg(PURPLE).add_modifier(Modifier::REVERSED))
                } else {
                    Line::from(l).style(Style::default().fg(PURPLE))
                }
            })
            .collect()
    } else if matches!(app.tab, Tab::Checkpoints) {
        if let Some(c) = app.selected_checkpoint() {
            let mut result = vec![
                Line::from(format!(
                    " ● Checkpoint: {:?}",
                    c.path.file_name().unwrap_or_default()
                ))
                .style(Style::default().fg(SUCCESS).bold()),
                Line::from("".to_string()),
                Line::from(" Details").style(Style::default().fg(CYAN).bold()),
                Line::from(format!("   Original PID: {}", c.pid))
                    .style(Style::default().fg(TEXT_PRIMARY)),
                Line::from(format!("   Command: {}", c.command))
                    .style(Style::default().fg(TEXT_PRIMARY)),
                Line::from(format!("   Memory Regions: {}", c.regions))
                    .style(Style::default().fg(TEXT_PRIMARY)),
                Line::from(format!("   Total Size: {}", c.human_size()))
                    .style(Style::default().fg(TEXT_PRIMARY)),
                Line::from(format!("   Created: {}", c.age_string()))
                    .style(Style::default().fg(TEXT_PRIMARY)),
            ];
            if c.path.join("regs.bin").exists() {
                result
                    .push(Line::from("   Registers: ✓ saved").style(Style::default().fg(SUCCESS)));
            }
            if c.path.join("fd").exists() {
                if let Ok(fd_count) = std::fs::read_dir(c.path.join("fd")) {
                    let count = fd_count.flatten().count();
                    result.push(
                        Line::from(format!("   File Descriptors: {} saved", count))
                            .style(Style::default().fg(SUCCESS)),
                    );
                }
            }
            result.push(Line::from("".to_string()));
            result.push(Line::from(" Files").style(Style::default().fg(CYAN).bold()));
            if let Ok(entries) = std::fs::read_dir(c.path.join("mem")) {
                let mut total_size = 0u64;
                for entry in entries.flatten() {
                    if let Ok(meta) = entry.metadata() {
                        total_size += meta.len();
                        let name = entry.file_name().to_string_lossy().to_string();
                        result.push(
                            Line::from(format!("   {} ({})", name, format_size(meta.len())))
                                .style(Style::default().fg(TEXT_PRIMARY)),
                        );
                    }
                }
                result.push(
                    Line::from(format!("   Total: {}", format_size(total_size)))
                        .style(Style::default().fg(ACCENT).bold()),
                );
            }
            result.push(Line::from("".to_string()));
            result.push(Line::from(" Actions").style(Style::default().fg(CYAN).bold()));
            result.push(
                Line::from("   u - Restore to original PID").style(Style::default().fg(TEXT_MUTED)),
            );
            result.push(
                Line::from("   d - Delete checkpoint").style(Style::default().fg(TEXT_MUTED)),
            );
            result
        } else {
            vec![Line::from(" ○ No checkpoint selected").style(Style::default().fg(TEXT_MUTED))]
        }
    } else if matches!(app.tab, Tab::Memory) && !app.dump_output.is_empty() {
        let visible_height = inner.height as usize;
        app.dump_output
            .lines()
            .skip(app.output_scroll)
            .take(visible_height)
            .map(|l| Line::from(l).style(Style::default().fg(TEXT_PRIMARY)))
            .collect()
    } else if matches!(app.tab, Tab::Memory) {
        vec![Line::from(" Select a memory region to view details")]
    } else if matches!(app.tab, Tab::Processes) && !app.process_info.is_empty() {
        let visible_height = inner.height as usize;
        app.process_info
            .lines()
            .skip(app.process_info_scroll)
            .take(visible_height)
            .map(|l| {
                if l.starts_with("  ") {
                    Line::from(l).style(Style::default().fg(PURPLE))
                } else {
                    Line::from(l).style(Style::default().fg(TEXT_PRIMARY))
                }
            })
            .collect()
    } else if matches!(app.tab, Tab::Processes) {
        if let Some(p) = app.selected_process() {
            vec![
                Line::from(format!(" ● PID: {}", p.pid)).style(Style::default().fg(SUCCESS).bold()),
                Line::from(format!("   Name: {}", p.name)).style(Style::default().fg(TEXT_PRIMARY)),
                Line::from(format!("   Memory: {}", format_size(p.memory_total)))
                    .style(Style::default().fg(TEXT_PRIMARY)),
                Line::from(format!("   Threads: {}", p.threads))
                    .style(Style::default().fg(TEXT_PRIMARY)),
                Line::from("".to_string()),
                Line::from(" Press Enter to view memory map")
                    .style(Style::default().fg(TEXT_MUTED)),
                Line::from(" Press c to create checkpoint").style(Style::default().fg(TEXT_MUTED)),
            ]
        } else {
            vec![Line::from(" ○ No process selected").style(Style::default().fg(TEXT_MUTED))]
        }
    } else {
        let log_len = app.output_log.len();
        let start = log_len.saturating_sub(MAX_LINES);
        app.output_log[start..]
            .iter()
            .map(|l| {
                if l.starts_with("[error]") {
                    Line::from(l.as_str()).style(Style::default().fg(ERROR))
                } else if l.starts_with("[status]") {
                    Line::from(l.as_str()).style(Style::default().fg(WARNING))
                } else {
                    Line::from(l.as_str()).style(Style::default().fg(TEXT_PRIMARY))
                }
            })
            .collect()
    };

    let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(paragraph, inner);

    if app.show_hex_view && app.focus == Focus::Output {
        let total_lines = app.hex_data.lines().count();
        let visible_height = inner.height as usize;
        if total_lines > visible_height {
            let scroll_pos =
                (app.hex_scroll as f64 / total_lines as f64 * visible_height as f64) as u16;
            let block = Block::bordered()
                .title("")
                .borders(ratatui::widgets::Borders::RIGHT)
                .border_style(Style::default().fg(ACCENT_DIM));
            let sb_area = Rect::new(
                inner.x + inner.width - 1,
                inner.y,
                inner.x + inner.width,
                inner.height,
            );
            f.render_widget(block, sb_area);
            if scroll_pos < inner.height {
                let thumb = Paragraph::new("█")
                    .style(Style::default().fg(ACCENT))
                    .alignment(Alignment::Center);
                f.render_widget(
                    thumb,
                    Rect::new(
                        sb_area.x + 2,
                        inner.y + scroll_pos,
                        sb_area.x + 2,
                        inner.y + scroll_pos + 1,
                    ),
                );
            }
        }
    } else if !app.show_hex_view
        && app.focus == Focus::Output
        && matches!(app.tab, Tab::Memory)
        && !app.dump_output.is_empty()
    {
        let total_lines = app.dump_output.lines().count();
        let visible_height = inner.height as usize;
        if total_lines > visible_height {
            let scroll_pos =
                (app.output_scroll as f64 / total_lines as f64 * visible_height as f64) as u16;
            let block = Block::bordered()
                .title("")
                .borders(ratatui::widgets::Borders::RIGHT)
                .border_style(Style::default().fg(ACCENT_DIM));
            let sb_area = Rect::new(
                inner.x + inner.width - 1,
                inner.y,
                inner.x + inner.width,
                inner.height,
            );
            f.render_widget(block, sb_area);
            if scroll_pos < inner.height {
                let thumb = Paragraph::new("█")
                    .style(Style::default().fg(ACCENT))
                    .alignment(Alignment::Center);
                f.render_widget(
                    thumb,
                    Rect::new(
                        sb_area.x + 2,
                        inner.y + scroll_pos,
                        sb_area.x + 2,
                        inner.y + scroll_pos + 1,
                    ),
                );
            }
        }
    }
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    if !app.input_mode && !app.is_searching && !app.is_hex_searching {
        return;
    }
    let title = if app.is_hex_searching {
        " Hex Search "
    } else {
        " Input "
    };
    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(ACCENT));
    f.render_widget(block, area);
    let inner = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    let text = if app.is_hex_searching {
        format!(" Search › {}", app.hex_search)
    } else if app.is_searching {
        format!(" Search › {}", app.search_query)
    } else {
        app.input_buffer.clone()
    };
    f.render_widget(
        Paragraph::new(text).style(Style::default().fg(TEXT_PRIMARY)),
        inner,
    );
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let pid_indicator = if app.selected_process().is_some() {
        "●"
    } else {
        "○"
    };
    let pid_color = if app.selected_process().is_some() {
        SUCCESS
    } else {
        TEXT_MUTED
    };
    let pid_str = app
        .selected_process()
        .map(|p| p.pid.to_string())
        .unwrap_or_else(|| "None".to_string());

    let (center_text, center_style) = if let Some(msg) = &app.status_message {
        let color = if app.status_is_error { ERROR } else { WARNING };
        (msg.clone(), Style::default().fg(color).bold())
    } else {
        let mode_tag = match app.focus {
            Focus::List => "[LIST]",
            Focus::Output => "[VIEW]",
        };
        (mode_tag.to_string(), Style::default().fg(CYAN).bold())
    };

    let help_text = if app.show_help {
        " Press any key "
    } else {
        " ?:help  q:quit  Tab:switch  Space:focus "
    };
    let left = format!(" ⌘ ckptmini {}  PID: {} ", pid_indicator, pid_str);
    let center = format!(" {} ", center_text);
    let help = help_text.to_string();
    let left_len = left.len() as u16;
    let help_len = help.len() as u16;
    let sep1 = area.x + left_len;
    let sep2 = area.x + area.width - help_len;
    f.render_widget(
        Paragraph::new(left).style(Style::default().fg(pid_color)),
        Rect::new(area.x, area.y, sep1, area.y + 1),
    );
    f.render_widget(
        Paragraph::new("│").style(Style::default().fg(ACCENT_DIM)),
        Rect::new(sep1, area.y, sep1 + 1, area.y + 1),
    );
    f.render_widget(
        Paragraph::new(center).style(center_style),
        Rect::new(sep1 + 1, area.y, sep2, area.y + 1),
    );
    f.render_widget(
        Paragraph::new("│").style(Style::default().fg(ACCENT_DIM)),
        Rect::new(sep2, area.y, sep2 + 1, area.y + 1),
    );
    f.render_widget(
        Paragraph::new(help).style(Style::default().fg(TEXT_MUTED)),
        Rect::new(sep2 + 1, area.y, area.width, area.y + 1),
    );
}

fn render_scrollbar(f: &mut Frame, area: Rect, total_items: usize, scroll_pos: usize) {
    let visible_height = area.height as usize;
    if total_items > visible_height {
        let scroll_pos = (scroll_pos as f64 / total_items as f64 * visible_height as f64) as u16;
        let thumb = Paragraph::new("█")
            .style(Style::default().fg(ACCENT))
            .alignment(Alignment::Center);
        f.render_widget(
            thumb,
            Rect::new(
                area.x + area.width - 1,
                area.y + scroll_pos,
                area.x + area.width,
                area.y + scroll_pos + 1,
            ),
        );
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
