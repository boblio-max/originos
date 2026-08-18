use std::{
    io::{Read, Write},
    path::Path,
    sync::{Arc, Mutex},
    thread,
};
// use crossterm::event::KeyModifiers;

mod command_bubble;
mod commands;
mod quick_settings;
mod taskbar;
mod settingswindow;
mod systeminfo;
mod window2;
mod window3;
mod window4;
mod window5;

use command_bubble::CommandBubble;
use commands::OriginShell;
use quick_settings::QuickSettings;
use taskbar::Taskbar;
use settingswindow::SettingsWindow;
use systeminfo::SystemInfo;
use window2::Window2;
use window3::Window3;
use window4::Window4;
use window5::Window5;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

slint::include_modules!();

#[derive(Default)]
struct WindowDrawTracker {
    draw_history: Vec<usize>,
}

impl WindowDrawTracker {
    fn record_draw(&mut self, window_index: usize) {
        self.draw_history.push(window_index);
    }

    fn latest_window_drawn_index(&self) -> Option<usize> {
        self.draw_history.last().copied()
    }
}

struct Translator {
    x_translation: i32,
    y_translation: i32,
    x_scale: i32,
    y_scale: i32,
}

impl Translator {
    fn new() -> Self {
        Self {
            x_translation: 0,
            y_translation: 0,
            x_scale: 0,
            y_scale: 0,
        }
    }

    fn translate(&mut self, delta_x: i32, delta_y: i32) -> (i32, i32) {
        self.x_translation += delta_x;
        self.y_translation += delta_y;
        (self.x_translation, self.y_translation)
    }
    fn scale(&mut self, delta_x: i32, delta_y: i32) -> (i32, i32) {
        self.x_scale += delta_x;
        self.y_scale += delta_y;
        (self.x_scale, self.y_scale)
    }
}
#[cfg(test)]
mod tests {
    use super::WindowDrawTracker;

    #[test]
    fn tracks_latest_drawn_window() {
        let mut tracker = WindowDrawTracker::default();
        tracker.record_draw(0);
        tracker.record_draw(2);
        tracker.record_draw(4);

        assert_eq!(tracker.latest_window_drawn_index(), Some(4));
    }
}

