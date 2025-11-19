pub mod cpu;
mod instruction_formats;
mod memory;

use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
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

pub static STOP_THREAD: AtomicBool = AtomicBool::new(false);
pub static RUN: AtomicBool = AtomicBool::new(false);
pub static REQUEST_UPDATE: AtomicBool = AtomicBool::new(false);
pub static CPU_STATE: Mutex<CpuState> = Mutex::new(CpuState::new());

pub fn run(mut cpu: Cpu) -> JoinHandle<Result<()>> {
    std::thread::Builder::new()
        .name("cpu".into())
        .spawn(move || {
            loop {
                if STOP_THREAD.load(Ordering::Relaxed) {
                    return Ok(());
                }
                if RUN.load(Ordering::Relaxed) {
                    cpu.tick().unwrap();
                } else {
                    std::thread::sleep(Duration::from_millis(1));
                }

                if REQUEST_UPDATE.swap(false, Ordering::Relaxed) {
                    let cpu_state = CpuState {
                        registers: cpu.registers,
                        pc: cpu.pc,
                        insn_count: cpu.insn_count,
                        fps: cpu.fps,
                    };
                    *CPU_STATE.lock().unwrap() = cpu_state;
                }
                //std::thread::sleep(Duration::from_micros(5));
                std::thread::yield_now();
            }
        })
        .unwrap()
}
