use crate::MainWindow;
use slint::ComponentHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use sysinfo::System;

struct SettingsSnapshot {
    sys_os: String,
    sys_kernel: String,
    sys_host: String,
    sys_uptime: String,
}

pub struct SettingsWindow {
    active: bool,
    x_translation: i32,
    y_translation: i32,
    x_scale: i32,
    y_scale: i32,
    refresh_toggle: Arc<AtomicBool>,
}

impl SettingsWindow {
    pub fn new() -> Self {
        Self {
            active: false,
            x_translation: 0,
            y_translation: 0,
            x_scale: 0,
            y_scale: 0,
            refresh_toggle: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn close(&mut self, window: &MainWindow) {
        self.active = false;
        self.refresh_toggle.store(false, Ordering::Relaxed);
        window.set_settings_window_active(false);
    }

    pub fn toggle(&mut self, window: &MainWindow) {
        self.active = !self.active;
        window.set_settings_window_active(self.active);
        if self.active {
            self.run_refresh(window);
        } else {
            self.refresh_toggle.store(false, Ordering::Relaxed);
        }
    }

    pub fn run_refresh(&mut self, window: &MainWindow) {
        self.refresh_toggle.store(true, Ordering::Relaxed);
        let weak = window.as_weak();
        let running = self.refresh_toggle.clone();
        thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                // Heavy sysinfo reads run *here* (background thread), not on
                // the Slint UI thread - `window.set_*` is the only thing that
                // must happen on the UI thread. Mirrors SystemInfo::collect_snapshot.
                let snapshot = SettingsWindow::collect_snapshot();

                let _ = weak.upgrade_in_event_loop(move |w| {
                    SettingsWindow::apply_snapshot(&w, snapshot);
                });

                thread::sleep(Duration::from_secs(5));

                if !running.load(Ordering::Relaxed) {
                    break;
                }
            }
        });
    }

    // ---- snapshot: pure data collected off the UI thread ----

    fn collect_snapshot() -> SettingsSnapshot {
        let (sys_os, sys_kernel, sys_host, sys_uptime) = Self::collect_sys();
        SettingsSnapshot {
            sys_os,
            sys_kernel,
            sys_host,
            sys_uptime,
        }
    }

    fn apply_snapshot(window: &MainWindow, s: SettingsSnapshot) {
        window.set_settings_sys_os(s.sys_os.into());
        window.set_settings_sys_kernel(s.sys_kernel.into());
        window.set_settings_sys_host(s.sys_host.into());
        window.set_settings_sys_uptime(s.sys_uptime.into());
    }

    fn collect_sys() -> (String, String, String, String) {
        let os = System::name().unwrap_or_else(|| "N/A".into());
        let kernel = System::kernel_version().unwrap_or_else(|| "N/A".into());
        let host = System::host_name().unwrap_or_else(|| "N/A".into());
        let uptime = System::uptime();
        let days = uptime / 86400;
        let hours = (uptime % 86400) / 3600;
        let mins = (uptime % 3600) / 60;
        (
            format!("OS  : {os}"),
            format!("KERN: {kernel}"),
            format!("HOST: {host}"),
            format!("UP  : {days}d {hours}h {mins}m"),
        )
    }

    // Legacy sync path - non-blocking helper for callers that need a
    // one-shot refresh without starting the background loop.
    pub fn get_sys(window: &MainWindow) {
        let (os, kernel, host, uptime) = Self::collect_sys();
        window.set_settings_sys_os(os.into());
        window.set_settings_sys_kernel(kernel.into());
        window.set_settings_sys_host(host.into());
        window.set_settings_sys_uptime(uptime.into());
    }

    pub fn translate(&mut self, window: &MainWindow, x: i32, y: i32) {
        self.x_translation = x;
        self.y_translation = y;
        window.set_settings_window_x_translation(x);
        window.set_settings_window_y_translation(y);
    }

    pub fn scale(&mut self, window: &MainWindow, x: i32, y: i32) {
        self.x_scale = x;
        self.y_scale = y;
        window.set_settings_window_x_scale(x);
        window.set_settings_window_y_scale(y);
    }
}
