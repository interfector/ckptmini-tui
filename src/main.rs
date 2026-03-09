use anyhow::Result;
use std::env;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{event, terminal};
use ratatui::{backend::CrosstermBackend, layout::Rect, widgets::Clear};

mod models;
mod theme;
mod ui;
mod wrapper;

use models::checkpoint::CheckpointInfo;
use ui::app::{App, Focus, Tab};
use ui::views::render_app;
use wrapper::parser;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let ckptmini_path = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "/usr/local/bin/ckptmini".to_string());
    let checkpoint_dir = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "/tmp/checkpoints".to_string());

    let output = std::process::Command::new(&ckptmini_path)
        .arg("--help")
        .output();

    match &output {
        Ok(o) if o.status.success() => {
            println!("ckptmini-tui - using ckptmini at: {}", ckptmini_path);
        }
        Ok(_) => {
            eprintln!("ckptmini exited with error");
        }
        Err(e) => {
            eprintln!("ckptmini not found at '{}': {}", ckptmini_path, e);
            eprintln!("Usage: {} [ckptmini-path] [checkpoint-dir]", args[0]);
            std::process::exit(1);
        }
    }

    std::fs::create_dir_all(&checkpoint_dir)?;

    let mut app = App::new(ckptmini_path.clone(), checkpoint_dir);
    app.add_output("ckptmini-tui started".to_string());

    refresh_processes(&mut app)?;
    refresh_checkpoints(&mut app)?;

    run_tui(&mut app)?;

    Ok(())
}

fn refresh_processes(app: &mut App) -> Result<()> {
    match models::process::list_processes() {
        Ok(processes) => {
            app.processes = processes;
            app.process_scroll = 0;
            app.sort_processes();
        }
        Err(e) => {
            app.set_error(format!("Failed to list processes: {}", e));
        }
    }
    Ok(())
}

fn refresh_checkpoints(app: &mut App) -> Result<()> {
    let dir = PathBuf::from(&app.checkpoint_dir);
    let paths = parser::list_checkpoints(&dir);
    app.checkpoints = paths
        .iter()
        .filter_map(|p| CheckpointInfo::from_dir(p))
        .collect();
    app.checkpoint_scroll = 0;
    Ok(())
}

fn run_tui(app: &mut App) -> Result<()> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    terminal::enable_raw_mode()?;
    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            let area = f.area();

            if app.show_help {
                let popup_width = 44u16;
                let popup_height = 20u16;
                let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
                let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
                let popup = Rect::new(x, y, x + popup_width, y + popup_height);
                f.render_widget(Clear, popup);
                ui::components::render_help(f, popup);
            } else {
                render_app::<CrosstermBackend<io::Stdout>>(f, app, area);
            }
        })?;

        if let Ok(event) = read_event(&mut terminal) {
            match event {
                Event::Quit => break,
                Event::Resize => {
                    terminal.clear()?;
                }
                Event::Key(c) => {
                    if app.show_help {
                        app.show_help = false;
                    } else {
                        handle_key(app, c)?;
                    }
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}

#[derive(Debug)]
enum Event {
    Key(crossterm::event::KeyEvent),
    Resize,
    Quit,
}

fn read_event<B: ratatui::backend::Backend>(
    _terminal: &mut ratatui::Terminal<B>,
) -> io::Result<Event> {
    loop {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                event::Event::Key(key) => {
                    if key.kind == event::KeyEventKind::Press {
                        if key.code == event::KeyCode::Char('q') {
                            return Ok(Event::Quit);
                        }
                        return Ok(Event::Key(key));
                    }
                }
                event::Event::Resize(_, _) => {
                    return Ok(Event::Resize);
                }
                _ => continue,
            }
        }
    }
}

fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) -> Result<()> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Tab => {
            app.next_tab();
        }
        KeyCode::BackTab => {
            app.prev_tab();
        }
        KeyCode::Char('n') | KeyCode::Char('N') if app.show_confirm => {
            app.show_confirm = false;
            app.confirm_message.clear();
            app.pending_hex_load = None;
            app.set_status("Checkpoint cancelled".to_string());
        }
        KeyCode::Char(' ') => {
            if app.is_searching {
                app.search_query.push(' ');
                let prefix = match app.tab {
                    Tab::Processes => "Search: ",
                    Tab::Memory => "Filter memory: ",
                    Tab::Checkpoints => "Filter checkpoints: ",
                };
                app.set_status(format!("{}{}", prefix, app.search_query));
                apply_search_filter(app);
            } else if app.is_hex_searching {
                app.hex_search.push(' ');
                app.set_status(format!("Hex search: {}", app.hex_search));
            } else {
                app.focus = match app.focus {
                    Focus::List => Focus::Output,
                    Focus::Output => Focus::List,
                };
                app.set_status(format!("Focus: {:?}", app.focus));
            }
        }
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
        }
        KeyCode::Esc => {
            if app.extended_mode {
                app.extended_mode = false;
                app.extended_command.clear();
                app.clear_status();
            } else if app.is_searching {
                let current_tab = app.tab;
                app.clear_saved_search();
                app.is_searching = false;
                app.search_query.clear();
                match current_tab {
                    Tab::Processes => {
                        let _ = refresh_processes(app);
                    }
                    Tab::Memory => {
                        app.memory_scroll = 0;
                        if let Some(p) = app.selected_process() {
                            let _ = load_memory_regions(app, p.pid);
                        }
                    }
                    Tab::Checkpoints => {
                        app.checkpoint_scroll = 0;
                        refresh_checkpoints(app)?;
                    }
                }
                app.clear_status();
            } else if app.is_hex_searching {
                app.is_hex_searching = false;
                app.hex_search.clear();
                app.clear_status();
            } else if app.show_hex_view {
                app.show_hex_view = false;
                app.hex_data.clear();
            }
        }
        KeyCode::Char(c) if app.is_searching => {
            app.search_query.push(c);
            let prefix = match app.tab {
                Tab::Processes => "Search: ",
                Tab::Memory => "Filter memory: ",
                Tab::Checkpoints => "Filter checkpoints: ",
            };
            app.set_status(format!("{}{}", prefix, app.search_query));
            apply_search_filter(app);
        }
        KeyCode::Enter if app.is_searching => {
            app.is_searching = false;
            if !app.search_query.is_empty() {
                app.save_current_search();
                apply_search_filter(app);
            }
            app.clear_status();
        }
        KeyCode::Char(c) if app.is_hex_searching => {
            app.hex_search.push(c);
            app.set_status(format!("Hex search: {}", app.hex_search));
        }
        KeyCode::Enter if app.is_hex_searching => {
            app.is_hex_searching = false;
            app.clear_status();
        }
        KeyCode::Enter if app.extended_mode => {
            if let Some(proc) = app.selected_process() {
                let symbol = app.extended_command.trim();
                if !symbol.is_empty() {
                    let runner = wrapper::CkptminiRunner::new(PathBuf::from(&app.ckptmini_path));
                    match runner.resolve(proc.pid, symbol) {
                        Ok(result) => {
                            app.dump_output = result;
                            app.set_status(format!("Resolved {} to: ", symbol));
                        }
                        Err(e) => {
                            app.set_error(format!("Resolve failed: {}", e));
                        }
                    }
                }
            }
            app.extended_mode = false;
            app.extended_command.clear();
        }
        KeyCode::Char(c) if app.extended_mode => {
            app.extended_command.push(c);
            app.set_status(format!(
                "Extended command (resolve <symbol>): {}",
                app.extended_command
            ));
        }
        KeyCode::Backspace if app.extended_mode => {
            app.extended_command.pop();
            if app.extended_command.is_empty() {
                app.set_status("Extended command (resolve <symbol>): ".to_string());
            } else {
                app.set_status(format!(
                    "Extended command (resolve <symbol>): {}",
                    app.extended_command
                ));
            }
        }
        KeyCode::Backspace if app.is_searching => {
            if app.search_query.is_empty() {
                app.is_searching = false;
                app.clear_saved_search();
                if matches!(app.tab, Tab::Processes) {
                    refresh_processes(app)?;
                } else if matches!(app.tab, Tab::Memory) {
                    app.memory_scroll = 0;
                    if let Some(p) = app.selected_process() {
                        let _ = load_memory_regions(app, p.pid);
                    }
                } else if matches!(app.tab, Tab::Checkpoints) {
                    app.checkpoint_scroll = 0;
                    refresh_checkpoints(app)?;
                }
                app.clear_status();
            } else {
                app.search_query.pop();
                let prefix = match app.tab {
                    Tab::Processes => "Search: ",
                    Tab::Memory => "Filter memory: ",
                    Tab::Checkpoints => "Filter checkpoints: ",
                };
                app.set_status(format!("{}{}", prefix, app.search_query));
                apply_search_filter(app);
            }
        }
        KeyCode::Backspace if app.is_hex_searching => {
            app.hex_search.pop();
            app.set_status(format!("Hex search: {}", app.hex_search));
        }
        KeyCode::Char('r') => {
            app.is_searching = false;
            app.search_query.clear();
            app.set_status("Refreshing...".to_string());
            refresh_processes(app)?;
            refresh_checkpoints(app)?;
            app.clear_status();
        }
        KeyCode::Char('/') => {
            if app.show_hex_view && app.focus == Focus::Output {
                app.is_hex_searching = true;
                app.hex_search.clear();
                app.set_status("Hex search: ".to_string());
            } else {
                if app.is_searching {
                    app.save_current_search();
                }
                app.is_searching = true;
                app.search_query.clear();
                let tab_label = match app.tab {
                    Tab::Processes => "Search: ",
                    Tab::Memory => "Filter memory: ",
                    Tab::Checkpoints => "Filter checkpoints: ",
                };
                app.set_status(tab_label.to_string());
            }
        }
        KeyCode::Char('M') => {
            app.sort_by = ui::app::SortBy::Memory;
            app.sort_ascending = !app.sort_ascending;
            app.sort_processes();
            app.set_status(
                if app.sort_ascending {
                    "Sorted by Memory (asc)"
                } else {
                    "Sorted by Memory (desc)"
                }
                .to_string(),
            );
        }
        KeyCode::Char('P') => {
            app.sort_by = ui::app::SortBy::Pid;
            app.sort_ascending = !app.sort_ascending;
            app.sort_processes();
            app.set_status(
                if app.sort_ascending {
                    "Sorted by PID (asc)"
                } else {
                    "Sorted by PID (desc)"
                }
                .to_string(),
            );
        }
        KeyCode::Char('N') => {
            app.sort_by = ui::app::SortBy::Name;
            app.sort_ascending = !app.sort_ascending;
            app.sort_processes();
            app.set_status(
                if app.sort_ascending {
                    "Sorted by Name (asc)"
                } else {
                    "Sorted by Name (desc)"
                }
                .to_string(),
            );
        }
        KeyCode::Char('c') => {
            if let Some(proc) = app.selected_process().cloned() {
                const GB: u64 = 1024 * 1024 * 1024;
                if app.total_memory_size > GB {
                    app.pending_hex_load = Some((proc.pid as u64, 0));
                    app.show_confirm = true;
                    app.confirm_message = format!(
                        "Total memory is {:.1} GB. Create checkpoint? [y/n]",
                        app.total_memory_size as f64 / GB as f64
                    );
                    app.set_status(app.confirm_message.clone());
                } else {
                    create_checkpoint(app, proc)?;
                }
            }
        }
        KeyCode::Char('y') | KeyCode::Char('Y') if app.show_confirm => {
            if let Some((pid, 0)) = app.pending_hex_load {
                if let Some(proc) = app.processes.iter().find(|p| p.pid as u64 == pid).cloned() {
                    create_checkpoint(app, proc)?;
                }
            }
            app.show_confirm = false;
            app.confirm_message.clear();
            app.pending_hex_load = None;
        }
        KeyCode::Char('n') if app.show_confirm => {
            app.show_confirm = false;
            app.confirm_message.clear();
            app.pending_hex_load = None;
            app.set_status("Checkpoint cancelled".to_string());
        }
        KeyCode::Char('u') => {
            if matches!(app.tab, Tab::Checkpoints) {
                let (ckpt_pid, ckpt_path) = app
                    .selected_checkpoint()
                    .map(|c| (c.pid, c.path.clone()))
                    .unwrap_or((0, PathBuf::new()));

                if ckpt_pid > 0 {
                    app.set_status(format!("Restoring checkpoint to PID {}...", ckpt_pid));
                    let runner = wrapper::CkptminiRunner::new(PathBuf::from(&app.ckptmini_path));
                    match runner.restore(ckpt_pid, &ckpt_path) {
                        Ok(_) => {
                            app.set_status(format!("Restored checkpoint to PID {}", ckpt_pid));
                        }
                        Err(e) => {
                            app.set_error(format!("Restore failed: {}", e));
                        }
                    }
                }
            }
        }
        KeyCode::Char('d') => {
            if matches!(app.tab, Tab::Checkpoints) {
                if let Some(ckpt) = app.selected_checkpoint().cloned() {
                    app.set_status(format!("Deleting checkpoint {:?}...", ckpt.path));
                    match std::fs::remove_dir_all(&ckpt.path) {
                        Ok(_) => {
                            app.set_status("Checkpoint deleted".to_string());
                            refresh_checkpoints(app)?;
                        }
                        Err(e) => {
                            app.set_error(format!("Delete failed: {}", e));
                        }
                    }
                }
            }
        }
        KeyCode::Char('p') => {
            if matches!(app.tab, Tab::Checkpoints) {
                if let Some(ckpt) = app.selected_checkpoint().cloned() {
                    if ckpt.pid > 0 {
                        let path = ckpt.path.clone();
                        app.set_status(format!("Parasite restore to PID {}", ckpt.pid));
                        let runner =
                            wrapper::CkptminiRunner::new(PathBuf::from(&app.ckptmini_path));
                        match runner.parasite(ckpt.pid, &path) {
                            Ok(_) => {
                                app.set_status(format!(
                                    "Parasite restore done for PID {}",
                                    ckpt.pid
                                ));
                            }
                            Err(e) => {
                                app.set_error(format!("Parasite failed: {}", e));
                            }
                        }
                    }
                }
            }
        }
        KeyCode::Char('i') => {
            if matches!(app.tab, Tab::Processes) {
                if let Some(proc) = app.selected_process().cloned() {
                    app.set_status(format!("Injecting shellcode to PID {}", proc.pid));
                    let runner = wrapper::CkptminiRunner::new(PathBuf::from(&app.ckptmini_path));
                    match runner.inject_shellcode(proc.pid) {
                        Ok(_) => {
                            app.set_status(format!("Shellcode injected to PID {}", proc.pid));
                        }
                        Err(e) => {
                            app.set_error(format!("Inject failed: {}", e));
                        }
                    }
                }
            }
        }
        KeyCode::Char('x') => {
            app.extended_mode = true;
            app.set_status("Extended command (resolve <symbol>): ".to_string());
        }
        KeyCode::Char('v') => {
            if matches!(app.tab, Tab::Memory) {
                if let Some(region) = app.selected_memory_region() {
                    load_hex_dump(app, region.start, region.end)?;
                }
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.show_hex_view && app.focus == Focus::Output {
                let total_lines = app.hex_data.lines().count();
                let visible_height = 20usize;
                app.hex_scroll =
                    (app.hex_scroll + 1).min(total_lines.saturating_sub(visible_height));
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Memory)
            {
                let total_lines = app.dump_output.lines().count();
                let visible_height = 20usize;
                app.output_scroll =
                    (app.output_scroll + 1).min(total_lines.saturating_sub(visible_height));
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Processes)
                && !app.process_info.is_empty()
            {
                let total_lines = app.process_info.lines().count();
                let visible_height = 20usize;
                app.process_info_scroll =
                    (app.process_info_scroll + 1).min(total_lines.saturating_sub(visible_height));
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Checkpoints)
                && app.selected_checkpoint().is_some()
            {
            } else {
                match app.tab {
                    Tab::Processes => {
                        if app.focus == Focus::List
                            && app.process_scroll < app.processes.len().saturating_sub(1)
                        {
                            app.process_scroll += 1;
                            let (name, pid) = if let Some(p) = app.selected_process() {
                                (p.name.clone(), p.pid)
                            } else {
                                (String::new(), 0)
                            };
                            if pid > 0 {
                                app.set_status(format!("Selected: {} - PID {}", name, pid));
                                let _ = load_process_info(app, pid);
                            }
                        }
                    }
                    Tab::Memory => {
                        if app.focus == Focus::List
                            && app.memory_scroll < app.memory_regions.len().saturating_sub(1)
                        {
                            app.memory_scroll += 1;
                            let (start, end, path) = if let Some(r) = app.selected_memory_region() {
                                (
                                    r.start,
                                    r.end,
                                    r.path.clone().unwrap_or_else(|| "[anon]".to_string()),
                                )
                            } else {
                                (0, 0, String::new())
                            };
                            if !path.is_empty() {
                                app.set_status(format!(
                                    "Selected: 0x{:x} - {} - {}",
                                    start,
                                    crate::models::memory::MemoryRegion::format_size(end - start),
                                    path,
                                ));
                                let _ = load_hex_dump(app, start, end);
                            }
                        }
                    }
                    Tab::Checkpoints => {
                        if app.focus == Focus::List
                            && app.checkpoint_scroll < app.checkpoints.len().saturating_sub(1)
                        {
                            app.checkpoint_scroll += 1;
                            if let Some(c) = app.selected_checkpoint() {
                                app.set_status(format!(
                                    "Selected checkpoint: {:?}",
                                    c.path.file_name().unwrap_or_default()
                                ));
                            }
                        }
                    }
                }
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.show_hex_view && app.focus == Focus::Output {
                if app.hex_scroll > 0 {
                    app.hex_scroll -= 1;
                }
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Memory)
            {
                if app.output_scroll > 0 {
                    app.output_scroll -= 1;
                }
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Processes)
                && !app.process_info.is_empty()
            {
                if app.process_info_scroll > 0 {
                    app.process_info_scroll -= 1;
                }
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Checkpoints)
                && app.selected_checkpoint().is_some()
            {
            } else {
                match app.tab {
                    Tab::Processes => {
                        if app.focus == Focus::List && app.process_scroll > 0 {
                            app.process_scroll -= 1;
                            let (name, pid) = if let Some(p) = app.selected_process() {
                                (p.name.clone(), p.pid)
                            } else {
                                (String::new(), 0)
                            };
                            if pid > 0 {
                                app.set_status(format!("Selected: {} - PID {}", name, pid));
                                let _ = load_process_info(app, pid);
                            }
                        }
                    }
                    Tab::Memory => {
                        if app.focus == Focus::List && app.memory_scroll > 0 {
                            app.memory_scroll -= 1;
                            let (start, end, path) = if let Some(r) = app.selected_memory_region() {
                                (
                                    r.start,
                                    r.end,
                                    r.path.clone().unwrap_or_else(|| "[anon]".to_string()),
                                )
                            } else {
                                (0, 0, String::new())
                            };
                            if !path.is_empty() {
                                app.set_status(format!(
                                    "Selected: 0x{:x} - {} - {}",
                                    start,
                                    crate::models::memory::MemoryRegion::format_size(end - start),
                                    path,
                                ));
                                let _ = load_hex_dump(app, start, end);
                            }
                        }
                    }
                    Tab::Checkpoints => {
                        if app.focus == Focus::List && app.checkpoint_scroll > 0 {
                            app.checkpoint_scroll -= 1;
                            if let Some(c) = app.selected_checkpoint() {
                                app.set_status(format!(
                                    "Selected checkpoint: {:?}",
                                    c.path.file_name().unwrap_or_default()
                                ));
                            }
                        }
                    }
                }
            }
        }
        KeyCode::PageDown => {
            if app.show_hex_view && app.focus == Focus::Output {
                let total_lines = app.hex_data.lines().count();
                let visible_height = 20usize;
                app.hex_scroll = (app.hex_scroll + visible_height)
                    .min(total_lines.saturating_sub(visible_height));
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Memory)
            {
                let total_lines = app.dump_output.lines().count();
                let visible_height = 20usize;
                app.output_scroll = (app.output_scroll + visible_height)
                    .min(total_lines.saturating_sub(visible_height));
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Processes)
                && !app.process_info.is_empty()
            {
                let total_lines = app.process_info.lines().count();
                let visible_height = 20usize;
                app.process_info_scroll = (app.process_info_scroll + visible_height)
                    .min(total_lines.saturating_sub(visible_height));
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Checkpoints)
                && app.selected_checkpoint().is_some()
            {
            } else {
                match app.tab {
                    Tab::Processes => {
                        let jump = 10.min(
                            app.processes
                                .len()
                                .saturating_sub(1)
                                .saturating_sub(app.process_scroll),
                        );
                        app.process_scroll += jump;
                        if let Some(p) = app.selected_process() {
                            let _ = load_process_info(app, p.pid);
                        }
                    }
                    Tab::Memory => {
                        let jump = 10.min(
                            app.memory_regions
                                .len()
                                .saturating_sub(1)
                                .saturating_sub(app.memory_scroll),
                        );
                        app.memory_scroll += jump;
                        if let Some(r) = app.selected_memory_region() {
                            let _ = load_hex_dump(app, r.start, r.end);
                        }
                    }
                    Tab::Checkpoints => {
                        let jump = 10.min(
                            app.checkpoints
                                .len()
                                .saturating_sub(1)
                                .saturating_sub(app.checkpoint_scroll),
                        );
                        app.checkpoint_scroll += jump;
                    }
                }
            }
        }
        KeyCode::PageUp => {
            if app.show_hex_view && app.focus == Focus::Output {
                let visible_height = 20usize;
                app.hex_scroll = app.hex_scroll.saturating_sub(visible_height);
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Memory)
            {
                let visible_height = 20usize;
                app.output_scroll = app.output_scroll.saturating_sub(visible_height);
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Processes)
                && !app.process_info.is_empty()
            {
                let visible_height = 20usize;
                app.process_info_scroll = app.process_info_scroll.saturating_sub(visible_height);
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Checkpoints)
                && app.selected_checkpoint().is_some()
            {
            } else {
                match app.tab {
                    Tab::Processes => {
                        let jump = 10.min(app.process_scroll);
                        app.process_scroll -= jump;
                        if let Some(p) = app.selected_process() {
                            let _ = load_process_info(app, p.pid);
                        }
                    }
                    Tab::Memory => {
                        let jump = 10.min(app.memory_scroll);
                        app.memory_scroll -= jump;
                        if let Some(r) = app.selected_memory_region() {
                            let _ = load_hex_dump(app, r.start, r.end);
                        }
                    }
                    Tab::Checkpoints => {
                        let jump = 10.min(app.checkpoint_scroll);
                        app.checkpoint_scroll -= jump;
                    }
                }
            }
        }
        KeyCode::Home => {
            if app.show_hex_view && app.focus == Focus::Output {
                app.hex_scroll = 0;
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Memory)
            {
                app.output_scroll = 0;
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Processes)
                && !app.process_info.is_empty()
            {
                app.process_info_scroll = 0;
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Checkpoints)
                && app.selected_checkpoint().is_some()
            {
            } else {
                match app.tab {
                    Tab::Processes => {
                        app.process_scroll = 0;
                        if let Some(p) = app.selected_process() {
                            let _ = load_process_info(app, p.pid);
                        }
                    }
                    Tab::Memory => {
                        app.memory_scroll = 0;
                        if let Some(r) = app.selected_memory_region() {
                            let _ = load_hex_dump(app, r.start, r.end);
                        }
                    }
                    Tab::Checkpoints => {
                        app.checkpoint_scroll = 0;
                    }
                }
            }
        }
        KeyCode::End => {
            if app.show_hex_view && app.focus == Focus::Output {
                let total_lines = app.hex_data.lines().count();
                let visible_height = 20usize;
                app.hex_scroll = total_lines.saturating_sub(visible_height);
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Memory)
            {
                let total_lines = app.dump_output.lines().count();
                let visible_height = 20usize;
                app.output_scroll = total_lines.saturating_sub(visible_height);
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Processes)
                && !app.process_info.is_empty()
            {
                let total_lines = app.process_info.lines().count();
                let visible_height = 20usize;
                app.process_info_scroll = total_lines.saturating_sub(visible_height);
            } else if !app.show_hex_view
                && app.focus == Focus::Output
                && matches!(app.tab, Tab::Checkpoints)
                && app.selected_checkpoint().is_some()
            {
            } else {
                match app.tab {
                    Tab::Processes => {
                        app.process_scroll = app.processes.len().saturating_sub(1);
                        if let Some(p) = app.selected_process() {
                            let _ = load_process_info(app, p.pid);
                        }
                    }
                    Tab::Memory => {
                        app.memory_scroll = app.memory_regions.len().saturating_sub(1);
                        if let Some(r) = app.selected_memory_region() {
                            let _ = load_hex_dump(app, r.start, r.end);
                        }
                    }
                    Tab::Checkpoints => {
                        app.checkpoint_scroll = app.checkpoints.len().saturating_sub(1);
                    }
                }
            }
        }
        KeyCode::Enter => match app.tab {
            Tab::Processes => {
                if let Some(pid) = app.selected_process().map(|p| p.pid) {
                    app.tab = Tab::Memory;
                    app.memory_scroll = 0;
                    app.show_hex_view = false;
                    app.hex_data.clear();
                    load_memory_regions(app, pid)?;
                }
            }
            Tab::Memory => {
                if let Some(region) = app.selected_memory_region() {
                    load_hex_dump(app, region.start, region.end)?;
                }
            }
            Tab::Checkpoints => {}
        },
        _ => {}
    }

    Ok(())
}

