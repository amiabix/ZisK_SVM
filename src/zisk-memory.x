/* ZisK Memory Layout for Solana Test */
MEMORY {
    /* 64KB of RAM starting at 0x1000 */
    ram (rwx) : ORIGIN = 0x1000, LENGTH = 64K
}

SECTIONS {
    .text : {
        *(.text .text.*)
    } > ram
    
    .rodata : {
        *(.rodata .rodata.*)
    } > ram
    
    .data : {
        *(.data .data.*)
    } > ram
    
    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } > ram
    
    /* Stack grows downward from end of RAM */
    .stack : {
        . = . + 8K;
        _stack = .;
    } > ram
    
    /* Heap starts after stack */
    _heap_start = _stack;
    _heap_end = ORIGIN(ram) + LENGTH(ram);
}
