
use core::arch::asm;

use crate::serial::{self, print_fmt_func};



#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    riscv::interrupt::disable();

    serial::println("");
    serial::println("");
    serial::println("");
    serial::println("-------------- Panic ------------");
    serial::print_fmt_func(format_args!("{}\r\n", _info));
    serial::println("Stopping CPU.");
    serial::println("Trace:");

    let mut fp = unsafe {
        let mut _tmp: u32;
        asm!("mv {0}, x8", out(reg) _tmp);
        _tmp
    };

    let mut sp = unsafe {
        let mut _tmp: u32;
        asm!("mv {0}, x2", out(reg) _tmp);
        _tmp
    };

    

    loop {

        if !is_addr_aligned(fp) {
            serial::print_fmt_func(format_args!("Address {:08x} unaligned...", fp));
            break;
        }
    
        if !is_addr_valid(fp) {
            serial::print_fmt_func(format_args!("Address {:08x} not in mem...",fp));
            break;
        }

        // RA/PC
        let address = unsafe { (fp as *const u32).offset(-1).read_volatile() };
        // next FP
        fp = unsafe { (fp as *const u32).offset(-2).read_volatile() };

        if address == 0 {
            serial::println("PC Addr == 0 ....");
            break;
        }


        serial::print_fmt_func(format_args!("PC ADDR: {:08x}",address));

        let raw_ptr = address as *mut TrapFrame;
        let rust_reference: &mut TrapFrame = unsafe{ raw_ptr.as_mut().unwrap() };
        rust_reference.print();
        
    }
    




    loop {}
}




fn is_addr_aligned(address: u32) -> bool {
    if (address & 0xF) != 0 {
        return false;
    }
    return true;
}

fn is_addr_valid(address: u32) -> bool {
    // TODO implement auto 
    if !(0x1000_0000..=0x1002_0000).contains(&address) {
        return false;
    }

    true
}


pub struct TrapFrame {
    /// Return address, stores the address to return to after a function call or
    /// interrupt.
    pub ra: usize,
    /// Temporary register t0, used for intermediate values.
    pub t0: usize,
    /// Temporary register t1, used for intermediate values.
    pub t1: usize,
    /// Temporary register t2, used for intermediate values.
    pub t2: usize,
    /// Temporary register t3, used for intermediate values.
    pub t3: usize,
    /// Temporary register t4, used for intermediate values.
    pub t4: usize,
    /// Temporary register t5, used for intermediate values.
    pub t5: usize,
    /// Temporary register t6, used for intermediate values.
    pub t6: usize,
    /// Argument register a0, typically used to pass the first argument to a
    /// function.
    pub a0: usize,
    /// Argument register a1, typically used to pass the second argument to a
    /// function.
    pub a1: usize,
    /// Argument register a2, typically used to pass the third argument to a
    /// function.
    pub a2: usize,
    /// Argument register a3, typically used to pass the fourth argument to a
    /// function.
    pub a3: usize,
    /// Argument register a4, typically used to pass the fifth argument to a
    /// function.
    pub a4: usize,
    /// Argument register a5, typically used to pass the sixth argument to a
    /// function.
    pub a5: usize,
    /// Argument register a6, typically used to pass the seventh argument to a
    /// function.
    pub a6: usize,
    /// Argument register a7, typically used to pass the eighth argument to a
    /// function.
    pub a7: usize,
    /// Saved register s0, used to hold values across function calls.
    pub s0: usize,
    /// Saved register s1, used to hold values across function calls.
    pub s1: usize,
    /// Saved register s2, used to hold values across function calls.
    pub s2: usize,
    /// Saved register s3, used to hold values across function calls.
    pub s3: usize,
    /// Saved register s4, used to hold values across function calls.
    pub s4: usize,
    /// Saved register s5, used to hold values across function calls.
    pub s5: usize,
    /// Saved register s6, used to hold values across function calls.
    pub s6: usize,
    /// Saved register s7, used to hold values across function calls.
    pub s7: usize,
    /// Saved register s8, used to hold values across function calls.
    pub s8: usize,
    /// Saved register s9, used to hold values across function calls.
    pub s9: usize,
    /// Saved register s10, used to hold values across function calls.
    pub s10: usize,
    /// Saved register s11, used to hold values across function calls.
    pub s11: usize,
    /// Global pointer register, holds the address of the global data area.
    pub gp: usize,
    /// Thread pointer register, holds the address of the thread-local storage
    /// area.
    pub tp: usize,
    /// Stack pointer register, holds the address of the top of the stack.
    pub sp: usize,
    /// Program counter, stores the address of the next instruction to be
    /// executed.
    pub pc: usize,
    /// Machine status register, holds the current status of the processor,
    /// including interrupt enable bits and privilege mode.
    pub mstatus: usize,
    /// Machine cause register, contains the reason for the trap (e.g.,
    /// exception or interrupt number).
    pub mcause: usize,
    /// Machine trap value register, contains additional information about the
    /// trap (e.g., faulting address).
    pub mtval: usize,
}