fn filter_processes_by_search(app: &mut App) {
    let query = app.search_query.to_lowercase();
    if query.is_empty() {
        return;
    }

    let all_processes = match models::process::list_processes() {
        Ok(p) => p,
        Err(_) => return,
    };

    app.processes = all_processes
        .into_iter()
        .filter(|p| p.name.to_lowercase().contains(&query) || p.pid.to_string().contains(&query))
        .collect();

    app.process_scroll = 0;
    app.sort_processes();
}

fn apply_search_filter(app: &mut App) {
    let query = app.search_query.to_lowercase();
    if query.is_empty() {
        return;
    }

    match app.tab {
        Tab::Processes => {
            filter_processes_by_search(app);
        }
        Tab::Memory => {
            if let Some(proc) = app.selected_process() {
                let runner = wrapper::CkptminiRunner::new(PathBuf::from(&app.ckptmini_path));
                if let Ok(output) = runner.dump(proc.pid) {
                    let all_regions = wrapper::parser::parse_memory_regions(&output);
                    app.memory_regions = all_regions
                        .into_iter()
                        .filter(|r| {
                            let path = r.path.clone().unwrap_or_default();
                            let addr = format!("{:x}", r.start);
                            let perms = r.perms.as_string();
                            path.to_lowercase().contains(&query)
                                || addr.contains(&query)
                                || perms.to_lowercase().contains(&query)
                        })
                        .collect();
                    app.memory_scroll = 0;
                }
            }
        }
        Tab::Checkpoints => {
            let dir = PathBuf::from(&app.checkpoint_dir);
            let paths = wrapper::parser::list_checkpoints(&dir);
            let all_checkpoints: Vec<CheckpointInfo> = paths
                .iter()
                .filter_map(|p| CheckpointInfo::from_dir(p))
                .collect();
            app.checkpoints = all_checkpoints
                .into_iter()
                .filter(|c| {
                    let name = c
                        .path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    name.contains(&query)
                })
                .collect();
            app.checkpoint_scroll = 0;
        }
    }
}

