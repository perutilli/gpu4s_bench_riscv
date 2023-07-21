#![no_std]
#![no_main]

use core::arch::asm;

extern crate panic_halt;

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!(unsafe {crate::console::Console::get()}, $($args)+);
        });
}

#[macro_export]
macro_rules! println
{
    () => ({
        print!("\r\n")
    });
    ($fmt:expr) => ({
        print!(concat!($fmt, "\r\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\r\n"), $($args)+)
    });
}

#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

#[no_mangle]
extern "C" fn main_init() {
    println!("Hello from the main hart!");

    // Now, we can send an interrupt to the other harts, by setting msip to 1
    unsafe {
        asm!("li t3, (1 << 3)");
        asm!("csrw mie, t3");
    }

    // spin forever
    unsafe {
        asm!("wfi");
    }
}

#[no_mangle]
extern "C" fn hart_init(_hartid: usize) {
    println!("Hello from secondary hart!");

    // spin forever
    unsafe {
        asm!("wfi");
    }
}

pub mod assembly;
pub mod console;
pub mod uart;
