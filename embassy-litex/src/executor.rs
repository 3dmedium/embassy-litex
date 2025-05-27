//! Interrupt-mode executor.

use core::{array, usize};
use core::{cell::UnsafeCell, mem::MaybeUninit};

use critical_section;
use embassy_executor::SendSpawner;
use embassy_executor::Spawner;
use embassy_executor::raw;

use crate::soc_headers;
use crate::soc_headers::CSR_CPU_SOFTWARE_INTERRUPT_REQUEST_INTERRUPT_ADDR;
use crate::register::Register;
use crate::serial;
use crate::serial::print_fmt_func;


pub use embassy_litex_macros::main as main;









/// Interrupt mode executor.
///
/// This executor runs tasks in interrupt mode. The interrupt handler is set up
/// to poll tasks, and when a task is woken the interrupt is pended from
/// software.
pub struct InterruptExecutor {
    pub executor: UnsafeCell<MaybeUninit<raw::Executor>>,
    started: Mutex<Cell<bool>>,
    context :usize
}

// TODO: initialize with function
const MAX_LEVELS:usize = 16;
pub static EXECUTOR_LIST: [InterruptExecutor;MAX_LEVELS] = [
    InterruptExecutor::new( 0 ),
    InterruptExecutor::new( 1 ),
    InterruptExecutor::new( 2 ),
    InterruptExecutor::new( 3 ),

    InterruptExecutor::new( 4 ),
    InterruptExecutor::new( 5 ),
    InterruptExecutor::new( 6 ),
    InterruptExecutor::new( 7 ),

    InterruptExecutor::new( 8 ),
    InterruptExecutor::new( 9 ),
    InterruptExecutor::new( 10 ),
    InterruptExecutor::new( 11 ),

    InterruptExecutor::new( 12 ),
    InterruptExecutor::new( 13 ),
    InterruptExecutor::new( 14 ),
    InterruptExecutor::new( 15 ),
];



use core::cell::{Cell};
use critical_section::Mutex;

use portable_atomic::{AtomicU32, Ordering};


unsafe impl Send for InterruptExecutor {}
unsafe impl Sync for InterruptExecutor {}

struct ExtendedTrapFrame {

}

impl InterruptExecutor {
    /// Create a new `InterruptExecutor`.
    /// This takes the software interrupt to be used internally.
    #[inline]
    pub const fn new(ctx:usize) -> Self {
        Self {
            executor: UnsafeCell::new(MaybeUninit::uninit()),
            started: Mutex::new(Cell::new(false)),
            context:ctx,
        }
    }

    pub fn on_interrupt(&'static self) {
        // Floating point implementation on public litex repository missing
        //unsafe {
        //    core::arch::asm!(
        //        "addi	sp, sp, -176",
        //
        //        "fsd f0, 0(sp)",
        //        "fsd f1, 8(sp)",
        //        "fsd f2, 16(sp)",
        //        "fsd f3, 24(sp)",
        //        "fsd f4, 32(sp)",
        //        "fsd f5, 40(sp)",
        //        "fsd f6, 48(sp)",
        //        "fsd f7, 56(sp)",
        //        "fsd f10, 64(sp)",
        //        "fsd f11, 72(sp)",
        //        "fsd f12, 80(sp)",
        //        "fsd f13, 88(sp)",
        //        "fsd f14, 96(sp)",
        //        "fsd f15, 104(sp)",
        //        "fsd f16, 112(sp)",
        //        "fsd f17, 120(sp)",
        //        "fsd f28, 128(sp)",
        //        "fsd f29, 136(sp)",
        //        "fsd f30, 144(sp)",
        //        "fsd f31, 152(sp)",
        //
        //        "frcsr t0",
        //        "sw t0,  160(sp)",
        //
        //        out("t0") _,
        //        
        //    );
        //}
        unsafe {
            riscv::interrupt::enable();
        }
        let executor = unsafe { (&*self.executor.get()).assume_init_ref() };
        unsafe {
            executor.poll();
        }
        unsafe {
            riscv::interrupt::disable();
        }
        //unsafe {
        //    core::arch::asm!(
        //        "fld f0, 0(sp)",
        //        "fld f1, 8(sp)",
        //        "fld f2, 16(sp)",
        //        "fld f3, 24(sp)",
        //        "fld f4, 32(sp)",
        //        "fld f5, 40(sp)",
        //        "fld f6, 48(sp)",
        //        "fld f7, 56(sp)",
        //        "fld f10, 64(sp)",
        //        "fld f11, 72(sp)",
        //        "fld f12, 80(sp)",
        //        "fld f13, 88(sp)",
        //        "fld f14, 96(sp)",
        //        "fld f15, 104(sp)",
        //        "fld f16, 112(sp)",
        //        "fld f17, 120(sp)",
        //        "fld f28, 128(sp)",
        //        "fld f29, 136(sp)",
        //        "fld f30, 144(sp)",
        //        "fld f31, 152(sp)",
        //
        //        "lw t0,  160(sp)",
        //        "fscsr t0",
        //        
        //        "addi	sp, sp, 176",
        //        
        //        out("t0") _,
        //    );
        //}
    }

