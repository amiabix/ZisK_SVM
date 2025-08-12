use crate::types::{BpfInstruction, BpfOpcode, BpfProgram, RiscvInstruction, RiscvProgram, RegisterMapping};
use crate::error::{RiscvGenerationError, TranspilerError};
use std::collections::HashMap;

/// RISC-V code generator for BPF instructions
pub struct RiscvGenerator {
    register_mapping: RegisterMapping,
    label_counter: usize,
}

impl RiscvGenerator {
    /// Create a new RISC-V generator
    pub fn new() -> Self {
        Self {
            register_mapping: RegisterMapping::new(),
            label_counter: 0,
        }
    }
    
    /// Generate RISC-V assembly from BPF program
    pub fn generate(&mut self, bpf_program: &BpfProgram) -> Result<Vec<u8>, TranspilerError> {
        let mut riscv_program = RiscvProgram {
            instructions: Vec::new(),
            labels: HashMap::new(),
            data_section: Vec::new(),
            text_section: Vec::new(),
        };
        
        // Add program header
        riscv_program.instructions.push(RiscvInstruction::Label {
            name: "_start".to_string(),
        });
        
        // Generate RISC-V for each BPF instruction
        for (index, bpf_inst) in bpf_program.instructions.iter().enumerate() {
            let riscv_instructions = self.translate_bpf_instruction(bpf_inst, index)?;
            riscv_program.instructions.extend(riscv_instructions);
        }
        
        // Add program footer
        riscv_program.instructions.push(RiscvInstruction::Ecall);
        
        // Convert to binary
        self.assemble_to_binary(riscv_program)
    }
    
