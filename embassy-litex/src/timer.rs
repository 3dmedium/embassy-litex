#![no_std]


use core::task::Waker;
use core::cell::RefCell;

use crate::{soc_headers::CSR_CPU_TIMER_LATCH_ADDR, interrupt};
use crate::soc_headers::{CSR_CPU_TIMER_TIME_CMP_LSB, CSR_CPU_TIMER_TIME_CMP_MSB, CSR_CPU_TIMER_TIME_LSB, CSR_CPU_TIMER_TIME_MSB, IRQ_NUM_TIMER0};

use crate::serial::println;
use crate::serial;

use crate::register::Register;

use embassy_time_driver::{Driver, TICK_HZ};
use embassy_time_queue_utils::Queue;

use critical_section::{CriticalSection, Mutex};

pub struct Timer {
    pub registers: TimerRegisters,
    pub initialized:bool
}

struct TimerRegisters {
    #[doc = "0x00 - Load value when Timer is (re-)enabled. In One-Shot mode, the value written to this register specifies the Timer's duration in clock cycles."]
    pub load: Register,
    #[doc = "0x04 - Reload value when Timer reaches ``0``. In Periodic mode, the value written to this register specify the Timer's period in clock cycles."]
    pub reload: Register,
    #[doc = "0x08 - Enable flag of the Timer. Set this flag to ``1`` to enable/start the Timer. Set to ``0`` to disable the Timer."]
    pub en: Register,
    #[doc = "0x0c - Update trigger for the current countdown value. A write to this register latches the current countdown value to ``value`` register."]
    pub update_value: Register,
    #[doc = "0x10 - Latched countdown value. This value is updated by writing to ``update_value``."]
    pub value: Register,
    #[doc = "0x14 - This register contains the current raw level of the zero event trigger. Writes to this register have no effect."]
    pub ev_status: Register,
    #[doc = "0x18 - When a zero event occurs, the corresponding bit will be set in this register. To clear the Event, set the corresponding bit in this register."]
    pub ev_pending: Register,
    #[doc = "0x1c - This register enables the corresponding zero events. Write a ``0`` to this register to disable individual events."]
    pub ev_enable: Register,
    #[doc = "0x20 - Write a ``1`` to latch current Uptime cycles to ``uptime_cycles`` register."]
    pub uptime_latch: Register,
    #[doc = "0x24 - Bits 32-63 of `TIMER0_UPTIME_CYCLES`. Latched Uptime since power-up (in ``sys_clk`` cycles)."]
    pub uptime_cycles1: Register,
    #[doc = "0x28 - Bits 0-31 of `TIMER0_UPTIME_CYCLES`."]
    pub uptime_cycles0: Register,
}


impl Timer {
    
    pub const fn create_timer(base_addr:u32) -> Timer {
        let mut s = Timer { initialized:false, registers :TimerRegisters { 
            load:           Register { addr: base_addr + 0x00 }, 
            reload:         Register { addr: base_addr + 0x04 }, 
            en:             Register { addr: base_addr + 0x08 }, 
            update_value:   Register { addr: base_addr + 0x0c }, 
            value:          Register { addr: base_addr + 0x10 }, 
            ev_status:      Register { addr: base_addr + 0x14 }, 
            ev_pending:     Register { addr: base_addr + 0x18 }, 
            ev_enable:      Register { addr: base_addr + 0x1c },
            uptime_latch:   Register { addr: base_addr + 0x20 },
            uptime_cycles1: Register { addr: base_addr + 0x24 },
            uptime_cycles0: Register { addr: base_addr + 0x28 }
        }};
        if base_addr != 0 {
            s.initialized  =true;
            //s.registers.reload.write(0);
            //s.registers.en.write(0);
        }
        
        return s;
    }

    pub fn enable_irq(&self){
        //println("TIMER enable irq");
        //self.registers.en.write(0);
        //self.registers.reload.write(0);
        //interrupt::ExternalInterrupt::register_interrupt(IRQ_NUM_TIMER0, external_interrupt_timer, 0 as *mut ());
        //self.registers.ev_pending.write(u32::MAX);
        //self.registers.ev_enable.write(1);
    }

    pub fn stop(&self){

    }