    /// Start the executor at the given priority level.
    ///
    /// This initializes the executor, enables the interrupt, and returns.
    /// The executor keeps running in the background through the interrupt.
    ///
    /// This returns a [`SendSpawner`] you can use to spawn tasks on it. A
    /// [`SendSpawner`] is returned instead of a
    /// [`Spawner`](embassy_executor::Spawner) because the
    /// executor effectively runs in a different "thread" (the interrupt),
    /// so spawning tasks on it is effectively sending them.
    ///
    /// To obtain a [`Spawner`](embassy_executor::Spawner) for this executor,
    /// use [`Spawner::for_current_executor`](embassy_executor::Spawner::for_current_executor)
    /// from a task running in it.
    pub fn start(&'static self) -> SendSpawner {
        if critical_section::with(|cs| self.started.borrow(cs).replace(true)) {
            panic!("InterruptExecutor::start() called multiple times on the same executor.");
        }

        unsafe {
            (&mut *self.executor.get())
                .as_mut_ptr()
                .write(raw::Executor::new(self.context as *mut ()))
        }

        let executor = unsafe { (&*self.executor.get()).assume_init_ref() };
        print_fmt_func(format_args!("Executor {} initialized...\n",self.context ));
        executor.spawner().make_send()
    }

    /// Get a SendSpawner for this executor
    ///
    /// This returns a [`SendSpawner`] you can use to spawn tasks on this
    /// executor.
    ///
    /// This MUST only be called on an executor that has already been started.
    /// The function will panic otherwise.
    pub fn send_spawner(&'static self) -> SendSpawner {
        if !critical_section::with(|cs| self.started.borrow(cs).get()) {
            panic!("InterruptExecutor::spawner() called on uninitialized executor.");
        }
        let executor = unsafe { (&*self.executor.get()).assume_init_ref() };
        executor.spawner().make_send()
    }


    pub fn spawner(&'static self) -> Spawner {
        if !critical_section::with(|cs| self.started.borrow(cs).get()) {
            panic!("InterruptExecutor::spawner() called on uninitialized executor.");
        }
        let executor = unsafe { (&*self.executor.get()).assume_init_ref() };
        executor.spawner()
    }





}


const SOFTWARE_INTERRUPT_REQUEST_REGISTER:Register = Register { addr: soc_headers::CSR_CPU_SOFTWARE_INTERRUPT_REQUEST_INTERRUPT_ADDR };
const SOFTWARE_INTERRUPT_ACTIVE_REGISTER:Register = Register { addr: soc_headers::CSR_CPU_SOFTWARE_INTERRUPT_ACTIVE_INTERRUPT_ADDR };
const SOFTWARE_INTERRUPT_STATUS_REGISTER:Register = Register { addr: soc_headers::CSR_CPU_SOFTWARE_INTERRUPT_STATUS_INTERRUPT_ADDR };


const CSR_OPERATION_SET:u32 = 0x8000_0000;
const CSR_OPERATION_CLEAR:u32 = 0x4000_0000;
const CSR_OPERATION_REPLACE:u32 = 0x0000_0000;

#[export_name = "__pender"]
fn __pender(context: *mut ()) {
    let context = context as usize;
    //serial::print_fmt_func(format_args!("EXE call pender: {}\r\n", context));
    if context < MAX_LEVELS{
        let mut mask = CSR_OPERATION_SET + (0x1 << context);
        SOFTWARE_INTERRUPT_REQUEST_REGISTER.write(mask);
    }
}

#[allow(non_snake_case)]
#[export_name = "MachineSoftInterruptHandler"]
pub extern "Rust" fn MachineSoftInterruptHandler(level: u32, interrupt: usize) {
    // get Executor number from CSR

    let mut num: usize = usize::MAX;

    critical_section::with(|_| {
        // get the highest set bit from the interrupt register
        // read Status
        num = SOFTWARE_INTERRUPT_STATUS_REGISTER.read() as usize;

        if num <= MAX_LEVELS {
            SOFTWARE_INTERRUPT_REQUEST_REGISTER.write(CSR_OPERATION_CLEAR + (0x1 << num));
            SOFTWARE_INTERRUPT_REQUEST_REGISTER.write(CSR_OPERATION_CLEAR + (0x1 << num));
        }

    });

    if num <= MAX_LEVELS {

            SOFTWARE_INTERRUPT_ACTIVE_REGISTER.write(CSR_OPERATION_SET + (0x1 << num));
            EXECUTOR_LIST[num].on_interrupt();
            SOFTWARE_INTERRUPT_ACTIVE_REGISTER.write(CSR_OPERATION_CLEAR + (0x1 << num));
        
    }
    
}