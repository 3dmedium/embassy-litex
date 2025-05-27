#![no_std]
#![no_main]

use log::{info, trace, warn, error, debug};

extern crate riscv;


use embassy_executor::{raw::task_from_waker, SendSpawner, Spawner};
use embassy_litex::interrupt;
use embassy_litex::soc_headers;
use embassy_litex::serial::{self, print_fmt_func, println} ;

use embassy_time::{Duration, Timer, Instant};

extern crate alloc;
use embedded_alloc::LlffHeap as Heap;
//
#[global_allocator]
static HEAP: Heap = Heap::empty();


// use for macros
use embassy_litex::executor::EXECUTOR_LIST;



#[no_mangle]
pub static SUPERVISOR_SERIAL_BASE_ADDR:u32 = soc_headers::CSR_UART_BASE_ADDR;
#[no_mangle]
pub static SUPERVISOR_SERIAL_IRQ_NUM:usize = soc_headers::IRQ_NUM_UART;




#[embassy_executor::task]
async fn blink() {    
        serial::println("Blink Timer starting...");
        loop {
            Timer::after(Duration::from_secs(1)).await;
            serial::println("Blink...");
        }
        
}


#[embassy_executor::task]
async fn processing() {
    loop {
        Timer::after(Duration::from_secs(1)).await;
        println("calculating......");


        let mut a = 0.1;
        for i in 0..10000 {
            a+= 0.01;
        }

        print_fmt_func(format_args!("Result: {}\n",a));

    }
        
        
}





#[embassy_litex::executor::main]
async fn main()  {

    log::set_logger(&serial::LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace));

    serial::println("Booting rust app...");

    serial::init();

    extern {
        static _stack_start: *mut u8;
        static __sstack: *mut u8;
        static __sheap: *mut u8;
        static __eheap: *mut u8;
        
    }

    serial::print_fmt_func(format_args!("Stack: {:#08x}    {:#08x}\n", core::ptr::addr_of!(_stack_start) as usize, core::ptr::addr_of!(__sstack) as usize));
    serial::print_fmt_func(format_args!("Heap: {:#08x}    {:#08x}\n", core::ptr::addr_of!(__sheap) as usize, core::ptr::addr_of!(__eheap) as usize));

    unsafe {

        extern {
            static __sheap: *mut u8;
            static __eheap: *mut u8;
        }
        let start = core::ptr::addr_of!(__sheap) as usize;
        let size: usize = core::ptr::addr_of!(__eheap) as usize - core::ptr::addr_of!(__sheap) as usize;
        serial::print_fmt_func(format_args!("Creating HEAP at: {:#08x}   size: {:#08x}\n", start,size ));
        unsafe { HEAP.init(start, size) }
    }

    serial::println("Heap initialized...");


    let spawner = EXECUTOR_LIST[10].start() ;
    spawner.must_spawn(blink());

    let spawner =  EXECUTOR_LIST[3].start() ;
    spawner.must_spawn(processing());


    loop {
        let a = serial::read().await;
        serial::write(a);

    }

    

    
}