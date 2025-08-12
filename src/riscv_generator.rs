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
