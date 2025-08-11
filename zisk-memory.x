/* ZisK Memory Layout Configuration
 * 
 * This file defines the memory layout for our Solana Virtual Machine
 * when running within the ZisK zero-knowledge virtual machine.
 * 
 * Memory Layout:
 * - Code:    0x1000 - 0x11000 (64KB)
 * - Data:    0x11000 - 0x111000 (1MB)
 * - Stack:   0x111000 - 0x211000 (1MB)
 * - Heap:    0x211000 - 0x411000 (32MB)
 * - Total:   64MB available
 */

MEMORY {
    /* Code section - contains BPF program code */
    code (rx) : ORIGIN = 0x1000, LENGTH = 0x10000
    
    /* Data section - contains static data and constants */
    data (rw) : ORIGIN = 0x11000, LENGTH = 0x100000
    
    /* Stack section - for function call stack */
    stack (rw) : ORIGIN = 0x111000, LENGTH = 0x100000
    
    /* Heap section - for dynamic memory allocation */
    heap (rw) : ORIGIN = 0x211000, LENGTH = 0x2000000
}

/* Entry point */
ENTRY(_start)

/* Section definitions */
SECTIONS {
    /* Code section */
    .text : {
        *(.text)
        *(.text.*)
        . = ALIGN(4);
    } > code
    
    /* Read-only data */
    .rodata : {
        *(.rodata)
        *(.rodata.*)
        . = ALIGN(4);
    } > data
    
    /* Read-write data */
    .data : {
        *(.data)
        *(.data.*)
        . = ALIGN(4);
    } > data
    
    /* Uninitialized data */
    .bss : {
        *(.bss)
        *(.bss.*)
        *(COMMON)
        . = ALIGN(4);
    } > data
    
    /* Stack pointer initialization */
    .stack : {
        . = ALIGN(16);
        _stack_start = .;
        . = . + 0x100000; /* 1MB stack */
        _stack_end = .;
        . = ALIGN(16);
    } > stack
    
    /* Heap initialization */
    .heap : {
        . = ALIGN(16);
        _heap_start = .;
        . = . + 0x2000000; /* 32MB heap */
        _heap_end = .;
        . = ALIGN(16);
    } > heap
    
    /* Discard unused sections */
    /DISCARD/ : {
        *(.comment)
        *(.gnu.*)
        *(.note.*)
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}

/* Symbol definitions for Rust */
PROVIDE(_start = 0x1000);
PROVIDE(_stack_start = 0x111000);
PROVIDE(_stack_end = 0x211000);
PROVIDE(_heap_start = 0x211000);
PROVIDE(_heap_end = 0x411000);
