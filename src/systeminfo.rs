use crate::MainWindow;
use slint::ComponentHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use sysinfo::{
    Components, Cpu, CpuRefreshKind, Disk, DiskKind, DiskRefreshKind, Disks, MemoryRefreshKind,
    Networks, Pid, Process, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System,
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
    }

    pub fn toggle(&mut self, window: &MainWindow) {
        self.active = !self.active;
        window.set_window_1_active(self.active);
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
                let _ = weak.upgrade_in_event_loop(move |w| {
                    SystemInfo::refresh_all(&w);
                });
                thread::sleep(Duration::from_secs(5));
            }
        });
    }

    fn refresh_all(window: &MainWindow) {
        SystemInfo::get_sys(window);
        SystemInfo::get_thermals(window);
        SystemInfo::get_processes(window);
        SystemInfo::get_storage(window);
        SystemInfo::get_cpu(window);
        SystemInfo::get_memory(window);
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
        let os = System::name().unwrap_or_else(|| "N/A".into());
        let kernel = System::kernel_version().unwrap_or_else(|| "N/A".into());
        let host = System::host_name().unwrap_or_else(|| "N/A".into());

        let uptime = System::uptime();
        let days = uptime / 86400;
        let hours = (uptime % 86400) / 3600;
        let mins = (uptime % 3600) / 60;

        window.set_sys_os(format!("OS  : {os}").into());
        window.set_sys_kernel(format!("KERN: {kernel}").into());
        window.set_sys_host(format!("HOST: {host}").into());
        window.set_sys_uptime(format!("UP  : {days}d {hours}h {mins}m").into());
    }

    pub fn get_processes(window: &MainWindow) {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
        );
        thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_processes(ProcessesToUpdate::All, true);

        let mut procs: Vec<(u32, u32, f32, u64)> = system
            .processes()
            .into_iter()
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

        window.set_proc_header("PID  PARENT CPU    MEM".into());
        window.set_proc_a(lines[0].clone().into());
        window.set_proc_b(lines[1].clone().into());
        window.set_proc_c(lines[2].clone().into());
    }

    pub fn get_storage(window: &MainWindow) {
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

        window.set_storage_header("DRV FS   CAP  FREE  TYPE".into());
        window.set_storage_a(lines[0].clone().into());
        window.set_storage_b(lines[1].clone().into());
        window.set_storage_c(lines[2].clone().into());
    }

    pub fn get_cpu(window: &MainWindow) {
        let system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );

        let cpus = system.cpus();
        let logical = cpus.len();
        let physical = System::physical_core_count().unwrap_or(0);
        let brand = cpus.first().map(|cpu| cpu.brand()).unwrap_or("N/A");
        let max_freq = cpus.iter().map(|cpu| cpu.frequency()).max().unwrap_or(0);
        let cur_freq = cpus.first().map(|cpu| cpu.frequency()).unwrap_or(0);

        window.set_cpu_brand(format!("CPU : {brand}").into());
        window.set_cpu_cores(format!("CORE: {physical} phys / {logical} logical").into());
        window
            .set_cpu_max_freq(format!("FREQ: {:.2} GHz max", max_freq as f64 / 1000.0).into());
        window
            .set_cpu_cur_freq(format!("CUR : {:.2} GHz", cur_freq as f64 / 1000.0).into());
    }

    pub fn get_memory(window: &MainWindow) {
        let system = System::new_with_specifics(
            RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
        );

        let total = system.total_memory();
        let avail = system.available_memory();
        let used = system.used_memory();
        let total_swap = system.total_swap();
        let used_swap = system.used_swap();

        window.set_mem_total(format!("TOTAL: {} GB", fmt_gib(total)).into());
        window.set_mem_avail(format!("AVAIL: {} GB", fmt_gib(avail)).into());
        window.set_mem_used(format!("USED : {} GB", fmt_gib(used)).into());
        window.set_mem_swap(
            format!("SWAP : {} / {} GB", fmt_gib(used_swap), fmt_gib(total_swap)).into(),
        );
    }
    pub fn get_thermals(window: &MainWindow) {
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

        window.set_thermal_cpu(cpu.into());
        window.set_thermal_gpu(gpu.into());
        window.set_thermal_ssd(disk.into());
        window.set_thermal_status(status.into());
    }

}
