use crate::types::{BpfInstruction, BpfOpcode, BpfProgram};
use crate::error::{BpfParseError, TranspilerError};
use std::collections::HashMap;

/// BPF bytecode parser
pub struct BpfParser {
    max_program_size: usize,
}

impl BpfParser {
    /// Create a new BPF parser
    pub fn new() -> Self {
        Self {
            max_program_size: 1_000_000, // 1MB max program size
        }
    }
    
    /// Parse BPF bytecode into structured instructions
    pub fn parse(&self, bytecode: &[u8]) -> Result<BpfProgram, TranspilerError> {
        if bytecode.len() > self.max_program_size {
            return Err(TranspilerError::BpfParseError(BpfParseError::ProgramTooLarge { 
                size: bytecode.len(), 
                max_size: self.max_program_size 
            }));
        }
        
        let mut instructions = Vec::new();
        let labels = HashMap::new();
        let mut offset = 0;
        
        while offset < bytecode.len() {
            if offset + 8 > bytecode.len() {
                return Err(TranspilerError::BpfParseError(BpfParseError::UnexpectedEndOfInput { offset }));
            }
            
            let instruction = self.parse_instruction(bytecode, offset)?;
            instructions.push(instruction.clone());
            
            // BPF instructions are 8 bytes, except LD_IMM64 which is 16 bytes
            if instruction.opcode == BpfOpcode::LdImm64 {
                offset += 16;
            } else {
                offset += 8;
            }
        }
        
        Ok(BpfProgram {
            instructions,
            labels,
            size: bytecode.len(),
        })
    }
    
    /// Parse a single BPF instruction
    fn parse_instruction(&self, bytecode: &[u8], offset: usize) -> Result<BpfInstruction, TranspilerError> {
        let opcode = bytecode[offset];
        let dst_reg = bytecode[offset + 1] & 0x0f; // Lower 4 bits
        let src_reg = (bytecode[offset + 1] >> 4) & 0x0f; // Upper 4 bits

        // Handle LD_IMM64 instruction (16 bytes)
        if opcode == 0x18 { // LD_IMM64
            if offset + 16 > bytecode.len() {
                return Err(TranspilerError::BpfParseError(BpfParseError::UnexpectedEndOfInput { offset }));
            }
            
            let immediate_bytes = &bytecode[offset + 8..offset + 16];
            let immediate = i64::from_le_bytes([
                immediate_bytes[0], immediate_bytes[1], immediate_bytes[2], immediate_bytes[3],
                immediate_bytes[4], immediate_bytes[5], immediate_bytes[6], immediate_bytes[7]
            ]);

            Ok(BpfInstruction {
                opcode: BpfOpcode::LdImm64,
                dst_reg,
                src_reg: 0,
                immediate,
                offset: 0,
            })
        } else {
            // Regular 8-byte instruction
            if offset + 8 > bytecode.len() {
                return Err(TranspilerError::BpfParseError(BpfParseError::UnexpectedEndOfInput { offset }));
            }

            let offset_bytes = &bytecode[offset + 2..offset + 4];
            let immediate_bytes = &bytecode[offset + 4..offset + 8];

            // Validate register indices
            if dst_reg > 10 {
                return Err(TranspilerError::BpfParseError(BpfParseError::InvalidOpcode { opcode: dst_reg }));
            }
            if src_reg > 10 {
                return Err(TranspilerError::BpfParseError(BpfParseError::InvalidOpcode { opcode: src_reg }));
            }

            let offset = i16::from_le_bytes([offset_bytes[0], offset_bytes[1]]);
            let immediate = i64::from_le_bytes([
                immediate_bytes[0], immediate_bytes[1], immediate_bytes[2], immediate_bytes[3],
                0, 0, 0, 0
            ]);

            let opcode = self.parse_opcode(opcode)?;

            Ok(BpfInstruction {
                opcode,
                dst_reg,
                src_reg,
                immediate,
                offset,
            })
        }
    }
    
