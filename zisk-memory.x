/* ZisK Memory Layout for RISC-V zkVM */
MEMORY {
    /* Program memory - starts at 0x1000 */
    prog (rx) : ORIGIN = 0x1000, LENGTH = 64K
    
    /* Stack - grows downward from top of RAM */
    stack (rw) : ORIGIN = 0x20000, LENGTH = 8K
    
    /* Heap - grows upward from end of program */
    heap (rw) : ORIGIN = 0x11000, LENGTH = 64K
}

/* Entry point */
ENTRY(_start)

/* Stack pointer initialization */
_stack_start = ORIGIN(stack) + LENGTH(stack);

SECTIONS {
    /* Program code and data */
    .text : {
        *(.text .text.*)
        *(.rodata .rodata.*)
    } > prog
    
    /* Data section */
    .data : {
        *(.data .data.*)
    } > heap
    
    /* BSS section */
    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } > heap
    
    /* Stack section */
    .stack : {
        . = ALIGN(8);
        _stack_end = .;
        . = . + LENGTH(stack);
        . = ALIGN(8);
        _stack_start = .;
    } > stack
    
    /* Heap section */
    .heap : {
        . = ALIGN(8);
        _heap_start = .;
        . = . + LENGTH(heap);
        . = ALIGN(8);
        _heap_end = .;
    } > heap
}
