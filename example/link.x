



EXTERN(_start_trap);

EXTERN(ExceptionHandler);
EXTERN(DefaultHandler);

PROVIDE(_start_DefaultHandler_trap = _start_trap);


PROVIDE(_stext = ORIGIN(REGION_TEXT));
PROVIDE(_max_hart_id = 0);


PROVIDE(_hart_stack_size = 1M);
PROVIDE(_stack_size = 2M);
PROVIDE(_heap_size = 1M);


SECTIONS
{
  .text.dummy (NOLOAD) :
  {
    /* This section is intended to make _stext address work */
    . = ABSOLUTE(_stext);
  } > REGION_TEXT

  .text _stext :
  {
    __stext = .;

    /* Put reset handler first in .text section so it ends up as the entry */
    /* point of the program. */
    KEEP(*(.init));
    . = ALIGN(8);
    KEEP(*(.init.trap));
    . = ALIGN(8);
    *(.trap);
    *(.trap.rust);
    *(.text.abort);
    *(.text .text.*);

    . = ALIGN(8);
    __etext = .;
  } > REGION_TEXT

  .rodata : ALIGN(8)
  {
     . = ALIGN(8);
    __srodata = .;

    *(.srodata .srodata.*);
    *(.rodata .rodata.*);

    /* 4-byte align the end (VMA) of this section.
       This is required by LLD to ensure the LMA of the following .data
       section will have the correct alignment. */
    . = ALIGN(8);
    __erodata = .;
  } > REGION_RODATA

  .data : ALIGN(8)
  {
    . = ALIGN(8);
    __sdata = .;

    /* Must be called __global_pointer$ for linker relaxations to work. */
    PROVIDE(__global_pointer$ = . + 0x800);
    *(.sdata .sdata.* .sdata2 .sdata2.*);
    *(.data .data.*);

  } > REGION_DATA AT > REGION_RODATA
  
  /* Allow sections from user `memory.x` injected using `INSERT AFTER .data` to
   * use the .data loading mechanism by pushing __edata. Note: do not change
   * output region or load region in those user sections! */
  . = ALIGN(8);
  __edata = .;
  
  /* LMA of .data */
  __sidata = LOADADDR(.data);

  .bss : ALIGN(8)
  {
    . = ALIGN(8);
    __sbss = .;

    *(.sbss .sbss.* .bss .bss.*);
  } > REGION_BSS

  /* Allow sections from user `memory.x` injected using `INSERT AFTER .bss` to
   * use the .bss zeroing mechanism by pushing __ebss. Note: do not change
   * output region or load region in those user sections! */
  . = ALIGN(8);
  __ebss = .;


/* fictitious region that represents the memory available for the stack */
  .stack (NOLOAD) : ALIGN(1M)
  {
    __estack = .;
    _stack_start = .;
    . += _stack_size;
    . = ALIGN(8);
    __sstack = .;
  } > REGION_STACK


      /* fictitious region that represents the memory available for the heap */
  .heap (NOLOAD) : ALIGN(1M)
  {
    __sheap = .;
    . += _heap_size;
    . = ALIGN(8);
    __eheap = .;
  } > REGION_HEAP

  


  /* fake output .got section */
  /* Dynamic relocations are unsupported. This section is only used to detect
     relocatable code in the input files and raise an error if relocatable code
     is found */
  .got (INFO) :
  {
    KEEP(*(.got .got.*));
  }


}








/* # CORE INTERRUPT HANDLERS DESCRIBED IN THE STANDARD RISC-V ISA
   
   If the `no-interrupts` feature is DISABLED, this file will be included in link.x.in.
   If the `no-interrupts` feature is ENABLED, this file will be ignored.
*/

/* It is possible to define a special handler for each interrupt type.
   By default, all interrupts are handled by DefaultHandler. However, users can
   override these alias by defining the symbol themselves */
PROVIDE(SupervisorSoft = DefaultHandler);
PROVIDE(MachineSoft = MachineSoftInterruptHandler);
PROVIDE(SupervisorTimer = DefaultHandler);
PROVIDE(MachineTimer = MachineTimerInterruptHandler);
PROVIDE(SupervisorExternal = DefaultHandler);
PROVIDE(MachineExternal = MachineExternalInterruptHandler);