    /// Parse BPF opcode
    fn parse_opcode(&self, opcode: u8) -> Result<BpfOpcode, TranspilerError> {
        match opcode {
            0x07 => Ok(BpfOpcode::Add64Imm),
            0x0f => Ok(BpfOpcode::Add64Reg),
            0x17 => Ok(BpfOpcode::Sub64Imm),
            0x1f => Ok(BpfOpcode::Sub64Reg),
            0x27 => Ok(BpfOpcode::Mul64Imm),
            0x2f => Ok(BpfOpcode::Mul64Reg),
            0x37 => Ok(BpfOpcode::Div64Imm),
            0x3f => Ok(BpfOpcode::Div64Reg),
            0x47 => Ok(BpfOpcode::Or64Imm),
            0x4f => Ok(BpfOpcode::Or64Reg),
            0x57 => Ok(BpfOpcode::And64Imm),
            0x5f => Ok(BpfOpcode::And64Reg),
            0x67 => Ok(BpfOpcode::Lsh64Imm),
            0x6f => Ok(BpfOpcode::Lsh64Reg),
            0x77 => Ok(BpfOpcode::Rsh64Imm),
            0x7f => Ok(BpfOpcode::Rsh64Reg),
            0x87 => Ok(BpfOpcode::Neg64),
            0x97 => Ok(BpfOpcode::Mod64Imm),
            0x9f => Ok(BpfOpcode::Mod64Reg),
            0xa7 => Ok(BpfOpcode::Xor64Imm),
            0xaf => Ok(BpfOpcode::Xor64Reg),
            0xb7 => Ok(BpfOpcode::Mov64Imm),
            0xbf => Ok(BpfOpcode::Mov64Reg),
            0x18 => Ok(BpfOpcode::LdImm64),
            0x30 => Ok(BpfOpcode::LdAbs8),
            0x28 => Ok(BpfOpcode::LdAbs16),
            0x20 => Ok(BpfOpcode::LdAbs32),
            0x19 => Ok(BpfOpcode::LdAbs64),
            0x38 => Ok(BpfOpcode::LdInd8),
            0x31 => Ok(BpfOpcode::LdInd16),
            0x29 => Ok(BpfOpcode::LdInd32),
            0x21 => Ok(BpfOpcode::LdInd64),
            0x71 => Ok(BpfOpcode::Ldx8),
            0x69 => Ok(BpfOpcode::Ldx16),
            0x61 => Ok(BpfOpcode::Ldx32),
            0x79 => Ok(BpfOpcode::Ldx64),
            0x72 => Ok(BpfOpcode::St8),
            0x6a => Ok(BpfOpcode::St16),
            0x62 => Ok(BpfOpcode::St32),
            0x7a => Ok(BpfOpcode::St64),
            0x73 => Ok(BpfOpcode::Stx8),
            0x6b => Ok(BpfOpcode::Stx16),
            0x63 => Ok(BpfOpcode::Stx32),
            0x7b => Ok(BpfOpcode::Stx64),
            0x05 => Ok(BpfOpcode::Ja),
            0x15 => Ok(BpfOpcode::JeqImm),
            0x1d => Ok(BpfOpcode::JeqReg),
            0x25 => Ok(BpfOpcode::JgtImm),
            0x2d => Ok(BpfOpcode::JgtReg),
            0x35 => Ok(BpfOpcode::JgeImm),
            0x3d => Ok(BpfOpcode::JgeReg),
            0xa5 => Ok(BpfOpcode::JltImm),
            0xad => Ok(BpfOpcode::JltReg),
            0xb5 => Ok(BpfOpcode::JleImm),
            0xbd => Ok(BpfOpcode::JleReg),
            0x45 => Ok(BpfOpcode::JsetImm),
            0x4d => Ok(BpfOpcode::JsetReg),
            0x55 => Ok(BpfOpcode::JneImm),
            0x5d => Ok(BpfOpcode::JneReg),
            0x65 => Ok(BpfOpcode::JsgtImm),
            0x6d => Ok(BpfOpcode::JsgtReg),
            0x75 => Ok(BpfOpcode::JsgeImm),
            0x7d => Ok(BpfOpcode::JsgeReg),
            0xc5 => Ok(BpfOpcode::JsltImm),
            0xcd => Ok(BpfOpcode::JsltReg),
            0xd5 => Ok(BpfOpcode::JsleImm),
            0xdd => Ok(BpfOpcode::JsleReg),
            0x85 => Ok(BpfOpcode::Call),
            0x95 => Ok(BpfOpcode::Exit),
            _ => Err(TranspilerError::BpfParseError(BpfParseError::InvalidOpcode { opcode })),
        }
    }
    
    /// Set maximum program size
    pub fn set_max_program_size(&mut self, size: usize) {
        self.max_program_size = size;
    }
}

impl Default for BpfParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BpfOpcode;
    
    #[test]
    fn test_parse_simple_instruction() {
        let parser = BpfParser::new();
        
        // MOV64_IMM R0, 42
        let bytecode = vec![0xb7, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00];
        
        let result = parser.parse(&bytecode).unwrap();
        assert_eq!(result.instructions.len(), 1);
        
        let instruction = &result.instructions[0];
        assert_eq!(instruction.opcode, BpfOpcode::Mov64Imm);
        assert_eq!(instruction.dst_reg, 0);
        assert_eq!(instruction.src_reg, 0);
        assert_eq!(instruction.immediate, 42);
        assert_eq!(instruction.offset, 0);
    }
    
    #[test]
    fn test_parse_ld_imm64() {
        let parser = BpfParser::new();
        
        // LD_IMM64 R0, 0x1234567890abcdef
        let bytecode = vec![
            0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xef, 0xcd, 0xab, 0x90, 0x78, 0x56, 0x34, 0x12,
        ];
        
        let result = parser.parse(&bytecode).unwrap();
        assert_eq!(result.instructions.len(), 1);
        
        let instruction = &result.instructions[0];
        assert_eq!(instruction.opcode, BpfOpcode::LdImm64);
        assert_eq!(instruction.dst_reg, 0);
        assert_eq!(instruction.immediate, 0x1234567890abcdef);
    }
    
    #[test]
    fn test_parse_multiple_instructions() {
        let parser = BpfParser::new();
        
        // MOV64_IMM R0, 42
        // ADD64_IMM R0, 10
        // EXIT
        let bytecode = vec![
            0xb7, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00,
            0x07, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00,
            0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        
        let result = parser.parse(&bytecode).unwrap();
        assert_eq!(result.instructions.len(), 3);
        
        assert_eq!(result.instructions[0].opcode, BpfOpcode::Mov64Imm);
        assert_eq!(result.instructions[1].opcode, BpfOpcode::Add64Imm);
        assert_eq!(result.instructions[2].opcode, BpfOpcode::Exit);
    }
    
    #[test]
    fn test_parse_invalid_register() {
        let parser = BpfParser::new();
        
        // MOV64_IMM R15, 42 (invalid register)
        let bytecode = vec![0xb7, 0xf0, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00];
        
        let result = parser.parse(&bytecode);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parse_unsupported_opcode() {
        let parser = BpfParser::new();
        
        // Invalid opcode 0xff
        let bytecode = vec![0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        
        let result = parser.parse(&bytecode);
        assert!(result.is_err());
    }
}
