use std::{
    sync::{
        atomic::{AtomicBool, Ordering}, Arc, Mutex
    },
    thread::JoinHandle, time::Duration,
};

use anyhow::Result;

use crate::cpu::Cpu;

pub static STOP: AtomicBool = AtomicBool::new(false);

pub fn run(cpu: Arc<Mutex<Cpu>>) -> JoinHandle<Result<()>> {
    std::thread::Builder::new()
        .name("cpu".into())
        .spawn(move || {
            loop {
                if STOP.load(Ordering::SeqCst) {
                    return Ok(());
                }
                {
                    let result = std::panic::catch_unwind(|| {
                        let mut cpu = cpu.lock().unwrap();
                        cpu.tick().unwrap();
                    });
                    if let Err(err) = result {
                        match cpu.lock() {
                            Ok(cpu) => eprintln!("panic at pc: {:x}", cpu.pc),
                            Err(poison) => eprintln!("panic at pc: {:x}", poison.into_inner().pc),
                        }
                        std::panic::resume_unwind(err);
                    }
                }
                //std::thread::sleep(Duration::from_micros(5));
                std::thread::yield_now();
            }
        })
        .unwrap()
}
