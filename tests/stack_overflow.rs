#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;

use rust_os_journey::{exit_qemu, serial_print, serial_println, QemuExitCode};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    rust_os_journey::gdt::init();
    init_test_idt();

    // trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os_journey::test_panic_handler(info)
}
use core::ops::{Deref, DerefMut};

#[derive(Copy, Clone)]
struct Dummy(u8);

impl Deref for Dummy {
    type Target = Dummy;
    fn deref(&self) -> &Self::Target {
        self
    }
}

impl DerefMut for Dummy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(Dummy(0)).read(); // prevent tail recursion optimizations
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(rust_os_journey::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    rust_os_journey::hlt_loop();
}
