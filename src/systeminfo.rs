use crate::MainWindow;
use slint::ComponentHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use sysinfo::{
    Components, CpuRefreshKind, DiskKind, Disks, MemoryRefreshKind, ProcessRefreshKind,
    ProcessesToUpdate, RefreshKind, System,
};

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

fn fmt_gib(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / GIB)
}

fn fmt_disk_size(bytes: u64) -> String {
    let gib = bytes as f64 / GIB;
    if gib >= 1024.0 {
        format!("{:.1}T", gib / 1024.0)
    } else if gib >= 100.0 {
        format!("{:.0}G", gib)
    } else {
        format!("{:.1}G", gib)
    }
}

struct Snapshot {
    sys_os: String,
    sys_kernel: String,
    sys_host: String,
    sys_uptime: String,
    thermal_cpu: String,
    thermal_gpu: String,
    thermal_ssd: String,
    thermal_status: String,
    proc_header: String,
    proc_a: String,
    proc_b: String,
    proc_c: String,
    storage_header: String,
    storage_a: String,
    storage_b: String,
    storage_c: String,
    cpu_brand: String,
    cpu_cores: String,
    cpu_max_freq: String,
    cpu_cur_freq: String,
    mem_total: String,
    mem_avail: String,
    mem_used: String,
    mem_swap: String,
}

pub struct SystemInfo {
    active: bool,
    x_translation: i32,
    y_translation: i32,
    x_scale: i32,
    y_scale: i32,
    refresh_toggle: Arc<AtomicBool>,
}

