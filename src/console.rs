/* Very basic console implementation
 * Uses naive spinlock (no protection against interrupts)
 * Need to analyze what ordering should be
 * also does not work if write!() is called with arguments, is should be an easy enough fix
 */

use crate::uart;
use core::fmt::Error;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Console {
    uart: Option<uart::Uart>,
    locked: AtomicBool,
}

static mut CONSOLE: Console = {
    Console {
        uart: None,
        locked: AtomicBool::new(false),
    }
};

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

impl Console {
    pub unsafe fn get() -> &'static mut Self {
        if CONSOLE.uart.is_none() {
            while CONSOLE
                .locked
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                // spin
            }
            // lock acquired
            assert_eq!(
                CONSOLE.locked.load(Ordering::SeqCst),
                true,
                "lock not held before release"
            );
            // check again since we read without aquiring the lock
            if CONSOLE.uart.is_none() {
                let mut uart = uart::Uart::new(0x1000_0000);
                // need to understand what init does
                uart.init();
                CONSOLE.uart = Some(uart);
            }
            assert_eq!(
                CONSOLE.locked.load(Ordering::SeqCst),
                true,
                "lock not held before release"
            );
            CONSOLE.locked.store(false, Ordering::Release);
        }
        &mut CONSOLE
    }
}

impl Write for Console {
    fn write_str(&mut self, out: &str) -> Result<(), Error> {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // write!(self.uart.as_mut().unwrap(), "locked").unwrap();
        }
        assert_eq!(
            self.locked.load(Ordering::SeqCst),
            true,
            "lock not held before release"
        );
        let res = write!(self.uart.as_mut().unwrap(), "{}", out);
        assert_eq!(
            self.locked.load(Ordering::SeqCst),
            true,
            "lock not held before release"
        );
        self.locked.store(false, Ordering::Release);
        // write!(self.uart, "lock released");
        res
    }
}