    /// Translate a single BPF instruction to RISC-V
    fn translate_bpf_instruction(&mut self, bpf_inst: &BpfInstruction, index: usize) -> Result<Vec<RiscvInstruction>, TranspilerError> {
        let mut riscv_instructions = Vec::new();
        
        match bpf_inst.opcode {
            // ALU operations
            BpfOpcode::Add64Imm => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd,
                    rs1,
                    immediate: bpf_inst.immediate as i32,
                });
            },
            
            BpfOpcode::Add64Reg => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Add {
                    rd,
                    rs1,
                    rs2,
                });
            },
            
            BpfOpcode::Sub64Imm => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd,
                    rs1,
                    immediate: -(bpf_inst.immediate as i32),
                });
            },
            
            BpfOpcode::Sub64Reg => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Sub {
                    rd,
                    rs1,
                    rs2,
                });
            },
            
            BpfOpcode::Mul64Imm => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                riscv_instructions.push(RiscvInstruction::Mul {
                    rd,
                    rs1,
                    rs2: temp_reg,
                });
            },
            
            BpfOpcode::Mul64Reg => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Mul {
                    rd,
                    rs1,
                    rs2,
                });
            },
            
            BpfOpcode::Div64Imm => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                riscv_instructions.push(RiscvInstruction::Div {
                    rd,
                    rs1,
                    rs2: temp_reg,
                });
            },
            
            BpfOpcode::Div64Reg => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Div {
                    rd,
                    rs1,
                    rs2,
                });
            },
            
            BpfOpcode::Or64Imm => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Ori {
                    rd,
                    rs1,
                    immediate: bpf_inst.immediate as i32,
                });
            },
            
            BpfOpcode::Or64Reg => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Or {
                    rd,
                    rs1,
                    rs2,
                });
            },
            
            BpfOpcode::And64Imm => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Andi {
                    rd,
                    rs1,
                    immediate: bpf_inst.immediate as i32,
                });
            },
            
            BpfOpcode::And64Reg => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::And {
                    rd,
                    rs1,
                    rs2,
                });
            },
            
            BpfOpcode::Lsh64Imm => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Slli {
                    rd,
                    rs1,
                    shamt: (bpf_inst.immediate & 0x3F) as u8, // 6-bit shift amount
                });
            },
            
            BpfOpcode::Lsh64Reg => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Sll {
                    rd,
                    rs1,
                    rs2,
                });
            },
            
            BpfOpcode::Rsh64Imm => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Srli {
                    rd,
                    rs1,
                    shamt: (bpf_inst.immediate & 0x3F) as u8, // 6-bit shift amount
                });
            },
            
            BpfOpcode::Rsh64Reg => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Srl {
                    rd,
                    rs1,
                    rs2,
                });
            },
            
            BpfOpcode::Neg64 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // NEG = SUB rd, x0, rs1
                riscv_instructions.push(RiscvInstruction::Sub {
                    rd,
                    rs1: 0, // x0 is always 0
                    rs2: rs1,
                });
            },
            
            BpfOpcode::Mod64Imm => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                riscv_instructions.push(RiscvInstruction::Rem {
                    rd,
                    rs1,
                    rs2: temp_reg,
                });
            },
            
            BpfOpcode::Mod64Reg => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Rem {
                    rd,
                    rs1,
                    rs2,
                });
            },
            
            BpfOpcode::Xor64Imm => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Xori {
                    rd,
                    rs1,
                    immediate: bpf_inst.immediate as i32,
                });
            },
            
            BpfOpcode::Xor64Reg => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Xor {
                    rd,
                    rs1,
                    rs2,
                });
            },
            
            BpfOpcode::Mov64Imm => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // For large immediates, we need to use multiple instructions
                if bpf_inst.immediate > i32::MAX as i64 || bpf_inst.immediate < i32::MIN as i64 {
                    // Load upper 32 bits
                    riscv_instructions.push(RiscvInstruction::Lui {
                        rd,
                        immediate: (bpf_inst.immediate >> 12) as u32,
                    });
                    // Add lower 12 bits
                    riscv_instructions.push(RiscvInstruction::Addi {
                        rd,
                        rs1: rd,
                        immediate: (bpf_inst.immediate & 0xfff) as i32,
                    });
                } else {
                    riscv_instructions.push(RiscvInstruction::Addi {
                        rd,
                        rs1: 0, // x0 is always 0
                        immediate: bpf_inst.immediate as i32,
                    });
                }
            },
            
            BpfOpcode::Mov64Reg => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Add {
                    rd,
                    rs1,
                    rs2: 0, // x0 is always 0
                });
            },
            
            // Memory operations
            BpfOpcode::LdImm64 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // LD_IMM64 loads a 64-bit immediate value
                if bpf_inst.immediate > i32::MAX as i64 || bpf_inst.immediate < i32::MIN as i64 {
                    // Load upper 32 bits
                    riscv_instructions.push(RiscvInstruction::Lui {
                        rd,
                        immediate: (bpf_inst.immediate >> 12) as u32,
                    });
                    // Add lower 12 bits
                    riscv_instructions.push(RiscvInstruction::Addi {
                        rd,
                        rs1: rd,
                        immediate: (bpf_inst.immediate & 0xfff) as i32,
                    });
                } else {
                    riscv_instructions.push(RiscvInstruction::Addi {
                        rd,
                        rs1: 0, // x0 is always 0
                        immediate: bpf_inst.immediate as i32,
                    });
                }
            },
            
            BpfOpcode::Ldx64 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // LDX64 loads from memory address in rs1 + offset
                riscv_instructions.push(RiscvInstruction::Ld {
                    rd,
                    rs1,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::St64 => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // ST64 stores immediate value to memory address in rs1 + offset
                // First load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                riscv_instructions.push(RiscvInstruction::Sd {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::Stx64 => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // STX64 stores register value to memory address in rs1 + offset
                riscv_instructions.push(RiscvInstruction::Sd {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            // Additional memory operations for smaller data types
            BpfOpcode::LdAbs8 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load 8-bit value from absolute address
                riscv_instructions.push(RiscvInstruction::Lb {
                    rd,
                    rs1: 0, // x0 is always 0
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::LdAbs16 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load 16-bit value from absolute address
                riscv_instructions.push(RiscvInstruction::Lh {
                    rd,
                    rs1: 0, // x0 is always 0
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::LdAbs32 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load 32-bit value from absolute address
                riscv_instructions.push(RiscvInstruction::Lw {
                    rd,
                    rs1: 0, // x0 is always 0
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::LdAbs64 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load 64-bit value from absolute address
                riscv_instructions.push(RiscvInstruction::Ld {
                    rd,
                    rs1: 0, // x0 is always 0
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::LdInd8 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Load 8-bit value from indirect address
                riscv_instructions.push(RiscvInstruction::Lb {
                    rd,
                    rs1,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::LdInd16 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Load 16-bit value from indirect address
                riscv_instructions.push(RiscvInstruction::Lh {
                    rd,
                    rs1,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::LdInd32 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Load 32-bit value from indirect address
                riscv_instructions.push(RiscvInstruction::Lw {
                    rd,
                    rs1,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::LdInd64 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Load 64-bit value from indirect address
                riscv_instructions.push(RiscvInstruction::Ld {
                    rd,
                    rs1,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::Ldx8 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Load 8-bit value from indexed address
                riscv_instructions.push(RiscvInstruction::Lb {
                    rd,
                    rs1,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::Ldx16 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Load 16-bit value from indexed address
                riscv_instructions.push(RiscvInstruction::Lh {
                    rd,
                    rs1,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::Ldx32 => {
                let rd = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Load 32-bit value from indexed address
                riscv_instructions.push(RiscvInstruction::Lw {
                    rd,
                    rs1,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::St8 => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Store 8-bit immediate value
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                riscv_instructions.push(RiscvInstruction::Sb {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::St16 => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Store 16-bit immediate value
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                riscv_instructions.push(RiscvInstruction::Sh {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::St32 => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Store 32-bit immediate value
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                riscv_instructions.push(RiscvInstruction::Sw {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::Stx8 => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Store 8-bit register value
                riscv_instructions.push(RiscvInstruction::Sb {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::Stx16 => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Store 16-bit register value
                riscv_instructions.push(RiscvInstruction::Sh {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::Stx32 => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Store 32-bit register value
                riscv_instructions.push(RiscvInstruction::Sw {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            // Branch operations
            BpfOpcode::Ja => {
                // JA jumps to PC + offset
                riscv_instructions.push(RiscvInstruction::Jal {
                    rd: 0, // Don't save return address
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JeqImm => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                // Compare and branch if equal
                riscv_instructions.push(RiscvInstruction::Beq {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JeqReg => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Compare and branch if equal
                riscv_instructions.push(RiscvInstruction::Beq {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JgtImm => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                // Compare and branch if greater than
                riscv_instructions.push(RiscvInstruction::Bgt {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JgtReg => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Compare and branch if greater than
                riscv_instructions.push(RiscvInstruction::Bgt {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JgeImm => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                // Compare and branch if greater than or equal
                riscv_instructions.push(RiscvInstruction::Bge {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JgeReg => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Compare and branch if greater than or equal
                riscv_instructions.push(RiscvInstruction::Bge {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JltImm => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                // Compare and branch if less than
                riscv_instructions.push(RiscvInstruction::Blt {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JltReg => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Compare and branch if less than
                riscv_instructions.push(RiscvInstruction::Blt {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JleImm => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                // Compare and branch if less than or equal
                riscv_instructions.push(RiscvInstruction::Ble {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JleReg => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Compare and branch if less than or equal
                riscv_instructions.push(RiscvInstruction::Ble {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JsetImm => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                // Test if any bits are set (AND then branch if not zero)
                let test_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::And {
                    rd: test_reg,
                    rs1,
                    rs2: temp_reg,
                });
                
                // Branch if any bits are set (not equal to zero)
                riscv_instructions.push(RiscvInstruction::Bne {
                    rs1: test_reg,
                    rs2: 0, // x0 is always 0
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JsetReg => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Test if any bits are set (AND then branch if not zero)
                let test_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::And {
                    rd: test_reg,
                    rs1,
                    rs2,
                });
                
                // Branch if any bits are set (not equal to zero)
                riscv_instructions.push(RiscvInstruction::Bne {
                    rs1: test_reg,
                    rs2: 0, // x0 is always 0
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JneImm => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                // Compare and branch if not equal
                riscv_instructions.push(RiscvInstruction::Bne {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JneReg => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Compare and branch if not equal
                riscv_instructions.push(RiscvInstruction::Bne {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JsgtImm => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                // Signed comparison: branch if greater than
                riscv_instructions.push(RiscvInstruction::Bgt {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JsgtReg => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Signed comparison: branch if greater than
                riscv_instructions.push(RiscvInstruction::Bgt {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JsgeImm => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                // Signed comparison: branch if greater than or equal
                riscv_instructions.push(RiscvInstruction::Bge {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JsgeReg => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Signed comparison: branch if greater than or equal
                riscv_instructions.push(RiscvInstruction::Bge {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JsltImm => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                // Signed comparison: branch if less than
                riscv_instructions.push(RiscvInstruction::Blt {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JsltReg => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Signed comparison: branch if less than
                riscv_instructions.push(RiscvInstruction::Blt {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JsleImm => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                // Load immediate into temporary register
                let temp_reg = self.register_mapping.allocate_temp_reg();
                riscv_instructions.push(RiscvInstruction::Addi {
                    rd: temp_reg,
                    rs1: 0, // x0 is always 0
                    immediate: bpf_inst.immediate as i32,
                });
                
                // Signed comparison: branch if less than or equal
                riscv_instructions.push(RiscvInstruction::Ble {
                    rs1,
                    rs2: temp_reg,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::JsleReg => {
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                let rs2 = self.register_mapping.get_riscv_reg(bpf_inst.src_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.src_reg)
                    }))?;
                
                // Signed comparison: branch if less than or equal
                riscv_instructions.push(RiscvInstruction::Ble {
                    rs1,
                    rs2,
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::Call => {
                // CALL calls a function at PC + offset
                riscv_instructions.push(RiscvInstruction::Jal {
                    rd: 1, // Save return address in x1 (ra)
                    offset: bpf_inst.offset as i32,
                });
            },
            
            BpfOpcode::Exit => {
                // Set exit code in a0 (x10)
                let rd = 10; // a0 register
                let rs1 = self.register_mapping.get_riscv_reg(bpf_inst.dst_reg)
                    .ok_or_else(|| TranspilerError::RiscvGenerationError(RiscvGenerationError::RegisterAllocationError {
                        message: format!("Failed to map BPF register {}", bpf_inst.dst_reg)
                    }))?;
                
                riscv_instructions.push(RiscvInstruction::Add {
                    rd,
                    rs1,
                    rs2: 0,
                });
                
                // Jump to exit
                riscv_instructions.push(RiscvInstruction::Jal {
                    rd: 0, // Don't save return address
                    offset: 0, // Will be resolved during assembly
                });
            },
            
            // Add more opcode translations here...
            _ => {
                return Err(TranspilerError::RiscvGenerationError(RiscvGenerationError::InstructionGenerationFailed {
                    instruction: format!("{:?}", bpf_inst.opcode),
                }));
            }
        }
        
        Ok(riscv_instructions)
    }
    
    /// Assemble RISC-V program to binary
    fn assemble_to_binary(&self, riscv_program: RiscvProgram) -> Result<Vec<u8>, TranspilerError> {
        // This is a simplified assembler - in production, you'd use a proper RISC-V assembler
        let mut binary = Vec::new();
        
        for instruction in riscv_program.instructions {
            match instruction {
                RiscvInstruction::Add { rd, rs1, rs2 } => {
                    // R-type instruction: ADD rd, rs1, rs2
                    let opcode = 0x33; // R-type
                    let funct3 = 0x0;  // ADD
                    let funct7 = 0x0;  // ADD
                    
                    let instruction: u32 = (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Addi { rd, rs1, immediate } => {
                    // I-type instruction: ADDI rd, rs1, immediate
                    let opcode = 0x13; // I-type
                    let funct3 = 0x0;  // ADDI
                    
                    let instruction: u32 = (immediate as u32) << 20 | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Sub { rd, rs1, rs2 } => {
                    // R-type instruction: SUB rd, rs1, rs2
                    let opcode = 0x33; // R-type
                    let funct3 = 0x0;  // SUB
                    let funct7 = 0x20; // SUB
                    
                    let instruction: u32 = (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Mul { rd, rs1, rs2 } => {
                    // R-type instruction: MUL rd, rs1, rs2
                    let opcode = 0x33; // R-type
                    let funct3 = 0x0;  // MUL
                    let funct7 = 0x1;  // MUL
                    
                    let instruction: u32 = (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Div { rd, rs1, rs2 } => {
                    // R-type instruction: DIV rd, rs1, rs2
                    let opcode = 0x33; // R-type
                    let funct3 = 0x4;  // DIV
                    let funct7 = 0x1;  // DIV
                    
                    let instruction: u32 = (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Rem { rd, rs1, rs2 } => {
                    // R-type instruction: REM rd, rs1, rs2
                    let opcode = 0x33; // R-type
                    let funct3 = 0x6;  // REM
                    let funct7 = 0x1;  // REM
                    
                    let instruction: u32 = (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::And { rd, rs1, rs2 } => {
                    // R-type instruction: AND rd, rs1, rs2
                    let opcode = 0x33; // R-type
                    let funct3 = 0x7;  // AND
                    let funct7 = 0x0;  // AND
                    
                    let instruction: u32 = (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Andi { rd, rs1, immediate } => {
                    // I-type instruction: ANDI rd, rs1, immediate
                    let opcode = 0x13; // I-type
                    let funct3 = 0x7;  // ANDI
                    
                    let instruction: u32 = (immediate as u32) << 20 | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Or { rd, rs1, rs2 } => {
                    // R-type instruction: OR rd, rs1, rs2
                    let opcode = 0x33; // R-type
                    let funct3 = 0x6;  // OR
                    let funct7 = 0x0;  // OR
                    
                    let instruction: u32 = (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Ori { rd, rs1, immediate } => {
                    // I-type instruction: ORI rd, rs1, immediate
                    let opcode = 0x13; // I-type
                    let funct3 = 0x6;  // ORI
                    
                    let instruction: u32 = (immediate as u32) << 20 | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Xor { rd, rs1, rs2 } => {
                    // R-type instruction: XOR rd, rs1, rs2
                    let opcode = 0x33; // R-type
                    let funct3 = 0x4;  // XOR
                    let funct7 = 0x0;  // XOR
                    
                    let instruction: u32 = (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Xori { rd, rs1, immediate } => {
                    // I-type instruction: XORI rd, rs1, immediate
                    let opcode = 0x13; // I-type
                    let funct3 = 0x4;  // XORI
                    
                    let instruction: u32 = (immediate as u32) << 20 | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Sll { rd, rs1, rs2 } => {
                    // R-type instruction: SLL rd, rs1, rs2
                    let opcode = 0x33; // R-type
                    let funct3 = 0x1;  // SLL
                    let funct7 = 0x0;  // SLL
                    
                    let instruction: u32 = (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Slli { rd, rs1, shamt } => {
                    // I-type instruction: SLLI rd, rs1, shamt
                    let opcode = 0x13; // I-type
                    let funct3 = 0x1;  // SLLI
                    let funct6 = 0x0;  // SLLI
                    
                    let instruction: u32 = ((funct6 as u32) << 26) | ((shamt as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Srl { rd, rs1, rs2 } => {
                    // R-type instruction: SRL rd, rs1, rs2
                    let opcode = 0x33; // R-type
                    let funct3 = 0x5;  // SRL
                    let funct7 = 0x0;  // SRL
                    
                    let instruction: u32 = (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Srli { rd, rs1, shamt } => {
                    // I-type instruction: SRLI rd, rs1, shamt
                    let opcode = 0x13; // I-type
                    let funct3 = 0x5;  // SRLI
                    let funct6 = 0x0;  // SRLI
                    
                    let instruction: u32 = ((funct6 as u32) << 26) | ((shamt as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Sra { rd, rs1, rs2 } => {
                    // R-type instruction: SRA rd, rs1, rs2
                    let opcode = 0x33; // R-type
                    let funct3 = 0x5;  // SRA
                    let funct7 = 0x20; // SRA
                    
                    let instruction: u32 = (funct7 << 25) | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Srai { rd, rs1, shamt } => {
                    // I-type instruction: SRAI rd, rs1, shamt
                    let opcode = 0x13; // I-type
                    let funct3 = 0x5;  // SRAI
                    let funct6 = 0x10; // SRAI
                    
                    let instruction: u32 = ((funct6 as u32) << 26) | ((shamt as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Jal { rd, offset } => {
                    // J-type instruction: JAL rd, offset
                    let opcode = 0x6f; // J-type
                    
                    let instruction: u32 = ((offset as u32) << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Ecall => {
                    // System instruction: ECALL
                    let opcode = 0x73; // System
                    let funct12 = 0x0; // ECALL
                    
                    let instruction: u32 = (funct12 << 20) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Lui { rd, immediate } => {
                    // U-type instruction: LUI rd, immediate
                    let opcode = 0x37; // U-type
                    
                    let instruction: u32 = (immediate << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Label { name: _ } => {
                    // Labels are resolved during assembly - skip for now
                    continue;
                },
                
                RiscvInstruction::Nop => {
                    // NOP = ADDI x0, x0, 0
                    let opcode = 0x13; // I-type
                    let funct3 = 0x0;  // ADDI
                    let rd = 0;        // x0
                    let rs1 = 0;       // x0
                    let immediate = 0;
                    
                    let instruction: u32 = (immediate << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Bgeu { rs1, rs2, offset } => {
                    // B-type instruction: BGEU rs1, rs2, offset
                    let opcode = 0x63; // B-type
                    let funct3 = 0x7;  // BGEU
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 12) & 1) << 31 | ((offset_u32 >> 5) & 0x3f) << 25 | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((offset_u32 >> 1) & 0xf) << 8 | ((offset_u32 >> 11) & 1) << 7 | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Bgt { rs1, rs2, offset } => {
                    // B-type instruction: BGT rs1, rs2, offset (BGT = BLT with swapped operands)
                    let opcode = 0x63; // B-type
                    let funct3 = 0x4;  // BLT
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 12) & 1) << 31 | ((offset_u32 >> 5) & 0x3f) << 25 | ((rs1 as u32) << 20) | ((rs2 as u32) << 15) | (funct3 << 12) | ((offset_u32 >> 1) & 0xf) << 8 | ((offset_u32 >> 11) & 1) << 7 | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Ble { rs1, rs2, offset } => {
                    // B-type instruction: BLE rs1, rs2, offset (BLE = BGE with swapped operands)
                    let opcode = 0x63; // B-type
                    let funct3 = 0x5;  // BGE
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 12) & 1) << 31 | ((offset_u32 >> 5) & 0x3f) << 25 | ((rs1 as u32) << 20) | ((rs2 as u32) << 15) | (funct3 << 12) | ((offset_u32 >> 1) & 0xf) << 8 | ((offset_u32 >> 11) & 1) << 7 | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Ld { rd, rs1, offset } => {
                    // I-type instruction: LD rd, rs1, offset
                    let opcode = 0x03; // I-type
                    let funct3 = 0x3;  // LD
                    
                    let instruction: u32 = (offset as u32) << 20 | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Sd { rs1, rs2, offset } => {
                    // S-type instruction: SD rs1, rs2, offset
                    let opcode = 0x23; // S-type
                    let funct3 = 0x3;  // SD
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 5) & 0x7f) << 25 | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((offset_u32 & 0x1f) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Beq { rs1, rs2, offset } => {
                    // B-type instruction: BEQ rs1, rs2, offset
                    let opcode = 0x63; // B-type
                    let funct3 = 0x0;  // BEQ
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 12) & 1) << 31 | ((offset_u32 >> 5) & 0x3f) << 25 | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((offset_u32 >> 1) & 0xf) << 8 | ((offset_u32 >> 11) & 1) << 7 | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Bne { rs1, rs2, offset } => {
                    // B-type instruction: BNE rs1, rs2, offset
                    let opcode = 0x63; // B-type
                    let funct3 = 0x1;  // BNE
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 12) & 1) << 31 | ((offset_u32 >> 5) & 0x3f) << 25 | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((offset_u32 >> 1) & 0xf) << 8 | ((offset_u32 >> 11) & 1) << 7 | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Blt { rs1, rs2, offset } => {
                    // B-type instruction: BLT rs1, rs2, offset
                    let opcode = 0x63; // B-type
                    let funct3 = 0x4;  // BLT
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 12) & 1) << 31 | ((offset_u32 >> 5) & 0x3f) << 25 | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((offset_u32 >> 1) & 0xf) << 8 | ((offset_u32 >> 11) & 1) << 7 | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Bge { rs1, rs2, offset } => {
                    // B-type instruction: BGE rs1, rs2, offset
                    let opcode = 0x63; // B-type
                    let funct3 = 0x5;  // BGE
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 12) & 1) << 31 | ((offset_u32 >> 5) & 0x3f) << 25 | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((offset_u32 >> 1) & 0xf) << 8 | ((offset_u32 >> 11) & 1) << 7 | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Bltu { rs1, rs2, offset } => {
                    // B-type instruction: BLTU rs1, rs2, offset
                    let opcode = 0x63; // B-type
                    let funct3 = 0x6;  // BLTU
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 12) & 1) << 31 | ((offset_u32 >> 5) & 0x3f) << 25 | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((offset_u32 >> 1) & 0xf) << 8 | ((offset_u32 >> 11) & 1) << 7 | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Lb { rd, rs1, offset } => {
                    // I-type instruction: LB rd, rs1, offset
                    let opcode = 0x03; // I-type
                    let funct3 = 0x0;  // LB
                    
                    let instruction: u32 = (offset as u32) << 20 | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Lh { rd, rs1, offset } => {
                    // I-type instruction: LH rd, rs1, offset
                    let opcode = 0x03; // I-type
                    let funct3 = 0x1;  // LH
                    
                    let instruction: u32 = (offset as u32) << 20 | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Lw { rd, rs1, offset } => {
                    // I-type instruction: LW rd, rs1, offset
                    let opcode = 0x03; // I-type
                    let funct3 = 0x2;  // LW
                    
                    let instruction: u32 = (offset as u32) << 20 | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Sb { rs1, rs2, offset } => {
                    // S-type instruction: SB rs1, rs2, offset
                    let opcode = 0x23; // S-type
                    let funct3 = 0x0;  // SB
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 5) & 0x7f) << 25 | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((offset_u32 & 0x1f) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Sh { rs1, rs2, offset } => {
                    // S-type instruction: SH rs1, rs2, offset
                    let opcode = 0x23; // S-type
                    let funct3 = 0x1;  // SH
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 5) & 0x7f) << 25 | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((offset_u32 & 0x1f) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                RiscvInstruction::Sw { rs1, rs2, offset } => {
                    // S-type instruction: SW rs1, rs2, offset
                    let opcode = 0x23; // S-type
                    let funct3 = 0x2;  // SW
                    
                    let offset_u32 = offset as u32;
                    let instruction: u32 = ((offset_u32 >> 5) & 0x7f) << 25 | ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((offset_u32 & 0x1f) << 7) | opcode;
                    binary.extend_from_slice(&instruction.to_le_bytes());
                },
                
                // Add more instruction encodings here...
                _ => {
                    return Err(TranspilerError::RiscvGenerationError(RiscvGenerationError::InstructionGenerationFailed {
                        instruction: format!("{:?}", instruction),
                    }));
                }
            }
        }
        
        Ok(binary)
    }
    
    /// Generate a unique label
    fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }
}

impl Default for RiscvGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BpfInstruction, BpfOpcode};
    
    #[test]
    fn test_generate_simple_program() {
        let mut generator = RiscvGenerator::new();
        
        let bpf_program = BpfProgram {
            instructions: vec![
                BpfInstruction {
                    opcode: BpfOpcode::Mov64Imm,
                    dst_reg: 0,
                    src_reg: 0,
                    immediate: 42,
                    offset: 0,
                },
                BpfInstruction {
                    opcode: BpfOpcode::Exit,
                    dst_reg: 0,
                    src_reg: 0,
                    immediate: 0,
                    offset: 0,
                },
            ],
            labels: HashMap::new(),
            size: 16,
        };
        
        let result = generator.generate(&bpf_program);
        assert!(result.is_ok());
        
        let binary = result.unwrap();
        assert!(!binary.is_empty());
    }
}