/* When vectored trap mode is enabled, each interrupt source must implement its own
   trap entry point. By default, all interrupts start in _DefaultHandler_trap.
   However, users can override these alias by defining the symbol themselves */
PROVIDE(_start_SupervisorSoft_trap = _start_DefaultHandler_trap);
PROVIDE(_start_MachineSoft_trap = _start_DefaultHandler_trap);
PROVIDE(_start_SupervisorTimer_trap = _start_DefaultHandler_trap);
PROVIDE(_start_MachineTimer_trap = _start_DefaultHandler_trap);
PROVIDE(_start_SupervisorExternal_trap = _start_DefaultHandler_trap);
PROVIDE(_start_MachineExternal_trap = _start_DefaultHandler_trap);






/* # EXCEPTION HANDLERS DESCRIBED IN THE STANDARD RISC-V ISA
   
   If the `no-exceptions` feature is DISABLED, this file will be included in link.x.in.
   If the `no-exceptions` feature is ENABLED, this file will be ignored.
*/

/* It is possible to define a special handler for each exception type.
   By default, all exceptions are handled by ExceptionHandler. However,
   users can override these alias by defining the symbol themselves */
PROVIDE(InstructionMisaligned = ExceptionHandler);
PROVIDE(InstructionFault = ExceptionHandler);
PROVIDE(IllegalInstruction = ExceptionHandler);
PROVIDE(Breakpoint = ExceptionHandler);
PROVIDE(LoadMisaligned = ExceptionHandler);
PROVIDE(LoadFault = ExceptionHandler);
PROVIDE(StoreMisaligned = ExceptionHandler);
PROVIDE(StoreFault = ExceptionHandler);
PROVIDE(UserEnvCall = ExceptionHandler);
PROVIDE(SupervisorEnvCall = ExceptionHandler);
PROVIDE(MachineEnvCall = ExceptionHandler);
PROVIDE(InstructionPageFault = ExceptionHandler);
PROVIDE(LoadPageFault = ExceptionHandler);
PROVIDE(StorePageFault = ExceptionHandler);












/* Do not exceed this mark in the error messages above                                    | */
ASSERT(ORIGIN(REGION_TEXT) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_TEXT must be 4-byte aligned");

ASSERT(ORIGIN(REGION_RODATA) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_RODATA must be 4-byte aligned");

ASSERT(ORIGIN(REGION_DATA) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_DATA must be 4-byte aligned");

ASSERT(ORIGIN(REGION_HEAP) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_HEAP must be 4-byte aligned");

ASSERT(ORIGIN(REGION_STACK) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_STACK must be 4-byte aligned");

ASSERT(_stext % 4 == 0, "
ERROR(riscv-rt): `_stext` must be 4-byte aligned");

ASSERT(__sdata % 4 == 0 && __edata % 4 == 0, "
BUG(riscv-rt): .data is not 4-byte aligned");

ASSERT(__sidata % 4 == 0, "
BUG(riscv-rt): the LMA of .data is not 4-byte aligned");

ASSERT(__sbss % 4 == 0 && __ebss % 4 == 0, "
BUG(riscv-rt): .bss is not 4-byte aligned");

ASSERT(__sheap % 4 == 0, "
BUG(riscv-rt): start of .heap is not 4-byte aligned");

ASSERT(_stext + SIZEOF(.text) < ORIGIN(REGION_TEXT) + LENGTH(REGION_TEXT), "
ERROR(riscv-rt): The .text section must be placed inside the REGION_TEXT region.
Set _stext to an address smaller than 'ORIGIN(REGION_TEXT) + LENGTH(REGION_TEXT)'");

ASSERT(SIZEOF(.stack) > (_max_hart_id + 1) * _hart_stack_size, "
ERROR(riscv-rt): .stack section is too small for allocating stacks for all the harts.
Consider changing `_max_hart_id` or `_hart_stack_size`.");

/* # Other checks */
ASSERT(SIZEOF(.got) == 0, "
ERROR(riscv-rt): .got section detected in the input files. Dynamic relocations are not
supported. If you are linking to C code compiled using the `cc` crate then modify your
build script to compile the C code _without_ the -fPIC flag. See the documentation of
the `cc::Build.pic` method for details.");

/* Do not exceed this mark in the error messages above                                    | */
