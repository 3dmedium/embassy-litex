use core::{arch::asm, ptr};

use crate::{ serial::print_fmt_func};

use crate::soc_headers;

pub struct Register {
    pub addr: u32
}

pub fn create_register(addr:u32) -> Register {
    if addr == 0 {
        panic!("Creating Register at address: 0");
    }
    return Register { addr: addr };
}

impl Register {


    pub fn write_offset(&self, offset:u32,data:u32) {
        unsafe {
            ptr::write_volatile((self.addr + offset) as *mut u32, data);
        }
    }
    pub fn read_offset(&self, offset:u32) -> u32 {
        unsafe {
            return ptr::read_volatile((self.addr + offset) as *mut u32);
        }
    }

    pub fn write(&self,data:u32) {
        unsafe {
            ptr::write_volatile((self.addr) as *mut u32, data);
        }
    }
    pub fn read(&self) -> u32 {
        unsafe {
            return ptr::read_volatile((self.addr) as *mut u32);
        }
    }
}


const CACHE_BYTES_PER_LINE:usize = 32;

pub fn flush_data_cache(start_addr:usize, size:usize){

    // TODO maybe optimize this with compare of a bitmask

    let mem_start = soc_headers::MEM_MAIN_RAM_BASE_ADDR as usize;
    let mem_end = (soc_headers::MEM_MAIN_RAM_BASE_ADDR + soc_headers::MEM_MAIN_RAM_SIZE) as usize; // mem end shows one address outside of mem

    //let mem_start = 0x4000_0000;
    //let mem_end = 0x4080_0000;
    //
    //print_fmt_func(format_args!("Memory: {:#08x}   ---   {:#08x}\n", mem_start,mem_end));

    if   start_addr >= mem_start && start_addr < mem_end
                    && start_addr + size < mem_end {
                        // TODO Better check ! checks if full range is in mem, should flush only memory wich is in mem
    //    // Address is in valid range
        let count = size/CACHE_BYTES_PER_LINE;
        for i in 0..count {
            let addr = start_addr + i*CACHE_BYTES_PER_LINE;
//

            //const INSTR_FLUSH_ADDR_FROM_T0:usize = 0x500F | (0x5 << 15);
            unsafe { asm!(
                "mv t0, {0}",
                ".word(0x2D00F)",
                in(reg) addr,
                options(nomem, nostack),
            )}
//

            //unsafe { asm!(
            //    ".word(0x500F)"
            //)}
        }
    //    
//
    }
}