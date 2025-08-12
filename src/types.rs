use std::collections::HashMap;

/// BPF instruction structure
#[derive(Debug, Clone, PartialEq)]
pub struct BpfInstruction {
    pub opcode: BpfOpcode,
    pub dst_reg: u8,
    pub src_reg: u8,
    pub immediate: i64,
    pub offset: i16,
}

/// BPF opcodes supported by our transpiler
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BpfOpcode {
    // ALU operations
    Add64Imm = 0x07,      // ADD64_IMM
    Add64Reg = 0x0f,      // ADD64_REG
    Sub64Imm = 0x17,      // SUB64_IMM
    Sub64Reg = 0x1f,      // SUB64_REG
    Mul64Imm = 0x27,      // MUL64_IMM
    Mul64Reg = 0x2f,      // MUL64_REG
    Div64Imm = 0x37,      // DIV64_IMM
    Div64Reg = 0x3f,      // DIV64_REG
    Or64Imm = 0x47,       // OR64_IMM
    Or64Reg = 0x4f,       // OR64_REG
    And64Imm = 0x57,      // AND64_IMM
    And64Reg = 0x5f,      // AND64_REG
    Lsh64Imm = 0x67,      // LSH64_IMM
    Lsh64Reg = 0x6f,      // LSH64_REG
    Rsh64Imm = 0x77,      // RSH64_IMM
    Rsh64Reg = 0x7f,      // RSH64_REG
    Neg64 = 0x87,         // NEG64
    Mod64Imm = 0x97,      // MOD64_IMM
    Mod64Reg = 0x9f,      // MOD64_REG
    Xor64Imm = 0xa7,      // XOR64_IMM
    Xor64Reg = 0xaf,      // XOR64_REG
    Mov64Imm = 0xb7,      // MOV64_IMM
    Mov64Reg = 0xbf,      // MOV64_REG
    
    // Memory operations
    LdImm64 = 0x18,       // LD_IMM64
    LdAbs8 = 0x30,        // LD_ABS8
    LdAbs16 = 0x28,       // LD_ABS16
    LdAbs32 = 0x20,       // LD_ABS32
    LdAbs64 = 0x19,       // LD_ABS64 (different from LD_IMM64)
    LdInd8 = 0x38,        // LD_IND8
    LdInd16 = 0x31,       // LD_IND16 (different from LD_ABS8)
    LdInd32 = 0x29,       // LD_IND32 (different from LD_ABS16)
    LdInd64 = 0x21,       // LD_IND64 (different from LD_ABS32)
    Ldx8 = 0x71,          // LDX8
    Ldx16 = 0x69,         // LDX16
    Ldx32 = 0x61,         // LDX32
    Ldx64 = 0x79,         // LDX64
    St8 = 0x72,           // ST8
    St16 = 0x6a,          // ST16
    St32 = 0x62,          // ST32
    St64 = 0x7a,          // ST64
    Stx8 = 0x73,          // STX8
    Stx16 = 0x6b,         // STX16
    Stx32 = 0x63,         // STX32
    Stx64 = 0x7b,         // STX64
    
    // Branch operations
    Ja = 0x05,            // JA
    JeqImm = 0x15,        // JEQ_IMM
    JeqReg = 0x1d,        // JEQ_REG
    JgtImm = 0x25,        // JGT_IMM
    JgtReg = 0x2d,        // JGT_REG
    JgeImm = 0x35,        // JGE_IMM
    JgeReg = 0x3d,        // JGE_REG
    JltImm = 0xa5,        // JLT_IMM
    JltReg = 0xad,        // JLT_REG
    JleImm = 0xb5,        // JLE_IMM
    JleReg = 0xbd,        // JLE_REG
    JsetImm = 0x45,       // JSET_IMM
    JsetReg = 0x4d,       // JSET_REG
    JneImm = 0x55,        // JNE_IMM
    JneReg = 0x5d,        // JNE_REG
    JsgtImm = 0x65,       // JSGT_IMM
    JsgtReg = 0x6d,       // JSGT_REG
    JsgeImm = 0x75,       // JSGE_IMM
    JsgeReg = 0x7d,       // JSGE_REG
    JsltImm = 0xc5,       // JSLT_IMM
    JsltReg = 0xcd,       // JSLT_REG
    JsleImm = 0xd5,       // JSLE_IMM
    JsleReg = 0xdd,       // JSLE_REG
    Call = 0x85,          // CALL
    Exit = 0x95,          // EXIT
}

/// BPF program structure
#[derive(Debug, Clone)]
pub struct BpfProgram {
    pub instructions: Vec<BpfInstruction>,
    pub labels: HashMap<String, usize>,
    pub size: usize,
}

/// Result of BPF program execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: u64,
    pub registers: [u64; 11],
    pub instructions_executed: usize,
    pub execution_time: std::time::Duration,
}

/// Register mapping for BPF to RISC-V conversion
#[derive(Debug, Clone)]
pub struct RegisterMapping {
    pub bpf_reg: u8,
    pub riscv_reg: String,
    pub is_allocated: bool,
}

impl RegisterMapping {
    pub fn new(bpf_reg: u8) -> Self {
        Self {
            bpf_reg,
            riscv_reg: format!("r{}", bpf_reg),
            is_allocated: false,
        }
    }
}

/// BPF program metadata
#[derive(Debug, Clone)]
pub struct BpfProgramMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry_point: usize,
    pub max_stack_size: usize,
    pub max_memory_size: usize,
}

impl Default for BpfProgramMetadata {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            version: "1.0.0".to_string(),
            author: "Unknown".to_string(),
            description: "BPF program".to_string(),
            entry_point: 0,
            max_stack_size: 1024,
            max_memory_size: 1024 * 1024,
        }
    }
}

/// BPF execution context
#[derive(Debug, Clone)]
pub struct BpfExecutionContext {
    pub program: BpfProgram,
    pub metadata: BpfProgramMetadata,
    pub input_data: Vec<u8>,
    pub output_data: Vec<u8>,
    pub execution_trace: Vec<String>,
}

impl BpfExecutionContext {
    pub fn new(program: BpfProgram) -> Self {
        Self {
            metadata: BpfProgramMetadata::default(),
            input_data: Vec::new(),
            output_data: Vec::new(),
            execution_trace: Vec::new(),
            program,
        }
    }
    
    pub fn add_trace(&mut self, message: String) {
        self.execution_trace.push(message);
    }
    
    pub fn set_input(&mut self, data: Vec<u8>) {
        self.input_data = data;
    }
    
    pub fn get_output(&self) -> &[u8] {
        &self.output_data
    }
}