impl SystemInfo {
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
        window.set_window_1_active(false);
        window.set_window_1_second_active(false);
    }

    pub fn toggle(&mut self, window: &MainWindow) {
        self.active = !self.active;
        window.set_window_1_active(self.active);
        window.set_window_1_second_active(false);
        if self.active {
            self.run_refresh(window);
        } else {
            self.refresh_toggle.store(false, Ordering::Relaxed);
        }
    }

    pub fn switch_to_page2(&mut self, window: &MainWindow) {
        window.set_window_1_active(true);
        window.set_window_1_second_active(true);
    }

    pub fn switch_to_page1(&mut self, window: &MainWindow) {
        window.set_window_1_active(true);
        window.set_window_1_second_active(false);
    }

    pub fn run_refresh(&mut self, window: &MainWindow) {
        self.refresh_toggle.store(true, Ordering::Relaxed);
        let weak = window.as_weak();
        let running = self.refresh_toggle.clone();
        thread::spawn(move || {
            // Keep a persistent System for process CPU usage so we don't need
            // to sleep `MINIMUM_CPU_UPDATE_INTERVAL` on every refresh. The
            // interval between `refresh_processes` calls (5 s) is the delta
            // window for `cpu_usage()` - first snapshot will report 0% which
            // is replaced on the next cycle without ever blocking the UI.
            let mut proc_system = System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
            );
            // Prime baseline so the next refresh 5 s later can compute a delta.
            proc_system.refresh_processes(ProcessesToUpdate::All, true);

            while running.load(Ordering::Relaxed) {
                // Heavy sysinfo work runs *here* (background thread), not on
                // the Slint UI thread - `window.set_*` is the only thing that
                // must happen on the UI thread.
                let snapshot =
                    SystemInfo::collect_snapshot(&proc_system);

                let _ = weak.upgrade_in_event_loop(move |w| {
                    SystemInfo::apply_snapshot(&w, snapshot);
                });

                thread::sleep(Duration::from_secs(5));

                if !running.load(Ordering::Relaxed) {
                    break;
                }
                proc_system.refresh_processes(ProcessesToUpdate::All, true);
            }
        });
    }

    // ---- snapshot: pure data collected off the UI thread ----

    fn collect_snapshot(proc_system: &System) -> Snapshot {
        let (sys_os, sys_kernel, sys_host, sys_uptime) = Self::collect_sys();
        let (thermal_cpu, thermal_gpu, thermal_ssd, thermal_status) =
            Self::collect_thermals();
        let (proc_header, proc_a, proc_b, proc_c) =
            Self::collect_processes(proc_system);
        let (storage_header, storage_a, storage_b, storage_c) = Self::collect_storage();
        let (cpu_brand, cpu_cores, cpu_max_freq, cpu_cur_freq) = Self::collect_cpu();
        let (mem_total, mem_avail, mem_used, mem_swap) = Self::collect_memory();
        Snapshot {
            sys_os,
            sys_kernel,
            sys_host,
            sys_uptime,
            thermal_cpu,
            thermal_gpu,
            thermal_ssd,
            thermal_status,
            proc_header,
            proc_a,
            proc_b,
            proc_c,
            storage_header,
            storage_a,
            storage_b,
            storage_c,
            cpu_brand,
            cpu_cores,
            cpu_max_freq,
            cpu_cur_freq,
            mem_total,
            mem_avail,
            mem_used,
            mem_swap,
        }
    }

    fn apply_snapshot(window: &MainWindow, s: Snapshot) {
        window.set_sys_os(s.sys_os.into());
        window.set_sys_kernel(s.sys_kernel.into());
        window.set_sys_host(s.sys_host.into());
        window.set_sys_uptime(s.sys_uptime.into());
        window.set_thermal_cpu(s.thermal_cpu.into());
        window.set_thermal_gpu(s.thermal_gpu.into());
        window.set_thermal_ssd(s.thermal_ssd.into());
        window.set_thermal_status(s.thermal_status.into());
        window.set_proc_header(s.proc_header.into());
        window.set_proc_a(s.proc_a.into());
        window.set_proc_b(s.proc_b.into());
        window.set_proc_c(s.proc_c.into());
        window.set_storage_header(s.storage_header.into());
        window.set_storage_a(s.storage_a.into());
        window.set_storage_b(s.storage_b.into());
        window.set_storage_c(s.storage_c.into());
        window.set_cpu_brand(s.cpu_brand.into());
        window.set_cpu_cores(s.cpu_cores.into());
        window.set_cpu_max_freq(s.cpu_max_freq.into());
        window.set_cpu_cur_freq(s.cpu_cur_freq.into());
        window.set_mem_total(s.mem_total.into());
        window.set_mem_avail(s.mem_avail.into());
        window.set_mem_used(s.mem_used.into());
        window.set_mem_swap(s.mem_swap.into());
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

    fn collect_thermals() -> (String, String, String, String) {
        let components = Components::new_with_refreshed_list();
        let mut cpu = String::from("CPU : N/A");
        let mut gpu = String::from("GPU : N/A");
        let mut disk = String::from("SSD : N/A");
        let mut hottest = 0.0f32;
        for component in &components {
            let temp = component.temperature().unwrap_or(0.0);
            let label = component.label().to_lowercase();
            if temp > hottest {
                hottest = temp;
            }
            if label.contains("cpu") {
                cpu = format!("CPU : {temp:.1} C");
            } else if label.contains("gpu") {
                gpu = format!("GPU : {temp:.1} C");
            } else if label.contains("ssd") || label.contains("nvme") || label.contains("disk") {
                disk = format!("SSD : {temp:.1} C");
            }
        }
        let status = if components.is_empty() {
            String::from("STATUS: N/A")
        } else if hottest >= 90.0 {
            String::from("STATUS: CRITICAL")
        } else if hottest >= 80.0 {
            String::from("STATUS: WARNING")
        } else {
            String::from("STATUS: NOMINAL")
        };
        (cpu, gpu, disk, status)
    }

    fn collect_processes(proc_system: &System) -> (String, String, String, String) {
        let mut procs: Vec<(u32, u32, f32, u64)> = proc_system
            .processes()
            .iter()
            .map(|(pid, process)| {
                (
                    pid.as_u32(),
                    process.parent().map(|p| p.as_u32()).unwrap_or(0),
                    process.cpu_usage(),
                    process.memory(),
                )
            })
            .collect();
        procs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        procs.truncate(3);
        let mut lines: Vec<String> = Vec::new();
        for (pid, parent, cpu, mem) in &procs {
            let mem_mb = *mem as f64 / (1024.0 * 1024.0);
            lines.push(format!("{pid:04} {parent:04}  {cpu:4.1}% {mem_mb:5.0}MB"));
        }
        while lines.len() < 3 {
            lines.push(String::from("..."));
        }
        (
            "PID  PARENT CPU    MEM".to_string(),
            lines[0].clone(),
            lines[1].clone(),
            lines[2].clone(),
        )
    }

    fn collect_storage() -> (String, String, String, String) {
        let disks = Disks::new_with_refreshed_list();
        let mut lines: Vec<String> = Vec::new();
        for disk in &disks {
            if lines.len() >= 3 {
                break;
            }
            let name = disk.name().to_string_lossy();
            let fs = disk.file_system().to_string_lossy();
            let kind = match disk.kind() {
                DiskKind::SSD => "SSD",
                DiskKind::HDD => "HDD",
                _ => "?",
            };
            lines.push(format!(
                "{name:<3} {fs:<5} {:>4} {:>4} {kind}",
                fmt_disk_size(disk.total_space()),
                fmt_disk_size(disk.available_space()),
            ));
        }
        while lines.len() < 3 {
            lines.push(String::from("..."));
        }
        (
            "DRV FS   CAP  FREE  TYPE".to_string(),
            lines[0].clone(),
            lines[1].clone(),
            lines[2].clone(),
        )
    }

    fn collect_cpu() -> (String, String, String, String) {
        let system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );
        let cpus = system.cpus();
        let logical = cpus.len();
        let physical = System::physical_core_count().unwrap_or(0);
        let brand = cpus.first().map(|cpu| cpu.brand()).unwrap_or("N/A");
        let max_freq = cpus.iter().map(|cpu| cpu.frequency()).max().unwrap_or(0);
        let cur_freq = cpus.first().map(|cpu| cpu.frequency()).unwrap_or(0);
        (
            format!("CPU : {brand}"),
            format!("CORE: {physical} phys / {logical} logical"),
            format!("FREQ: {:.2} GHz max", max_freq as f64 / 1000.0),
            format!("CUR : {:.2} GHz", cur_freq as f64 / 1000.0),
        )
    }

    fn collect_memory() -> (String, String, String, String) {
        let system = System::new_with_specifics(
            RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
        );
        let total = system.total_memory();
        let avail = system.available_memory();
        let used = system.used_memory();
        let total_swap = system.total_swap();
        let used_swap = system.used_swap();
        (
            format!("TOTAL: {} GB", fmt_gib(total)),
            format!("AVAIL: {} GB", fmt_gib(avail)),
            format!("USED : {} GB", fmt_gib(used)),
            format!("SWAP : {} / {} GB", fmt_gib(used_swap), fmt_gib(total_swap)),
        )
    }

    pub fn translate(&mut self, window: &MainWindow, x: i32, y: i32) {
        self.x_translation = x;
        self.y_translation = y;
        window.set_window_1_x_translation(x);
        window.set_window_1_y_translation(y);
    }

    pub fn scale(&mut self, window: &MainWindow, x: i32, y: i32) {
        self.x_scale = x;
        self.y_scale = y;
        window.set_window_1_x_scale(x);
        window.set_window_1_y_scale(y);
    }

    pub fn get_sys(window: &MainWindow) {
        let (os, kernel, host, uptime) = Self::collect_sys();
        window.set_sys_os(os.into());
        window.set_sys_kernel(kernel.into());
        window.set_sys_host(host.into());
        window.set_sys_uptime(uptime.into());
    }

    pub fn get_processes(window: &MainWindow) {
        // Legacy sync path - non-blocking. Accurate CPU% is handled by the
        // background `run_refresh` thread with a persistent System; calling
        // this directly does a single refresh (cpu will be 0 on first call).
        let system = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
        );
        // `system` is freshly created and already contains one refresh of
        // process data; we intentionally do NOT sleep here - blocking the
        // UI thread was the cause of window1 open jank.
        let (header, a, b, c) = Self::collect_processes(&system);
        window.set_proc_header(header.into());
        window.set_proc_a(a.into());
        window.set_proc_b(b.into());
        window.set_proc_c(c.into());
    }

    pub fn get_storage(window: &MainWindow) {
        let (header, a, b, c) = Self::collect_storage();
        window.set_storage_header(header.into());
        window.set_storage_a(a.into());
        window.set_storage_b(b.into());
        window.set_storage_c(c.into());
    }

    pub fn get_cpu(window: &MainWindow) {
        let (brand, cores, max_freq, cur_freq) = Self::collect_cpu();
        window.set_cpu_brand(brand.into());
        window.set_cpu_cores(cores.into());
        window.set_cpu_max_freq(max_freq.into());
        window.set_cpu_cur_freq(cur_freq.into());
    }

    pub fn get_memory(window: &MainWindow) {
        let (total, avail, used, swap) = Self::collect_memory();
        window.set_mem_total(total.into());
        window.set_mem_avail(avail.into());
        window.set_mem_used(used.into());
        window.set_mem_swap(swap.into());
    }
    pub fn get_thermals(window: &MainWindow) {
        let (cpu, gpu, ssd, status) = Self::collect_thermals();
        window.set_thermal_cpu(cpu.into());
        window.set_thermal_gpu(gpu.into());
        window.set_thermal_ssd(ssd.into());
        window.set_thermal_status(status.into());
    }

}
