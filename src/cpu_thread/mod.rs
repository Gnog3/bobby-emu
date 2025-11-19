pub mod cpu;
mod instruction_formats;
mod memory;

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

use anyhow::Result;

use crate::cpu_thread::cpu::Cpu;

#[derive(Default, Clone, Copy)]
pub struct CpuState {
    pub registers: [u32; 32],
    pub pc: u32,
    pub insn_count: u64,
    pub fps: usize,
}

impl CpuState {
    pub const fn new() -> Self {
        CpuState {
            registers: [0; 32],
            pc: 0,
            insn_count: 0,
            fps: 0,
        }
    }
}

pub struct CpuHandle {
    stop_thread: Arc<AtomicBool>,
    request_update: Arc<AtomicBool>,
    cpu_state: Arc<Mutex<CpuState>>,
    thread_handle: Option<JoinHandle<(Cpu, Result<()>)>>,
    stopped_cpu: Option<Cpu>,
}

impl CpuHandle {
    /// Create a new CPU handle but do not start the CPU thread yet.
    pub fn new(cpu: Cpu) -> Self {
        let stop_thread = Arc::new(AtomicBool::new(false));
        let request_update = Arc::new(AtomicBool::new(false));
        let cpu_state = Arc::new(Mutex::new(CpuState::new()));

        Self {
            stop_thread,
            request_update,
            cpu_state,
            thread_handle: None,
            stopped_cpu: Some(cpu),
        }
    }

    pub fn start(&mut self) {
        if self.thread_handle.is_some() {
            return;
        }

        let Some(cpu) = self.stopped_cpu.take() else {
            return;
        };

        self.stop_thread.store(false, Ordering::Relaxed);

        let thread_handle = run_thread(
            cpu,
            Arc::clone(&self.stop_thread),
            Arc::clone(&self.request_update),
            Arc::clone(&self.cpu_state),
        );

        self.thread_handle = Some(thread_handle);
    }

    pub fn stop(&mut self) -> Result<()> {
        let Some(thread_handle) = self.thread_handle.take() else {
            return Ok(());
        };

        self.stop_thread.store(true, Ordering::Relaxed);

        let (cpu, result) = thread_handle.join().unwrap();
        self.stopped_cpu = Some(cpu);

        self.stop_thread.store(false, Ordering::Relaxed);

        result
    }

    pub fn request_stop(&self) {
        self.stop_thread.store(true, Ordering::Relaxed);
    }

    pub fn request_update(&self) {
        self.request_update.store(true, Ordering::Relaxed);
    }

    pub fn get_state(&self) -> CpuState {
        if let Some(cpu) = &self.stopped_cpu {
            return make_state(cpu);
        }

        *self.cpu_state.lock().unwrap()
    }
}

fn run_thread(
    mut cpu: Cpu,
    stop_thread: Arc<AtomicBool>,
    request_update: Arc<AtomicBool>,
    cpu_state: Arc<Mutex<CpuState>>,
) -> JoinHandle<(Cpu, Result<()>)> {
    std::thread::Builder::new()
        .name("cpu".into())
        .spawn(move || {
            loop {
                if stop_thread.load(Ordering::Relaxed) {
                    return (cpu, Ok(()));
                }

                if let Err(err) = cpu.tick() {
                    return (cpu, Err(err));
                }

                if request_update.swap(false, Ordering::Relaxed) {
                    *cpu_state.lock().unwrap() = make_state(&cpu);
                }
                std::thread::yield_now();
            }
        })
        .unwrap()
}

fn make_state(cpu: &Cpu) -> CpuState {
    CpuState {
        registers: cpu.registers,
        pc: cpu.pc,
        insn_count: cpu.insn_count,
        fps: cpu.fps,
    }
}
