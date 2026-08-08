use std::{
    io::{Read, Write},
    path::Path,
    sync::{Arc, Mutex},
    thread,
};
mod commands;

use commands::OriginShell;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};

slint::slint! {
component DockIcon inherits Rectangle {
    in property <bool> selected: false;
    width: 20px;
    height: 20px;
    border-radius: 5px;
    background: selected ? #00b3b3 : #008080;
}

export component MainWindow inherits Window {
title: "OrTerminal";
width: 1100px;
height: 650px;
background: #0c0c0c;

    default-font-family: "monospace";
    default-font-size: 15px;

    in property <string> terminal-text: "";
    in property <string> bubble-text: "origin-dev@OrOS-DEV:~$";
    in property <string> bubble-gen-text: "";
    in-out property <bool> shortcut_pressed: false;
    in-out property <bool> output_generated: false;
    // Rust receives every keyboard event directly.
    callback key-input(string, bool);

    // The dock is keyboard-driven: this OS has no mouse.
    in property <bool> dock-active: false;
    in property <bool> taskbar-active: false;
    in property <int> dock-selection: 0;
    in property <bool> qs_active: false;
    in property <bool> qsettings-active: false;
    in property <int> qs-selection: 0;

    callback icon-activated(int);

    // ─────────────────────────────────────────────
    // TITLE BAR
    // ─────────────────────────────────────────────

    Rectangle {
        x: 0px;
        y: 0px;
        width: parent.width;
        height: 28px;

        background: #1a1a1a;

        Text {
            text: "OrTerminal";
            color: #d4d4d4;
            font-size: 12px;

            width: parent.width;
            height: parent.height;

            horizontal-alignment: center;
            vertical-alignment: center;
        }
    }

    // ─────────────────────────────────────────────
    // ONE TERMINAL
    // ─────────────────────────────────────────────

    terminal := FocusScope {
        x: 0px;
        y: 28px;

        width: parent.width;
        height: parent.height - 28px;

        focus-on-click: true;

        // Grab keyboard focus when the terminal starts.
        init => {
            self.focus();
        }

        key-pressed(event) => {
            root.key-input(
                event.text,
                event.modifiers.control
            );
            accept
        }

        key-released(event) => {
            accept
        }

        // ─────────────────────────────────────────
        // Terminal screen
        // ─────────────────────────────────────────

        Flickable {
            x: 0px;
            y: 0px;

            width: parent.width;
            height: parent.height;

            viewport-width: self.width;
            viewport-height: self.height;

            Text {
                x: 12px;
                y: 12px;

                width: parent.width - 24px;
                height: parent.height - 24px;

                text: root.terminal-text;

                color: #d4d4d4;

                wrap: word-wrap;

                horizontal-alignment: left;
                vertical-alignment: top;
            }
        }
    }

    // ─────────────────────────────────────────────
    // COMMAND BUBBLE — a real secondary shell.
    // ─────────────────────────────────────────────
    // Top-level sibling (not nested inside FocusScope/Flickable)
    // so it's never clipped, and declared last so it always
    // paints on top. Height grows with content instead of being
    // pinned to one line, since it needs room to show output.
    bubble_label := Text {
        x: 725px;
        y: 85px;
        width: 110px;

        text: "COMMAND BUBBLE";
        color: #00b3b3;
        font-size: 11px;
        horizontal-alignment: center;
        visible: root.shortcut_pressed;
    }
    bubble := Rectangle {
        x: 650px;
        y: 100px;

        background: #000000dd;
        border-radius: 6px;
        border-width: 5px;
        border-color: #008080;
        width: max(text_box.preferred-width + 24px, 260px);
        height: max(text_box.preferred-height + 20px, 36px);
        visible: root.shortcut_pressed;

        text_box := Text {
            x: 12px;
            y: 10px;

            text: root.bubble-text;
            color: #d4d4d4;
            font-size: 15px;

            wrap: word-wrap;

            horizontal-alignment: left;
            vertical-alignment: top;
        }
    }
    bubble_output := Rectangle {
        x: 650px;
        y: 150px;

        background: #000000dd;
        border-radius: 6px;
        border-width: 5px;
        border-color: #008080;
        width: max(bubble_text_box.preferred-width + 24px, 260px);
        height: max(bubble_text_box.preferred-height + 20px, 36px);
        visible: root.output_generated && root.shortcut_pressed;

        bubble_text_box := Text {
            x: 12px;
            y: 10px;

            text: root.bubble-gen-text;
            color: #d4d4d4;
            font-size: 15px;

            wrap: word-wrap;

            horizontal-alignment: left;
            vertical-alignment: top;
        }
    }

    dock_label := Text {
        x: (parent.width - 100px) / 2;
        y: parent.height - 62px;
        width: 100px;

        text: "TASKBAR";
        color: #00b3b3;
        font-size: 11px;
        horizontal-alignment: center;
        visible: root.dock-active;
    }

    taskbar := Rectangle {
        x: (parent.width - 190px) / 2;
        y: parent.height - 45px;

        width: 190px;
        height: 36px;
        border-radius: 10px;
        border-width: root.dock-active ? 2px : 0px;
        border-color: #00b3b3;

        background: #1a1a1a;
        visible: root.taskbar-active;
        HorizontalLayout {
            padding: 8px;
            spacing: 4px;
            alignment: center;

            for idx in [0, 1, 2, 3, 4, 5, 6] : DockIcon {
                selected: root.dock-selection == idx;
            }
        }
    }

    taskbar_hidden := Rectangle {
        x: (parent.width - 190px) / 2;
        y: parent.height - 10px;
        
        width: 190px;
        height: 36px;
        border-radius: 10px;
        border-width: root.dock-active ? 2px : 0px;
        border-color: #00b3b3;

        background: #1a1a1a;
        visible: !root.taskbar-active;
    }

    qs_hidden := Rectangle {
        x: parent.width - 200px;
        y: 23px;

        width: 200px;
        height: 15px;
        border-width: root.qs_active ? 2px : 0px;
        border-radius:10px;
        border-color: #1a1a1a;

        background: #1a1a1a;
        visible: !root.qs-active;
    }

    qs := Rectangle {
        x: (parent.width - 200px);
        y: 28px;

        width: 200px;
        height: 100px;
        border-radius: 10px;
        border-width: root.qs-active ? 2px : 0px;
        border-color: #00b3b3;

        background: #1a1a1a;
        visible: root.qsettings-active;
        HorizontalLayout {
            padding: 8px;
            spacing: 4px;
            alignment: center;

            for idx in [0, 1, 2, 3, 4, 5, 6] : DockIcon {
                selected: root.qs-selection == idx;
            }
        }
    }
    qs_label := Text {
        x: (parent.width - 150px);
        y: 130px;
        width: 100px;

        text: "QUICK SETTINGS";
        color: #00b3b3;
        font-size: 11px;
        horizontal-alignment: center;
        visible: root.qsettings-active;
    }
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
    // OrTerminal does NOT draw one.
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

        let mut bubble_mode = false;
        let mut dock_mode = false;
        let mut dock_selection = 0usize;
        let mut tab_held = false;
        let mut quick_settings = false;
        let mut qs_selection = 0usize;

        window.on_key_input(
            move |key, control| {
                if key == "\u{0009}" {
                    tab_held = true;
                    return;
                }

                if key.eq_ignore_ascii_case("q") && tab_held {
                    bubble_mode = !bubble_mode;
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        w1.set_shortcut_pressed(bubble_mode);
                        if !bubble_mode {
                            w1.set_output_generated(false);
                            w1.set_bubble_gen_text(String::new().into());
                        }
                    } 
                    tab_held = false;

                    println!(
                        "Keyboard mode: {}",
                        if bubble_mode { "BUBBLE" } else { "TERMINAL" }
                    );
                    return;
                }

                if key.eq_ignore_ascii_case("s") && tab_held {
                    quick_settings = !quick_settings;
                    qs_selection = 0;
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        w1.set_qsettings_active(quick_settings);
                        w1.set_qs_active(quick_settings);
                        w1.set_qs_selection(qs_selection as i32);
                    }
                    tab_held = false;

                    println!(
                        "Keyboard mode: {}",
                        if quick_settings { "QSETTINGS" } else { "TERMINAL" }
                    );
                    return;
                }
                if quick_settings {
                    match key.as_str() {
                        "\u{f702}" => {
                            qs_selection = (qs_selection + 6) % 7;
                            if let Some(w1) = weak_window_for_input.upgrade() {
                                w1.set_qs_selection(qs_selection as i32);
                            }
                            return;
                        }
                        "\u{f703}" => {
                            qs_selection = (qs_selection + 1) % 7;
                            if let Some(w1) = weak_window_for_input.upgrade() {
                                w1.set_qs_selection(qs_selection as i32);
                            }
                            return;
                        }
                        "\n" | "\r" => {
                            let activated = qs_selection;
                            quick_settings = false;
                            if let Some(w1) = weak_window_for_input.upgrade() {
                                w1.set_qsettings_active(false);
                                w1.set_qs_active(false);
                                w1.invoke_icon_activated(activated as i32);
                            }
                            return;
                        }
                        _ => {
                            quick_settings = false;
                            if let Some(w1) = weak_window_for_input.upgrade() {
                                w1.set_qsettings_active(false);
                                w1.set_qs_active(false);
                            }
                            return;
                        }
                    }
                }

                if key.eq_ignore_ascii_case("t") && tab_held {
                    dock_mode = !dock_mode;
                    dock_selection = 0;
                    if let Some(w1) = weak_window_for_input.upgrade() {
                        w1.set_dock_active(dock_mode);
                        w1.set_taskbar_active(dock_mode);
                        w1.set_dock_selection(dock_selection as i32);
                    }
                    tab_held = false;

                    println!(
                        "Keyboard mode: {}",
                        if dock_mode { "DOCK" } else { "TERMINAL" }
                    );

                    return;
                }

                if dock_mode {
                    match key.as_str() {
                        "\u{f702}" => {
                            dock_selection = (dock_selection + 6) % 7;
                            if let Some(w1) = weak_window_for_input.upgrade() {
                                w1.set_dock_selection(dock_selection as i32);
                            }
                            return;
                        }
                        "\u{f703}" => {
                            dock_selection = (dock_selection + 1) % 7;
                            if let Some(w1) = weak_window_for_input.upgrade() {
                                w1.set_dock_selection(dock_selection as i32);
                            }
                            return;
                        }
                        "\n" | "\r" => {
                            let activated = dock_selection;
                            dock_mode = false;
                            if let Some(w1) = weak_window_for_input.upgrade() {
                                w1.set_dock_active(false);
                                w1.set_taskbar_active(false);
                                w1.invoke_icon_activated(activated as i32);
                            }
                            return;
                        }
                        _ => {
                            dock_mode = false;
                            if let Some(w1) = weak_window_for_input.upgrade() {
                                w1.set_dock_active(false);
                                w1.set_taskbar_active(false);
                            }
                            return;
                        }
                    }
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
                let active_writer = if bubble_mode {
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
        window.on_icon_activated(move |index| {
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