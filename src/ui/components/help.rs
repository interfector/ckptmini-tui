use crate::theme::*;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub fn render_help(f: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title(" Keyboard Shortcuts ")
        .border_style(Style::default().fg(ACCENT));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let key_style = Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD);
    let header_style = Style::default().fg(ACCENT).add_modifier(Modifier::BOLD);

    let lines = vec![
        Line::from(vec![Span::styled("General", header_style)]),
        Line::from(vec![Span::styled("  q ", key_style), Span::raw(" Quit")]),
        Line::from(vec![
            Span::styled("  ? ", key_style),
            Span::raw(" Toggle help"),
        ]),
        Line::from(vec![
            Span::styled("  Tab ", key_style),
            Span::raw(" Switch tab"),
        ]),
        Line::from(vec![
            Span::styled("  Space ", key_style),
            Span::raw(" Toggle focus (list/output)"),
        ]),
        Line::from(vec![
            Span::styled("  r ", key_style),
            Span::raw(" Refresh list"),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled("Search & Filter", header_style)]),
        Line::from(vec![
            Span::styled("  / ", key_style),
            Span::raw(" Search/filter (tab-specific)"),
        ]),
        Line::from(vec![
            Span::styled("  Esc ", key_style),
            Span::raw(" Cancel search/close view"),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled("Sorting (Processes)", header_style)]),
        Line::from(vec![
            Span::styled("  M ", key_style),
            Span::raw(" By memory"),
        ]),
        Line::from(vec![Span::styled("  P ", key_style), Span::raw(" By PID")]),
        Line::from(vec![Span::styled("  N ", key_style), Span::raw(" By name")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled("List Navigation", header_style)]),
        Line::from(vec![
            Span::styled("  ↑/↓ or j/k ", key_style),
            Span::raw(" Navigate up/down"),
        ]),
        Line::from(vec![
            Span::styled("  PgUp/PgDn ", key_style),
            Span::raw(" Page up/down"),
        ]),
        Line::from(vec![
            Span::styled("  Home/End ", key_style),
            Span::raw(" First/last"),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "Output Navigation (when focused)",
            header_style,
        )]),
        Line::from(vec![
            Span::styled("  ↑/↓ or j/k ", key_style),
            Span::raw(" Scroll content"),
        ]),
        Line::from(vec![
            Span::styled("  PgUp/PgDn ", key_style),
            Span::raw(" Page through content"),
        ]),
        Line::from(vec![
            Span::styled("  Home/End ", key_style),
            Span::raw(" Jump to start/end"),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled("Processes Tab", header_style)]),
        Line::from(vec![
            Span::styled("  Enter ", key_style),
            Span::raw(" View memory map"),
        ]),
        Line::from(vec![
            Span::styled("  c ", key_style),
            Span::raw(" Create checkpoint"),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled("Memory Tab", header_style)]),
        Line::from(vec![
            Span::styled("  v or Enter ", key_style),
            Span::raw(" View hex dump"),
        ]),
        Line::from(vec![
            Span::styled("  / ", key_style),
            Span::raw(" Filter regions"),
        ]),
        Line::from(vec![
            Span::styled("  Esc ", key_style),
            Span::raw(" Back to regions"),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled("Hex Dump View", header_style)]),
        Line::from(vec![
            Span::styled("  / ", key_style),
            Span::raw(" Search in hex"),
        ]),
        Line::from(vec![
            Span::styled("  Esc ", key_style),
            Span::raw(" Close hex view"),
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled("Checkpoints Tab", header_style)]),
        Line::from(vec![
            Span::styled("  u ", key_style),
            Span::raw(" Restore checkpoint"),
        ]),
        Line::from(vec![
            Span::styled("  d ", key_style),
            Span::raw(" Delete checkpoint"),
        ]),
        Line::from(vec![
            Span::styled("  / ", key_style),
            Span::raw(" Filter checkpoints"),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(TEXT_PRIMARY))
        .block(Block::default());

    f.render_widget(paragraph, inner);
}
