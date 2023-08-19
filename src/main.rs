#![no_std]
#![no_main]

const N_HARTS: usize = 4;

use core::panic::PanicInfo;
// right now panic only halts one hart => to change this we should use some interrupt probably
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    abort();
}

#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

#[cfg_attr(
    feature = "matrix_multiplication",
    path = "benchmarks/matrix_multiplication.rs"
)]
#[cfg_attr(
    not(feature = "matrix_multiplication"),
    path = "benchmarks/convolution.rs"
)]
mod benchmark;

pub mod assembly;
pub mod console;
pub mod uart;

pub mod matrix;
pub mod shared_matrix;

// move to a CLINT module
use core::time::Duration;
pub fn time() -> Duration {
    let mtime = 0x200_BFF8 as *const u64;
    Duration::from_nanos(unsafe { mtime.read_volatile() } * 100)
}
