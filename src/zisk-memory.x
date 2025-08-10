/* ZisK Memory Layout for Solana Test */
MEMORY {
    /* Main program memory */
    ram : ORIGIN = 0x10000000, LENGTH = 64K
    /* Stack space */
    stack : ORIGIN = 0x10010000, LENGTH = 8K
}

SECTIONS {
    .text : {
        *(.text)
        *(.text.*)
    } > ram
    
    .rodata : {
        *(.rodata)
        *(.rodata.*)
    } > ram
    
    .data : {
        *(.data)
        *(.data.*)
    } > ram
    
    .bss : {
        *(.bss)
        *(.bss.*)
        *(COMMON)
    } > ram
    
    .stack : {
        . = ALIGN(8);
        . += 8K;
        . = ALIGN(8);
        _stack_top = .;
    } > stack
    
    /DISCARD/ : {
        *(.comment)
        *(.gnu.*)
        *(.note.*)
        *(.eh_frame)
    }
}

/* Entry point */
ENTRY(_start)
/* Stack pointer initialization */
_stack_pointer = _stack_top;