    pub fn start(&self, timeout:u32){
        //println("TIMER start");
        self.registers.en.write(0);
        self.registers.reload.write(0);
        self.registers.load.write(timeout);
        self.registers.en.write(1);
        self.registers.update_value.write(1);
    }


    pub fn uptime_clk(&self)->u64 {
        self.registers.uptime_latch.write(0x01);
        let msb:u32 = self.registers.uptime_cycles1.read();
        let lsb:u32 = self.registers.uptime_cycles0.read();
        //serial::print_fmt_func( format_args!("now: {}  ,   {}\r\n",msb,lsb));
        let time:u64 = ((msb as u64) << 32) + (lsb as u64);
        //serial::print_fmt_func( format_args!("now: {}  \r\n",time));
        return time;
    }

    pub fn uptime_micros(&self)->u64 {
        let clk = self.uptime_clk();
        let micros:u64 = (clk *TICK_HZ) / 1_000_000;
        return micros;
    }


}



struct VexRiscvTimer {
    pub latch:Register,
    pub time_lsb:Register,
    pub time_msb:Register,
    pub compare_lsb:Register,
    pub compare_msb:Register,
}

impl VexRiscvTimer {
    pub const fn create() -> Self {
        return Self {
            latch : Register {addr:CSR_CPU_TIMER_LATCH_ADDR},
            time_lsb: Register {addr: CSR_CPU_TIMER_TIME_LSB },
            time_msb: Register {addr:CSR_CPU_TIMER_TIME_MSB},
            compare_lsb:Register {addr:CSR_CPU_TIMER_TIME_CMP_LSB},
            compare_msb:Register {addr: CSR_CPU_TIMER_TIME_CMP_MSB}
        }
    }

    pub fn set(&self, end:u64){
       
        let lsb:u32 = end as u32;
        let msb:u32 = (end >> 32) as u32;
        self.compare_lsb.write(lsb);
        self.compare_msb.write(msb);
        self.latch.write(1);

    }

    pub fn stop(&self){
        self.compare_lsb.write(u32::MAX);
        self.compare_msb.write(u32::MAX);
    }


    pub fn uptime_clk(&self)->u64 {
        self.latch.write(1);

        let msb:u32 = self.time_msb.read();
        let lsb:u32 = self.time_lsb.read();
        //serial::print_fmt_func( format_args!("now: {}  ,   {}\r\n",msb,lsb));
        let time:u64 = ((msb as u64) << 32) + (lsb as u64);
        //serial::print_fmt_func( format_args!("vex now: {}  \r\n",time));
        return time;
    }

    pub fn uptime_micros(&self)->u64 {
        let clk = self.uptime_clk();
        let micros:u64 = (clk *TICK_HZ) / 1_000_000;
        return micros;
    }

    
}

static INTERNAL_TIMER:VexRiscvTimer = VexRiscvTimer::create();




struct TimeDriver {
    queue: Mutex<RefCell<Queue>>,
}

impl TimeDriver {
    fn set_alarm(&self, cs: &CriticalSection, timestamp: u64) -> bool {
        if timestamp > self.now() {
            INTERNAL_TIMER.set(timestamp);
            return true;
        }else{
            return false;
        }
    }

    pub fn on_interrupt(&self){
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
            let mut next = queue.next_expiration(self.now());
            //serial::println("next");
            while !self.set_alarm(&cs, next) {
                //serial::println("nextloop");
                next = queue.next_expiration(self.now());
            }
        });
    }
}

impl Driver for TimeDriver {
    fn now(&self) -> u64 {
        return INTERNAL_TIMER.uptime_clk();
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        critical_section::with(|cs| {
            let mut queue = self.queue.borrow(cs).borrow_mut();
            if queue.schedule_wake(at, waker) {
                let mut next = queue.next_expiration(self.now());
                use core::ptr;
                while !self.set_alarm(&cs, next) {
                    next = queue.next_expiration(self.now());
                }
            }
        });
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: TimeDriver = TimeDriver{ queue: Mutex::new(RefCell::new(Queue::new())) });






#[allow(non_snake_case)]
#[export_name = "MachineTimerInterruptHandler"]
pub extern "Rust" fn MachineTimerInterruptHandler(level: u32, interrupt: usize) {
    DRIVER.on_interrupt();
}