/// Resolve a usable shell program.
///
/// Preference order:
///   1. `$SHELL`, if it points at an existing binary.
///   2. A Git Bash install (Windows only).
///   3. PowerShell on Windows / `/bin/bash` on Unix.
///
/// Returns `(program, interactive_flag)`. Bash needs `-i` for an
/// interactive prompt; Windows shells must not receive it.
fn find_shell() -> (String, Option<&'static str>) {
    if let Ok(shell) = std::env::var("SHELL") {
        if !shell.trim().is_empty() && Path::new(&shell).exists() {
            return (shell, Some("-i"));
        }
    }

    #[cfg(windows)]
    {
        const GIT_BASH: &[&str] = &[
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\Program Files\Git\usr\bin\bash.exe",
            r"C:\Program Files (x86)\Git\bin\bash.exe",
        ];

        for bash in GIT_BASH {
            if Path::new(bash).exists() {
                return ((*bash).to_string(), Some("-i"));
            }
        }

        (String::from("powershell.exe"), None)
    }

    #[cfg(not(windows))]
    {
        (String::from("/bin/bash"), Some("-i"))
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ─────────────────────────────────────────────
    // CREATE WINDOW
    // ─────────────────────────────────────────────

    let window = MainWindow::new()?;
    let origin_shell = OriginShell::new();
    let draw_tracker = Arc::new(Mutex::new(WindowDrawTracker::default()));
    {
        let mut tracker = draw_tracker.lock().unwrap();
        tracker.record_draw(0);
    }
    // ─────────────────────────────────────────────
    // CREATE PTYs
    // ─────────────────────────────────────────────

    let pty_system = native_pty_system();

    let pty_size = PtySize {
        rows: 40,
        cols: 120,
        pixel_width: 0,
        pixel_height: 0,
    };

    // The bubble is a real secondary terminal, so it needs enough
    // rows to actually render a prompt AND command output. 1 row
    // was the earlier bug — bash had no room to draw anything.
    let bubble_size = PtySize {
        rows: 10,
        cols: 44,
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system.openpty(pty_size)?;
    let bpair = pty_system.openpty(bubble_size)?;

    // ─────────────────────────────────────────────
    // START SHELLS
    // ─────────────────────────────────────────────

    let (shell, shell_interactive) = find_shell();
    let (bshell, bshell_interactive) = find_shell();

    let mut cmd = CommandBuilder::new(shell);
    let mut bcmd = CommandBuilder::new(bshell);
    // Interactive shell.
    //
    // Bash owns the prompt.
    // Origin OS does NOT draw one.
    if let Some(arg) = shell_interactive {
        cmd.arg(arg);
    }
    if let Some(arg) = bshell_interactive {
        bcmd.arg(arg);
    }

    cmd.env("TERM", "xterm-256color");
    bcmd.env("TERM", "xterm-256color");
    // Give the terminal its Origin OS identity.
    cmd.env(
        "PS1",
        "\\[\\e[32m\\]origin-dev@OrOS-DEV:\\w$ \\[\\e[0m\\]",
    );
    bcmd.env(
        "PS1",
        "origin-dev@OrOS-DEV:~$ ",
    );

    cmd.env("PS2", "> ");
    bcmd.env("PS2", "> ");

    let mut child = pair.slave.spawn_command(cmd)?;
    let mut bchild = bpair.slave.spawn_command(bcmd)?;
    drop(pair.slave);
    drop(bpair.slave);
    // ─────────────────────────────────────────────
    // PTY READERS
    // ─────────────────────────────────────────────

    let mut pty_reader = pair.master.try_clone_reader()?;
    let mut bubble_reader = bpair.master.try_clone_reader()?;
    // ─────────────────────────────────────────────
    // PTY WRITERS
    // ─────────────────────────────────────────────

    let pty_writer = pair.master.take_writer()?;
    let bwriter = bpair.master.take_writer()?;

    // Multiple UI events can access the writer,
    // so protect it with a Mutex.
    let pty_writer = Arc::new(Mutex::new(pty_writer));
    let bwriter = Arc::new(Mutex::new(bwriter));
    // ─────────────────────────────────────────────
    // VT100 TERMINAL STATE
    // ─────────────────────────────────────────────

    let parser = Arc::new(Mutex::new(
        vt100::Parser::new(
            pty_size.rows,
            pty_size.cols,
            0,
        ),
    ));

    let bubble_parser = Arc::new(Mutex::new(
        vt100::Parser::new(
            bubble_size.rows,
            bubble_size.cols,
            0,
        ),
    ));
    // ─────────────────────────────────────────────
    // PTY → VT100 → SLINT
    // ─────────────────────────────────────────────

    {
        let parser = parser.clone();
        let weak_window = window.as_weak();

        thread::spawn(move || {
            let mut buffer = [0u8; 8192];

            loop {
                match pty_reader.read(&mut buffer) {
                    Ok(0) => {
                        // Shell exited.
                        break;
                    }

                    Ok(n) => {
                        // Feed raw PTY bytes into VT100.
                        {
                            let mut parser =
                                parser.lock().unwrap();

                            parser.process(&buffer[..n]);
                        }

                        let parser = parser.clone();

                        // Update Slint on the UI thread.
                        let _ =
                            weak_window
                                .upgrade_in_event_loop(
                                    move |window| {
                                        let screen = {
                                            let parser =
                                                parser
                                                    .lock()
                                                    .unwrap();

                                            parser
                                                .screen()
                                                .contents()
                                        };

                                        window
                                            .set_terminal_text(
                                                screen.into(),
                                            );
                                    },
                                );
                    }

                    Err(_) => {
                        break;
                    }
                }
            }
        });
    }

    {
        let bubble_parser = bubble_parser.clone();
        let weak_window = window.as_weak();
        let origin_shell = Arc::new(origin_shell);

        thread::spawn(move || {
            let mut buffer = [0u8; 8192];

            loop {
                match bubble_reader.read(&mut buffer) {
                    Ok(0) => break,

                    Ok(n) => {
                        {
                            let mut parser =
                                bubble_parser.lock().unwrap();

                            parser.process(&buffer[..n]);
                        }

                        let bubble_parser =
                            bubble_parser.clone();
                        let origin_shell = origin_shell.clone();

                        let _ =
                            weak_window
                                .upgrade_in_event_loop(
                                    move |window| {
                                        let screen = {
                                            let parser =
                                                bubble_parser
                                                    .lock()
                                                    .unwrap();

                                            parser
                                                .screen()
                                                .contents()
                                        };

                                        // Trim trailing blank lines from the
                                        // fixed-height PTY screen.
                                        let trimmed: String = screen
                                            .trim_end_matches(['\n', ' '])
                                            .to_string();

                                        const BUBBLE_PROMPT: &str =
                                            "origin-dev@OrOS-DEV:~$";

                                        // If the bubble has real output
                                        // (more than just the bare prompt),
                                        // print it to the console.
                                        let lines: Vec<&str> = trimmed.lines().collect();

                                        // Build the "generated" text shown in
                                        // the output bubble. It comes from the
                                        // OriginShell command loader; commands
                                        // without a loader response (open,
                                        // close, restart) produce no text.
                                        let clpass = if lines.len() >= 2 {
                                            let output = lines[1..]
                                                .iter()
                                                .copied()
                                                .filter(|line| {
                                                    !line.trim().is_empty()
                                                        && !line.contains(BUBBLE_PROMPT)
                                                })
                                                .collect::<Vec<_>>()
                                                .join("\n");

                                            origin_shell.command_loader(&output).join("\n")
                                        } else {
                                            String::new()
                                        };

                                        // Show the output bubble only while there
                                        // is actually generated text to display.
                                        if clpass.is_empty() {
                                            window.set_output_generated(false);
                                            window.set_bubble_gen_text(String::new().into());
                                        } else {
                                            window.set_output_generated(true);
                                            window.set_bubble_gen_text(clpass.into());
                                        }

                                        // The visible box only shows the
                                        // CURRENT line — this gives live
                                        // typing feedback while a command
                                        // is being entered, but once the
                                        // command finishes and a fresh
                                        // prompt appears on its own line,
                                        // the box effectively "clears" to
                                        // just that prompt. Full output
                                        // still goes to println! above.
                                        let last_line = trimmed
                                            .lines()
                                            .last()
                                            .unwrap_or(BUBBLE_PROMPT)
                                            .to_string();

                                        window.set_bubble_text(
                                            last_line.into(),
                                        );
                                    },
                                );
                    }

                    Err(_) => break,
                }
            }
        });
    }

    // ─────────────────────────────────────────────
    // KEYBOARD → PTY
    // ─────────────────────────────────────────────

    {
        let writer = pty_writer.clone();
        let bubble_writer = bwriter.clone();
        let weak_window_for_input = window.as_weak();
        let draw_tracker_for_input = draw_tracker.clone();

        let mut bubble = CommandBubble::new();
        let mut tab_held = false;
        let mut shift_held = false;
        // let mut arrow_held = false;
        let mut quick_settings = QuickSettings::new();
        let mut taskbar = Taskbar::new();
        let mut settings = SettingsWindow::new();
        let mut systeminfo = SystemInfo::new();
        let mut window2 = Window2::new();
        let mut window3 = Window3::new();
        let mut window4 = Window4::new();
        let mut window5 = Window5::new();
        let mut translator = Translator::new();
        window.on_key_input(
            move |key, control| {

                if key == "\u{0009}" {
                    tab_held = true;
                    return;
                }
                if key == "\u{0053}"  {
                    shift_held = true;
                    return;
                }


                if key.eq_ignore_ascii_case("q") && tab_held {
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        bubble.toggle(&w1);
                        let mut tracker = draw_tracker_for_input.lock().unwrap();
                        tracker.record_draw(1);
                    }
                    tab_held = false;

                    println!(
                        "Keyboard mode: {}",
                        if bubble.active() { "BUBBLE" } else { "TERMINAL" }
                    );
                    return;
                }

                if key.eq_ignore_ascii_case("s") && tab_held {
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        quick_settings.toggle(&w1);
                        let mut tracker = draw_tracker_for_input.lock().unwrap();
                        tracker.record_draw(2);
                    }
                    tab_held = false;

                    println!(
                        "Keyboard mode: {}",
                        if quick_settings.active() { "QSETTINGS" } else { "TERMINAL" }
                    );
                    return;
                }
                if quick_settings.active() {
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        if quick_settings.handle_key(&w1, key.as_str()) {
                            return;
                        }
                    }
                    return;
                }

                if key.eq_ignore_ascii_case("t") && tab_held {
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        taskbar.toggle(&w1);
                        let mut tracker = draw_tracker_for_input.lock().unwrap();
                        tracker.record_draw(4);
                    }
                    tab_held = false;

                    println!(
                        "Keyboard mode: {}",
                        if taskbar.active() { "DOCK" } else { "TERMINAL" }
                    );

                    return;
                }
                if key.eq("\u{f700}") && tab_held && !shift_held {
                    let latest_index = draw_tracker_for_input
                        .lock()
                        .unwrap()
                        .latest_window_drawn_index()
                        .unwrap_or(0);
                    let translated = translator.translate(0, -5);
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        settings.translate(&w1, translated.0, translated.1);
                    }
                    println!(
                        "Latest drawn window index: {} -> translated ({}, {})",
                        latest_index,
                        translated.0,
                        translated.1
                    );

                    tab_held = false;
                }
                if key.eq("\u{f701}") && tab_held && !shift_held {
                    let latest_index = draw_tracker_for_input
                        .lock()
                        .unwrap()
                        .latest_window_drawn_index()
                        .unwrap_or(0);
                    let translated = translator.translate(0, 5);
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        settings.translate(&w1, translated.0, translated.1);
                    }
                    println!(
                        "Latest drawn window index: {} -> translated ({}, {})",
                        latest_index,
                        translated.0,
                        translated.1
                    );

                    tab_held = false;
                }
                
                
                if key.eq("\u{f702}") && tab_held && !shift_held {
                    let latest_index = draw_tracker_for_input
                        .lock()
                        .unwrap()
                        .latest_window_drawn_index()
                        .unwrap_or(0);
                    let translated = translator.translate(-5, 0);
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        settings.translate(&w1, translated.0, translated.1);
                    }
                    println!(
                        "Latest drawn window index: {} -> translated ({}, {})",
                        latest_index,
                        translated.0,
                        translated.1
                    );

                    tab_held = false;
                }
                
                if key.eq("\u{f703}") && tab_held && !shift_held {
                    let latest_index = draw_tracker_for_input
                        .lock()
                        .unwrap()
                        .latest_window_drawn_index()
                        .unwrap_or(0);
                    let translated = translator.translate(5, 0);
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        settings.translate(&w1, translated.0, translated.1);
                    }
                    println!(
                        "Latest drawn window index: {} -> translated ({}, {})",
                        latest_index,
                        translated.0,
                        translated.1
                    );

                    tab_held = false;
                }

                // SCALING CHANGES
                // Up = grow taller, Down = grow shorter (vertical)
                // Right = grow wider, Left = grow narrower (horizontal)
                if key.eq("\u{f700}") && tab_held  && shift_held {
                    let latest_index = draw_tracker_for_input
                        .lock()
                        .unwrap()
                        .latest_window_drawn_index()
                        .unwrap_or(0);
                    let scaled = translator.scale(0, 5);
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        settings.scale(&w1, scaled.0, scaled.1);
                    }
                    println!(
                        "Latest drawn window index: {} -> scaled ({}, {})",
                        latest_index,
                        scaled.0,
                        scaled.1
                    );

                    tab_held = false;
                    shift_held = false;
                }
                if key.eq("\u{f701}") && tab_held && shift_held { 
                    let latest_index = draw_tracker_for_input
                        .lock()
                        .unwrap()
                        .latest_window_drawn_index()
                        .unwrap_or(0);
                    let scaled = translator.scale(0, -5);
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        settings.scale(&w1, scaled.0, scaled.1);
                    }
                    println!(
                        "Latest drawn window index: {} -> scaled ({}, {})",
                        latest_index,
                        scaled.0,
                        scaled.1
                    );

                    tab_held = false;
                    shift_held = false;
                }
                
                
                if key.eq("\u{f702}") && tab_held  && shift_held {
                    let latest_index = draw_tracker_for_input
                        .lock()
                        .unwrap()
                        .latest_window_drawn_index()
                        .unwrap_or(0);
                    let scaled = translator.scale(-5, 0);
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        settings.scale(&w1, scaled.0, scaled.1);
                    }
                    println!(
                        "Latest drawn window index: {} -> scaled ({}, {})",
                        latest_index,
                        scaled.0,
                        scaled.1
                    );

                    tab_held = false;
                    shift_held = false;
                }
                
                if key.eq("\u{f703}") && tab_held  && shift_held {
                    let latest_index = draw_tracker_for_input
                        .lock()
                        .unwrap()
                        .latest_window_drawn_index()
                        .unwrap_or(0);
                    let scaled = translator.scale(5, 0);
                    if let Some(w1) = weak_window_for_input.upgrade() && latest_index == 1 {
                        settings.scale(&w1, scaled.0, scaled.1);
                    }
                    else if let Some(w1) = weak_window_for_input.upgrade() && latest_index == 1 {
                        systeminfo.scale(&w1, scaled.0, scaled.1);
                    }
                    println!(
                        "Latest drawn window index: {} -> scaled ({}, {})",
                        latest_index,
                        scaled.0,
                        scaled.1
                    );

                    tab_held = false;
                    shift_held = false;
                }
                
                if taskbar.active() {
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        if taskbar.handle_key(&w1, key.as_str(), &mut settings, &mut systeminfo, &mut window2, &mut window3, &mut window4, &mut window5) {
                            let mut tracker = draw_tracker_for_input.lock().unwrap();
                            tracker.record_draw(1);
                            return;
                        }
                    }
                    return;
                }

                // Any non-shortcut key clears the Tab state so the
                // shortcut does not remain stuck after a single use.
                if tab_held {
                    tab_held = false;
                }

                // ─────────────────────────────────
                // DISCARD STRAY MODIFIER/CONTROL BYTES
                // ─────────────────────────────────
                //
                // Bare modifier keys (Shift, etc.) can emit raw
                // control characters on some platforms that aren't
                // meant to be sent to either shell.
                if key.chars().count() == 1 {
                    let c = key.chars().next().unwrap();
                    if (c as u32) < 0x20
                        && key != "\n"
                        && key != "\r"
                        && key != "\u{8}"
                        && key != "\u{1b}"
                    {
                        return;
                    }
                }

                // ─────────────────────────────────
                // SELECT ACTIVE PTY
                // ─────────────────────────────────
                //
                // Everything after this point gets sent
                // to whichever terminal currently owns
                // keyboard input.
                //
                let active_writer = if bubble.active() {
                    &bubble_writer
                } else {
                    &writer
                };

                let mut writer =
                    active_writer.lock().unwrap();

                // ─────────────────────────────────
                // CTRL + KEY
                // ─────────────────────────────────

                if control {
                    let key = key.to_ascii_lowercase();

                    if key.len() == 1 {
                        let byte = key.as_bytes()[0];

                        // Ctrl+A = 0x01
                        // Ctrl+B = 0x02
                        // ...
                        // Ctrl+C = 0x03
                        // Ctrl+Z = 0x1A
                        if (b'a'..=b'z').contains(&byte) {
                            let control_byte =
                                byte - b'a' + 1;

                            let _ =
                                writer.write_all(
                                    &[control_byte],
                                );

                            let _ =
                                writer.flush();

                            return;
                        }
                    }
                }

                // ─────────────────────────────────
                // SPECIAL KEYS
                // ─────────────────────────────────

                let bytes: Option<&[u8]> =
                    if key == "\n"
                        || key == "\r"
                    {
                        Some(b"\r")
                    }

                    else if key == "\u{8}" {
                        // Backspace
                        Some(b"\x7f")
                    }

                    else if key == "\u{1b}" {
                        // Escape
                        Some(b"\x1b")
                    }

                    else if key == "\u{f700}" {
                        // Up Arrow
                        Some(b"\x1b[A")
                    }

                    else if key == "\u{f701}" {
                        // Down Arrow
                        Some(b"\x1b[B")
                    }

                    else if key == "\u{f702}" {
                        // Left Arrow
                        Some(b"\x1b[D")
                    }

                    else if key == "\u{f703}" {
                        // Right Arrow
                        Some(b"\x1b[C")
                    }

                    else if key == "\u{f704}" {
                        // F1
                        Some(b"\x1bOP")
                    }

                    else if key == "\u{f705}" {
                        // F2
                        Some(b"\x1bOQ")
                    }

                    else if key == "\u{f706}" {
                        // F3
                        Some(b"\x1bOR")
                    }

                    else if key == "\u{f707}" {
                        // F4
                        Some(b"\x1bOS")
                    }

                    else if key == "\u{7f}" {
                        // Delete
                        Some(b"\x1b[3~")
                    }

                    else {
                        None
                    };

                // ─────────────────────────────────
                // SEND SPECIAL KEY
                // ─────────────────────────────────

                if let Some(bytes) = bytes {
                    let _ =
                        writer.write_all(bytes);

                    let _ =
                        writer.flush();

                    return;
                }

                // ─────────────────────────────────
                // NORMAL TEXT
                // ─────────────────────────────────

                if !key.is_empty() {
                    let _ =
                        writer.write_all(
                            key.as_bytes(),
                        );

                    let _ =
                        writer.flush();
                }
            },
        );
    }
    // ─────────────────────────────────────────────
    // DOCK ICON ACTIVATION
    // ─────────────────────────────────────────────
    // No apps are wired up yet, so activation echoes a
    // placeholder command into the main terminal for
    // visible feedback.
    {
        let writer = pty_writer.clone();
        let draw_tracker_for_icon = draw_tracker.clone();
        window.on_icon_activated(move |index| {
            {
                let mut tracker = draw_tracker_for_icon.lock().unwrap();
                tracker.record_draw(index as usize);
            }

            println!("Dock icon {} activated", index + 1);
            let mut writer = writer.lock().unwrap();
            let _ = writer.write_all(
                format!("echo \"[dock] app {}\"\r", index + 1).as_bytes(),
            );
            let _ = writer.flush();
        });
    }

    window.run()?;
    let _ = child.kill();
    let _ = bchild.kill();

    Ok(())
}