impl TrapFrame {
    pub fn print (&self){
        print_fmt_func(format_args!("
TrapFrame
PC=0x{:08x}         RA/x1=0x{:08x}      SP/x2=0x{:08x}      GP/x3=0x{:08x}      TP/x4=0x{:08x}
T0/x5=0x{:08x}      T1/x6=0x{:08x}      T2/x7=0x{:08x}      S0/FP/x8=0x{:08x}   S1/x9=0x{:08x}
A0/x10=0x{:08x}     A1/x11=0x{:08x}     A2/x12=0x{:08x}     A3/x13=0x{:08x}     A4/x14=0x{:08x}
A5/x15=0x{:08x}     A6/x16=0x{:08x}     A7/x17=0x{:08x}     S2/x18=0x{:08x}     S3/x19=0x{:08x}
S4/x20=0x{:08x}     S5/x21=0x{:08x}     S6/x22=0x{:08x}     S7/x23=0x{:08x}     S8/x24=0x{:08x}
S9/x25=0x{:08x}     S10/x26=0x{:08x}    S11/x27=0x{:08x}    T3/x28=0x{:08x}     T4/x29=0x{:08x}
T5/x30=0x{:08x}     T6/x31=0x{:08x}

MSTATUS=0x{:08x}
MCAUSE=0x{:08x}
MTVAL=0x{:08x}
",
self.pc,
self.ra,
self.gp,
self.sp,
self.tp,
self.t0,
self.t1,
self.t2,
self.s0,
self.s1,
self.a0,
self.a1,
self.a2,
self.a3,
self.a4,
self.a5,
self.a6,
self.a7,
self.s2,
self.s3,
self.s4,
self.s5,
self.s6,
self.s7,
self.s8,
self.s9,
self.s10,
self.s11,
self.t3,
self.t4,
self.t5,
self.t6,
self.mstatus,
self.mcause,
self.mtval,));
    }
}





// Executor trace
#[no_mangle]
fn _embassy_trace_poll_start(executor_id: u32){
    print_fmt_func(format_args!("_embassy_trace_poll_start {:#08x}\n",executor_id));
}

#[no_mangle]
fn _embassy_trace_task_new(executor_id: u32, task_id: u32){
    print_fmt_func(format_args!("_embassy_trace_task_new {:#08x}, {:#08x}\n",executor_id, task_id));
}

#[no_mangle]
fn _embassy_trace_task_end(executor_id: u32, task_id: u32){
    print_fmt_func(format_args!("_embassy_trace_task_new {:#08x}, {:#08x}\n",executor_id, task_id));
}

#[no_mangle]
fn _embassy_trace_task_exec_begin(executor_id: u32, task_id: u32){
    print_fmt_func(format_args!("_embassy_trace_task_new {:#08x}, {:#08x}\n",executor_id, task_id));
}

#[no_mangle]
fn _embassy_trace_task_exec_end(executor_id: u32, task_id: u32){
    print_fmt_func(format_args!("_embassy_trace_task_new {:#08x}, {:#08x}\n",executor_id, task_id));
}

#[no_mangle]
fn _embassy_trace_task_ready_begin(executor_id: u32, task_id: u32){
    print_fmt_func(format_args!("_embassy_trace_task_new {:#08x}, {:#08x}\n",executor_id, task_id));
}

#[no_mangle]
fn _embassy_trace_executor_idle(executor_id: u32){
    print_fmt_func(format_args!("_embassy_trace_poll_start {:#08x}\n",executor_id));
}