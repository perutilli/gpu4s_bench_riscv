// assembly.rs
// Assembly imports module
// Stephen Marz
// 20 April 2020

// This came from the Rust book documenting global_asm!.
// They show using include_str! with it to
// import a full assembly file, which is what I want here.
use core::arch::global_asm;

#[cfg(feature = "sequential")]
global_asm!(include_str!("asm/boot_single_hart.s"));
#[cfg(any(
    feature = "parallel",
    not(any(feature = "sequential", feature = "parallel"))
))]
global_asm!(include_str!("asm/boot.s"));

global_asm!(include_str!("asm/mem.s"));
global_asm!(include_str!("asm/trap.s"));