fn load_process_info(app: &mut App, pid: u32) -> Result<()> {
    let runner = wrapper::CkptminiRunner::new(PathBuf::from(&app.ckptmini_path));
    match runner.show(pid) {
        Ok(output) => {
            app.process_info = output;
        }
        Err(e) => {
            app.process_info = format!("Failed to get process info: {}", e);
        }
    }
    Ok(())
}

fn load_memory_regions(app: &mut App, pid: u32) -> Result<()> {
    let runner = wrapper::CkptminiRunner::new(PathBuf::from(&app.ckptmini_path));
    match runner.dump(pid) {
        Ok(output) => {
            app.dump_output = output.clone();
            app.memory_regions = wrapper::parser::parse_memory_regions(&output);
            app.total_memory_size = app.memory_regions.iter().map(|r| r.end - r.start).sum();
            let total_mb = app.total_memory_size / (1024 * 1024);
            app.set_status(format!(
                "Loaded {} memory regions ({:.1} MB)",
                app.memory_regions.len(),
                total_mb
            ));
        }
        Err(e) => {
            app.set_error(format!("Failed to load memory: {}", e));
            app.memory_regions.clear();
            app.dump_output.clear();
            app.total_memory_size = 0;
        }
    }
    Ok(())
}

fn load_hex_dump(app: &mut App, start: u64, end: u64) -> Result<()> {
    if let Some(proc) = app.selected_process() {
        let size = (end - start).min(4096);
        let runner = wrapper::CkptminiRunner::new(PathBuf::from(&app.ckptmini_path));
        match runner.read_memory(proc.pid, start, size as usize) {
            Ok(data) => {
                app.hex_data = format_hex_dump(start, &data);
                app.show_hex_view = true;
                app.set_status(format!("Hex dump: 0x{:x} - {} bytes", start, size));
            }
            Err(e) => {
                app.set_error(format!("Failed to read memory: {}", e));
                app.show_hex_view = false;
            }
        }
    }
    Ok(())
}

