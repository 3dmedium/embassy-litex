#![no_std]

use riscv_rt::{core_interrupt, entry, exception};

use riscv::{
    interrupt::{Exception, Interrupt},
    result::*,
};

use core::arch::asm;

use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex ;

use crate::{serial::{self, println}, soc_headers};

type InterruptCallbackFunctionType = fn(ctx:*mut ())->();

struct InterruptCallbackType {
    function: InterruptCallbackFunctionType,
    context:*mut ()
}

pub struct ExternalInterrupt {
    pub functions:[InterruptCallbackType; 32]
}

pub static mut EXTERNAL_INTERRUPT:ExternalInterrupt = ExternalInterrupt { functions : [
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},

    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},

    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},

    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
    InterruptCallbackType {function:external_interrupt_default, context:0 as *mut ()},
] };





impl ExternalInterrupt {
    pub fn register_interrupt(num: usize, function:InterruptCallbackFunctionType, context:*mut ()){
        serial::print_fmt_func(format_args!("Interrupt registering {}\n", num));
        if num <= soc_headers::IRQ_NUM_MAX {
            unsafe {
                EXTERNAL_INTERRUPT.functions[num] = InterruptCallbackType { function, context };
            }
        }
    }


    
    pub fn execute_interrupt(num:usize){
        //serial::print_fmt_func(format_args!("Interrupt executing {}\n", num));
        if num <= soc_headers::IRQ_NUM_MAX {
            unsafe{
                (EXTERNAL_INTERRUPT.functions[num].function)(EXTERNAL_INTERRUPT.functions[num].context);
            }
        }
    }

    pub fn un_register_interrupt(num:usize) {
        //serial::print_fmt_func(format_args!("Interrupt de-registering {}\n", num));
        if num <= soc_headers::IRQ_NUM_MAX {
            unsafe {
                EXTERNAL_INTERRUPT.functions[num] = InterruptCallbackType { function:external_interrupt_default, context:0 as *mut ()};
            }
        }
    }

}
    
pub fn initialize () {
    unsafe {
        riscv::interrupt::enable();

        for n in 0..soc_headers::IRQ_NUM_MAX {
            //ExternalInterrupt::registerInterrupt(n, external_interrupt_default);
        }

        const CSR_IRQ_MASK: usize = 0xBC0;
        let mut mask:usize = 0xffff_ffff;
        
        asm!("csrw {csr}, {mask}", csr = const CSR_IRQ_MASK, mask = in(reg) mask);
        
        const CSR_IRQ_772:usize =0x304;
        mask=0x888;
        asm!("csrw {csr}, {mask}", csr = const CSR_IRQ_772, mask = in(reg) mask);

        //mask=0x800;
        //asm!("csrrs x0, {csr}, {mask}", csr = const CSR_IRQ_772, mask = in(reg) mask);
//
        //mask=0x8;
        //asm!("csrrs x0, {csr}, {mask}", csr = const CSR_IRQ_772, mask = in(reg) mask);

    }
}

#[allow(non_snake_case)]
#[export_name = "MachineExternalInterruptHandler"]
pub extern "Rust" fn MachineExternalInterruptHandler(level: u32, interrupt: usize) {
    use core::ptr;
    
    //serial::println("Machine External Interrupt");

    const CSR_IRQ_PENDING:usize = 0xFC0;
    // TODO execute for all bits
    let mut num:usize = 0;
    let mut mask:usize = 0;
    unsafe {
        asm!("csrr {mask}, {csr}", csr = const CSR_IRQ_PENDING, mask = out(reg) mask);
        //asm!("csrrc x0, mcause, {mask}", mask = inout(reg) mask);
    }
    //serial::print_fmt_func(format_args!("   Mask: {:08b}\n", mask));
    for n in 0..usize::BITS {
        num = n as usize;
        if ( mask & ( 0x1 << num) ) != 0{
            //serial::print_fmt_func(format_args!("   Execute: {:08b}\n", num));
            ExternalInterrupt::execute_interrupt(num);
        }
    }
}

pub fn external_interrupt_default(_:*mut ()){
}
