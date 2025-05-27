use crate::soc_headers;
use crate::register::Register;

use core::fmt::Write;
use core::fmt::Arguments;
use core::fmt;


use core::future::Future;
use core::task::Context;
use core::task::Poll;
use critical_section::Mutex;
use embassy_time::Duration;
use core::cell::RefCell;
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt;


use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex ;

pub struct Serial {
    irq_enabled:bool,
    irq_num:u32,
    irq_signal:Signal<CriticalSectionRawMutex , u32>,
    base_addr:u32,
    pub registers: UartRegisters,
}
pub const SERIAL_EVENT_TX:u32 = 0x1;
pub const SERIAL_EVENT_RX:u32 = 0x2;

pub struct UartRegisters {
        #[doc = "0x00 - "]
        pub rxtx: Register,
        #[doc = "0x04 - TX FIFO Full."]
        pub txfull: Register,
        #[doc = "0x08 - RX FIFO Empty."]
        pub rxempty: Register,
        #[doc = "0x0c - This register contains the current raw level of the rx event trigger. Writes to this register have no effect."]
        pub ev_status: Register,
        #[doc = "0x10 - When a rx event occurs, the corresponding bit will be set in this register. To clear the Event, set the corresponding bit in this register."]
        pub ev_pending: Register,
        #[doc = "0x14 - This register enables the corresponding rx events. Write a ``0`` to this register to disable individual events."]
        pub ev_enable: Register,
        #[doc = "0x18 - TX FIFO Empty."]
        pub txempty: Register,
        #[doc = "0x1c - RX FIFO Full."]
        pub rxfull: Register
}


impl Serial {

    // TODO implement without IRQ
    
    pub fn create_serial(base_addr:u32, irq_enabled:bool, irq_num:u32) -> Serial {
        let mut s = Serial { 
            irq_enabled,
            irq_num,
            irq_signal: Signal::new(),
            base_addr: base_addr,
            registers :UartRegisters { 
                rxtx:       Register { addr: base_addr + 0x00 }, 
                txfull:     Register { addr: base_addr + 0x04 }, 
                rxempty:    Register { addr: base_addr + 0x08 }, 
                ev_status:  Register { addr: base_addr + 0x0c }, 
                ev_pending: Register { addr: base_addr + 0x10 }, 
                ev_enable:  Register { addr: base_addr + 0x14 }, 
                txempty:    Register { addr: base_addr + 0x18 }, 
                rxfull:     Register { addr: base_addr + 0x1c }
            }
        };
        
        return s;
    }

    pub fn clean_boot(&self){
        self.registers.ev_pending.write(SERIAL_EVENT_RX);
        self.registers.ev_pending.write(SERIAL_EVENT_TX);
        self.registers.ev_enable.write(0);
    }

    pub fn init (&self){
        let p = self.registers.ev_pending.read();
        self.registers.ev_pending.write(p);

        // TODO: Serial needs to live forever !
        let ptr = self as *const Self as *mut Self;
        let ptr = ptr as *mut ();
        
        interrupt::ExternalInterrupt::register_interrupt(self.irq_num as usize, external_interrupt_uart, ptr as *mut ());
        self.registers.ev_enable.write(SERIAL_EVENT_RX);
    }

    pub fn putc(&self, c: u8) {
        // TODO, buffer in Memory and wait for interrupt...
        while self.registers.txfull.read() != 0 {
            ()
        }
        self.registers.rxtx.write(c as u32);
    }

    pub fn getc(&self) -> (bool,u8) {
        if self.registers.rxempty.read() == 1 {
            return (false,0);
        }
        return (true,self.registers.rxtx.read() as u8);
    }

    pub async fn read_wait(&self) -> u8 {
        if self.registers.rxempty.read() == 0 {
            let mut c:u8 = 0;    
            c = self.registers.rxtx.read() as u8;
            self.registers.ev_pending.write(SERIAL_EVENT_RX);
            return c;
        }
        let val = self.irq_signal.wait().await as u8;
        return val;
    }

    pub fn write(&self, s: &str){
        for c in s.bytes() {
            self.putc(c);
        }
    }
}

struct SerialFormat {
    dummy:u32,
}
impl Write for SerialFormat {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print(s);
        Ok(())
    }
}

unsafe extern {
    static SUPERVISOR_SERIAL_BASE_ADDR:u32;
    static SUPERVISOR_SERIAL_IRQ_NUM:usize;
}

use embassy_sync::lazy_lock::LazyLock;
static SUPERVISOR_SERIAL_FORMATTER: embassy_sync::blocking_mutex::Mutex<CriticalSectionRawMutex,RefCell<SerialFormat>> = embassy_sync::blocking_mutex::Mutex::new( RefCell::new(SerialFormat {dummy:0} ));
static SUPERVISOR_SERIAL: LazyLock<Serial> = LazyLock::new(|| Serial::create_serial(unsafe { SUPERVISOR_SERIAL_BASE_ADDR }, true, unsafe { SUPERVISOR_SERIAL_IRQ_NUM as u32 }) );



pub fn init () {
    SUPERVISOR_SERIAL.get().init();
}


pub fn write(s:u8){
    critical_section::with(|cs| {
        SUPERVISOR_SERIAL.get().putc(s);
    });
}

pub fn print(s:&str){
    critical_section::with(|cs| {
        SUPERVISOR_SERIAL.get().write(s);
    });
}


pub async fn read() -> u8 {
    // TODO only one reader allowed at a time !
    SUPERVISOR_SERIAL.get().read_wait().await 
}

pub fn println(s:&str){
    print(s);
    print("\n");
}

#[export_name = "print_fmt_func"]
pub fn print_fmt_func(args: Arguments<'_>) {
    SUPERVISOR_SERIAL_FORMATTER.lock( |fmt| {fmt.borrow_mut().write_fmt(args);} );
}


fn external_interrupt_uart(ctx:*mut ()) {

    let serial = unsafe { &*(ctx as *const Serial) };

    //println("serial interr");
    let pending = serial.registers.ev_pending.read();
    if (pending & SERIAL_EVENT_RX) != 0 {
        // read
        if serial.registers.rxempty.read() == 0 {
            let mut c:u8 = 0;    
            c = serial.registers.rxtx.read() as u8;
            serial.registers.ev_pending.write(SERIAL_EVENT_RX);
            serial.irq_signal.signal(c as u32);
        }
    }
    
}




use log::{Level, Log, Metadata, Record, SetLoggerError};

pub static LOGGER: SerialLogger = SerialLogger {};

pub struct SerialLogger {}

impl Log for SerialLogger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        match record.level() {
            Level::Error => print("\u{001b}[31m LOG:  Error  : "),
            Level::Warn =>  print("\u{001b}[33m LOG:  Warn   : "),
            Level::Info =>  print( "\u{001b}[0m LOG:  Info   : "),
            Level::Debug => print("\u{001b}[35m LOG:  Debug  : "),
            Level::Trace => print("\u{001b}[32m LOG:  Trace  : "),
        }
        print_fmt_func(format_args!("{}",record.args()));
        print("\u{001b}[0m\n");
    }

    fn flush(&self) {}
}