fn format_hex_dump(_addr: u64, data: &str) -> String {
    let mut result = String::new();

    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Reading") {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 16 {
            continue;
        }

        let addr_val = parts[0];
        result.push_str(&format!("{}  ", addr_val));

        let bytes: Vec<u8> = parts
            .iter()
            .skip(1)
            .take(16)
            .filter_map(|s| u8::from_str_radix(s, 16).ok())
            .collect();

        for i in 0..16 {
            if i < bytes.len() {
                result.push_str(&format!("{:02x} ", bytes[i]));
            } else {
                result.push_str("   ");
            }
            if i == 7 {
                result.push(' ');
            }
        }

        result.push_str(" |");
        for &b in &bytes {
            let c = if b >= 32 && b < 127 { b as char } else { '.' };
            result.push(c);
        }
        result.push_str("|\n");
    }

    result
}

fn create_checkpoint(app: &mut App, proc: models::process::ProcessInfo) -> Result<()> {
    let checkpoint_path = PathBuf::from(&app.checkpoint_dir).join(format!(
        "ckpt_{}_{}",
        proc.pid,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    app.set_status(format!("Creating checkpoint for PID {}...", proc.pid));

    let runner = wrapper::CkptminiRunner::new(PathBuf::from(&app.ckptmini_path));
    match runner.save(proc.pid, &checkpoint_path) {
        Ok(_) => {
            let meta = serde_json::json!({
                "pid": proc.pid,
                "command": proc.name,
            });
            let _ = std::fs::write(
                checkpoint_path.join("meta.json"),
                serde_json::to_string_pretty(&meta).unwrap_or_default(),
            );
            app.set_status(format!("Checkpoint saved to {:?}", checkpoint_path));
            refresh_checkpoints(app)?;
        }
        Err(e) => {
            app.set_error(format!("Checkpoint failed: {}", e));
        }
    }
    Ok(())
}
