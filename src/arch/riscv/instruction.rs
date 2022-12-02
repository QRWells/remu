use super::{csr::Csr, reg::register_name};

use core::{fmt, sync::atomic::Ordering as MemOrder};

/// Ordering semantics for atomics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ordering {
    Relaxed = 0,
    Release = 1,
    Acquire = 2,
    SeqCst = 3,
}

impl From<Ordering> for MemOrder {
    fn from(ord: Ordering) -> Self {
        match ord {
            Ordering::Relaxed => MemOrder::Relaxed,
            Ordering::Acquire => MemOrder::Acquire,
            Ordering::Release => MemOrder::Release,
            Ordering::SeqCst => MemOrder::SeqCst,
        }
    }
}

/// RISC-V Instructions
#[rustfmt::skip]
#[derive(Debug,Clone, Copy, PartialEq)]
pub enum RiscvInst {
    Illegal,

    // RV64I
    // Load instructions
    Lb { rd: u8, rs1: u8, imm: i32 },
    Lh { rd: u8, rs1: u8, imm: i32 },
    Lw { rd: u8, rs1: u8, imm: i32 },
    Ld { rd: u8, rs1: u8, imm: i32 },
    Lbu { rd: u8, rs1: u8, imm: i32 },
    Lhu { rd: u8, rs1: u8, imm: i32 },
    Lwu { rd: u8, rs1: u8, imm: i32 },

    // Fence instructions
    Fence,
    FenceI,

    // Immediate instructions
    Addi { rd: u8, rs1: u8, imm: i32 },
    Slli { rd: u8, rs1: u8, imm: i32 },
    Slti { rd: u8, rs1: u8, imm: i32 },
    Sltiu { rd: u8, rs1: u8, imm: i32 },
    Xori { rd: u8, rs1: u8, imm: i32 },
    Srli { rd: u8, rs1: u8, imm: i32 },
    Srai { rd: u8, rs1: u8, imm: i32 },
    Ori { rd: u8, rs1: u8, imm: i32 },
    Andi { rd: u8, rs1: u8, imm: i32 },

    // PC relative instructions
    Auipc { rd: u8, imm: i32 },
    Lui { rd: u8, imm: i32 },

    // RV64-I instructions
    Addiw { rd: u8, rs1: u8, imm: i32 },
    Slliw { rd: u8, rs1: u8, imm: i32 },
    Srliw { rd: u8, rs1: u8, imm: i32 },
    Sraiw { rd: u8, rs1: u8, imm: i32 },
    Addw { rd: u8, rs1: u8, rs2: u8 },
    Subw { rd: u8, rs1: u8, rs2: u8 },
    Sllw { rd: u8, rs1: u8, rs2: u8 },
    Srlw { rd: u8, rs1: u8, rs2: u8 },
    Sraw { rd: u8, rs1: u8, rs2: u8 },
    
    // Store instructions
    Sb { rs1: u8, rs2: u8, imm: i32 },
    Sh { rs1: u8, rs2: u8, imm: i32 },
    Sw { rs1: u8, rs2: u8, imm: i32 },
    Sd { rs1: u8, rs2: u8, imm: i32 },
    
    // Register instructions
    Add { rd: u8, rs1: u8, rs2: u8 },
    Sub { rd: u8, rs1: u8, rs2: u8 },
    Sll { rd: u8, rs1: u8, rs2: u8 },
    Slt { rd: u8, rs1: u8, rs2: u8 },
    Sltu { rd: u8, rs1: u8, rs2: u8 },
    Xor { rd: u8, rs1: u8, rs2: u8 },
    Srl { rd: u8, rs1: u8, rs2: u8 },
    Sra { rd: u8, rs1: u8, rs2: u8 },
    Or { rd: u8, rs1: u8, rs2: u8 },
    And { rd: u8, rs1: u8, rs2: u8 },


    // Branch instructions
    Beq { rs1: u8, rs2: u8, imm: i32 },
    Bne { rs1: u8, rs2: u8, imm: i32 },
    Blt { rs1: u8, rs2: u8, imm: i32 },
    Bge { rs1: u8, rs2: u8, imm: i32 },
    Bltu { rs1: u8, rs2: u8, imm: i32 },
    Bgeu { rs1: u8, rs2: u8, imm: i32 },

    // Jump instructions
    Jalr { rd: u8, rs1: u8, imm: i32 },
    Jal { rd: u8, imm: i32 },

    // System instructions
    Ecall,
    Ebreak,

    // CSR instructions
    Csrrw { rd: u8, rs1: u8, csr: Csr },
    Csrrs { rd: u8, rs1: u8, csr: Csr },
    Csrrc { rd: u8, rs1: u8, csr: Csr },
    Csrrwi { rd: u8, imm: u8, csr: Csr },
    Csrrsi { rd: u8, imm: u8, csr: Csr },
    Csrrci { rd: u8, imm: u8, csr: Csr },

    // Multiply Extension
    Mul { rd: u8, rs1: u8, rs2: u8 },
    Mulh { rd: u8, rs1: u8, rs2: u8 },
    Mulhsu { rd: u8, rs1: u8, rs2: u8 },
    Mulhu { rd: u8, rs1: u8, rs2: u8 },
    Div { rd: u8, rs1: u8, rs2: u8 },
    Divu { rd: u8, rs1: u8, rs2: u8 },
    Rem { rd: u8, rs1: u8, rs2: u8 },
    Remu { rd: u8, rs1: u8, rs2: u8 },
    
    // Multiply Extension (RV64)
    Mulw { rd: u8, rs1: u8, rs2: u8 },
    Divw { rd: u8, rs1: u8, rs2: u8 },
    Divuw { rd: u8, rs1: u8, rs2: u8 },
    Remw { rd: u8, rs1: u8, rs2: u8 },
    Remuw { rd: u8, rs1: u8, rs2: u8 },

    // Atomic Extension
    LrW { rd: u8, rs1: u8, aqrl: Ordering },
    LrD { rd: u8, rs1: u8, aqrl: Ordering },
    ScW { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    ScD { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmoswapW { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmoswapD { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmoaddW { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmoaddD { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmoxorW { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmoxorD { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmoandW { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmoandD { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmoorW { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmoorD { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmominW { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmominD { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmomaxW { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmomaxD { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmominuW { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmominuD { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmomaxuW { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },
    AmomaxuD { rd: u8, rs1: u8, rs2: u8, aqrl: Ordering },

    // Floating-Point Extension
    Flw { frd: u8, rs1: u8, imm: i32 },
    Fsw { rs1: u8, frs2: u8, imm: i32 },
    FaddS { frd: u8, frs1: u8, frs2: u8, rm: u8 },
    FsubS { frd: u8, frs1: u8, frs2: u8, rm: u8 },
    FmulS { frd: u8, frs1: u8, frs2: u8, rm: u8 },
    FdivS { frd: u8, frs1: u8, frs2: u8, rm: u8 },
    FsqrtS { frd: u8, frs1: u8, rm: u8 },
    FsgnjS { frd: u8, frs1: u8, frs2: u8 },
    FsgnjnS { frd: u8, frs1: u8, frs2: u8 },
    FsgnjxS { frd: u8, frs1: u8, frs2: u8 },
    FminS { frd: u8, frs1: u8, frs2: u8 },
    FmaxS { frd: u8, frs1: u8, frs2: u8 },
    FcvtWS { rd: u8, frs1: u8, rm: u8 },
    FcvtWuS { rd: u8, frs1: u8, rm: u8 },
    FcvtLS { rd: u8, frs1: u8, rm: u8 },
    FcvtLuS { rd: u8, frs1: u8, rm: u8 },
    FmvXW { rd: u8, frs1: u8 },
    FclassS { rd: u8, frs1: u8 },
    FeqS { rd: u8, frs1: u8, frs2: u8 },
    FltS { rd: u8, frs1: u8, frs2: u8 },
    FleS { rd: u8, frs1: u8, frs2: u8 },
    FcvtSW { frd: u8, rs1: u8, rm: u8 },
    FcvtSWu { frd: u8, rs1: u8, rm: u8 },
    FcvtSL { frd: u8, rs1: u8, rm: u8 },
    FcvtSLu { frd: u8, rs1: u8, rm: u8 },
    FmvWX { frd: u8, rs1: u8 },
    FmaddS { frd: u8, frs1: u8, frs2: u8, frs3: u8, rm: u8 },
    FmsubS { frd: u8, frs1: u8, frs2: u8, frs3: u8, rm: u8 },
    FnmsubS { frd: u8, frs1: u8, frs2: u8, frs3: u8, rm: u8 },
    FnmaddS { frd: u8, frs1: u8, frs2: u8, frs3: u8, rm: u8 },

    // Double-Precision Floating-Point Extension
    Fld { frd: u8, rs1: u8, imm: i32 },
    Fsd { rs1: u8, frs2: u8, imm: i32 },
    FaddD { frd: u8, frs1: u8, frs2: u8, rm: u8 },
    FsubD { frd: u8, frs1: u8, frs2: u8, rm: u8 },
    FmulD { frd: u8, frs1: u8, frs2: u8, rm: u8 },
    FdivD { frd: u8, frs1: u8, frs2: u8, rm: u8 },
    FsqrtD { frd: u8, frs1: u8, rm: u8 },
    FsgnjD { frd: u8, frs1: u8, frs2: u8 },
    FsgnjnD { frd: u8, frs1: u8, frs2: u8 },
    FsgnjxD { frd: u8, frs1: u8, frs2: u8 },
    FminD { frd: u8, frs1: u8, frs2: u8 },
    FmaxD { frd: u8, frs1: u8, frs2: u8 },
    FcvtSD { frd: u8, frs1: u8, rm: u8 },
    FcvtDS { frd: u8, frs1: u8, rm: u8 },
    FcvtWD { rd: u8, frs1: u8, rm: u8 },
    FcvtWuD { rd: u8, frs1: u8, rm: u8 },
    FcvtLD { rd: u8, frs1: u8, rm: u8 },
    FcvtLuD { rd: u8, frs1: u8, rm: u8 },
    FmvXD { rd: u8, frs1: u8 },
    FclassD { rd: u8, frs1: u8 },
    FeqD { rd: u8, frs1: u8, frs2: u8 },
    FltD { rd: u8, frs1: u8, frs2: u8 },
    FleD { rd: u8, frs1: u8, frs2: u8 },
    FcvtDW { frd: u8, rs1: u8, rm: u8 },
    FcvtDWu { frd: u8, rs1: u8, rm: u8 },
    FcvtDL { frd: u8, rs1: u8, rm: u8 },
    FcvtDLu { frd: u8, rs1: u8, rm: u8 },
    FmvDX { frd: u8, rs1: u8 },
    FmaddD { frd: u8, frs1: u8, frs2: u8, frs3: u8, rm: u8 },
    FmsubD { frd: u8, frs1: u8, frs2: u8, frs3: u8, rm: u8 },
    FnmsubD { frd: u8, frs1: u8, frs2: u8, frs3: u8, rm: u8 },
    FnmaddD { frd: u8, frs1: u8, frs2: u8, frs3: u8, rm: u8 },

    // Privileged instructions
    Mret,
    Sret,
    Wfi,
    SfenceVma { rs1: u8, rs2: u8 },
}

impl RiscvInst {
    /// Whether this instruction changes the control flow.
    pub fn changes_control_flow(&self) -> bool {
        match self {
            // Branch and jump instructions will definitely disrupt the control flow.
            RiscvInst::Beq {..} |
            RiscvInst::Bne {..} |
            RiscvInst::Blt {..} |
            RiscvInst::Bge {..} |
            RiscvInst::Bltu {..} |
            RiscvInst::Bgeu {..} |
            RiscvInst::Jalr {..} |
            RiscvInst::Jal {..} |
            // Return from ecall also changes control flow.
            RiscvInst::Mret |
            RiscvInst::Sret |
            // They always trigger faults
            RiscvInst::Ecall |
            RiscvInst::Ebreak |
            RiscvInst::Illegal |
            // fence.i might cause instruction cache to be invalidated. If the code executing is invalidated, then we need
            // to stop executing, so it is safer to treat it as special instruction at the moment.
            // sfence.vma has similar effects.
            RiscvInst::FenceI |
            RiscvInst::SfenceVma {..} => true,

            // Some CSRs need special treatment
            RiscvInst::Csrrw { csr, .. } |
            RiscvInst::Csrrs { csr, .. } |
            RiscvInst::Csrrc { csr, .. } |
            RiscvInst::Csrrwi { csr, .. } |
            RiscvInst::Csrrsi { csr, .. } |
            RiscvInst::Csrrci { csr, .. } => match *csr {
                // A common way of using basic blocks is to `batch' instret and pc increment. So if CSR to be accessed is
                // instret, consider it as special.
                Csr::Instret |
                Csr::Instreth |
                // SATP shouldn't belong here, but somehow Linux assumes setting SATP changes
                // addressing mode immediately...
                Csr::Satp => true,
                _ => false,
            }
            _ => false,
        }
    }

    /// Get the minimal privilege level required to execute the instruction.
    pub fn min_prv_level(self) -> u8 {
        match self {
            RiscvInst::Csrrw { csr, .. }
            | RiscvInst::Csrrs { csr, .. }
            | RiscvInst::Csrrc { csr, .. }
            | RiscvInst::Csrrwi { csr, .. }
            | RiscvInst::Csrrsi { csr, .. }
            | RiscvInst::Csrrci { csr, .. } => csr.min_prv_level(),
            RiscvInst::Mret => 3,
            RiscvInst::Sret | RiscvInst::Wfi | RiscvInst::SfenceVma { .. } => 1,
            _ => 0,
        }
    }

    /// Retrieve the RD, RS1, RS2 from the op.
    /// If the operation does not use RD, RS1 or RS2, return 0 instead.
    pub fn regs(self) -> (u8, u8, u8) {
        match self {
            RiscvInst::Illegal => (0, 0, 0),

            RiscvInst::Lui { rd, .. } | RiscvInst::Auipc { rd, .. } => (rd, 0, 0),

            RiscvInst::Jal { rd, .. } => (rd, 0, 0),

            RiscvInst::Beq { rs1, rs2, .. }
            | RiscvInst::Bne { rs1, rs2, .. }
            | RiscvInst::Blt { rs1, rs2, .. }
            | RiscvInst::Bge { rs1, rs2, .. }
            | RiscvInst::Bltu { rs1, rs2, .. }
            | RiscvInst::Bgeu { rs1, rs2, .. } => (0, rs1, rs2),

            RiscvInst::Lb { rd, rs1, .. }
            | RiscvInst::Lh { rd, rs1, .. }
            | RiscvInst::Lw { rd, rs1, .. }
            | RiscvInst::Ld { rd, rs1, .. }
            | RiscvInst::Lbu { rd, rs1, .. }
            | RiscvInst::Lhu { rd, rs1, .. }
            | RiscvInst::Lwu { rd, rs1, .. } => (rd, rs1, 0),

            RiscvInst::Jalr { rd, rs1, .. } => (rd, rs1, 0),

            RiscvInst::Fence => (0, 0, 0),
            RiscvInst::FenceI => (0, 0, 0),

            RiscvInst::Ecall | RiscvInst::Ebreak => (0, 0, 0),

            RiscvInst::Mret | RiscvInst::Sret => (0, 0, 0),

            RiscvInst::Wfi => (0, 0, 0),
            RiscvInst::SfenceVma { rs1, rs2 } => (0, rs1, rs2),

            RiscvInst::Sb { rs1, rs2, .. }
            | RiscvInst::Sh { rs1, rs2, .. }
            | RiscvInst::Sw { rs1, rs2, .. }
            | RiscvInst::Sd { rs1, rs2, .. } => (0, rs1, rs2),

            RiscvInst::Addi { rd, rs1, .. }
            | RiscvInst::Slti { rd, rs1, .. }
            | RiscvInst::Sltiu { rd, rs1, .. }
            | RiscvInst::Xori { rd, rs1, .. }
            | RiscvInst::Ori { rd, rs1, .. }
            | RiscvInst::Andi { rd, rs1, .. }
            | RiscvInst::Addiw { rd, rs1, .. }
            | RiscvInst::Slli { rd, rs1, .. }
            | RiscvInst::Srli { rd, rs1, .. }
            | RiscvInst::Srai { rd, rs1, .. }
            | RiscvInst::Slliw { rd, rs1, .. }
            | RiscvInst::Srliw { rd, rs1, .. }
            | RiscvInst::Sraiw { rd, rs1, .. } => (rd, rs1, 0),

            RiscvInst::Add { rd, rs1, rs2 }
            | RiscvInst::Sub { rd, rs1, rs2 }
            | RiscvInst::Sll { rd, rs1, rs2 }
            | RiscvInst::Slt { rd, rs1, rs2 }
            | RiscvInst::Sltu { rd, rs1, rs2 }
            | RiscvInst::Xor { rd, rs1, rs2 }
            | RiscvInst::Srl { rd, rs1, rs2 }
            | RiscvInst::Sra { rd, rs1, rs2 }
            | RiscvInst::Or { rd, rs1, rs2 }
            | RiscvInst::And { rd, rs1, rs2 }
            | RiscvInst::Addw { rd, rs1, rs2 }
            | RiscvInst::Subw { rd, rs1, rs2 }
            | RiscvInst::Sllw { rd, rs1, rs2 }
            | RiscvInst::Srlw { rd, rs1, rs2 }
            | RiscvInst::Sraw { rd, rs1, rs2 } => (rd, rs1, rs2),

            RiscvInst::Mul { rd, rs1, rs2 } => (rd, rs1, rs2),

            RiscvInst::Mulh { rd, rs1, rs2 }
            | RiscvInst::Mulhsu { rd, rs1, rs2 }
            | RiscvInst::Mulhu { rd, rs1, rs2 } => (rd, rs1, rs2),

            RiscvInst::Div { rd, rs1, rs2 }
            | RiscvInst::Divu { rd, rs1, rs2 }
            | RiscvInst::Rem { rd, rs1, rs2 }
            | RiscvInst::Remu { rd, rs1, rs2 } => (rd, rs1, rs2),

            RiscvInst::Mulw { rd, rs1, rs2 } => (rd, rs1, rs2),

            RiscvInst::Divw { rd, rs1, rs2 }
            | RiscvInst::Divuw { rd, rs1, rs2 }
            | RiscvInst::Remw { rd, rs1, rs2 }
            | RiscvInst::Remuw { rd, rs1, rs2 } => (rd, rs1, rs2),

            RiscvInst::Csrrw { rd, rs1, .. }
            | RiscvInst::Csrrs { rd, rs1, .. }
            | RiscvInst::Csrrc { rd, rs1, .. } => (rd, rs1, 0),

            RiscvInst::Csrrwi { rd, .. }
            | RiscvInst::Csrrsi { rd, .. }
            | RiscvInst::Csrrci { rd, .. } => (rd, 0, 0),

            RiscvInst::LrW { rd, rs1, .. } | RiscvInst::LrD { rd, rs1, .. } => (rd, rs1, 0),

            RiscvInst::ScW { rd, rs1, rs2, .. }
            | RiscvInst::ScD { rd, rs1, rs2, .. }
            | RiscvInst::AmoswapW { rd, rs1, rs2, .. }
            | RiscvInst::AmoswapD { rd, rs1, rs2, .. }
            | RiscvInst::AmoaddW { rd, rs1, rs2, .. }
            | RiscvInst::AmoaddD { rd, rs1, rs2, .. }
            | RiscvInst::AmoxorW { rd, rs1, rs2, .. }
            | RiscvInst::AmoxorD { rd, rs1, rs2, .. }
            | RiscvInst::AmoandW { rd, rs1, rs2, .. }
            | RiscvInst::AmoandD { rd, rs1, rs2, .. }
            | RiscvInst::AmoorW { rd, rs1, rs2, .. }
            | RiscvInst::AmoorD { rd, rs1, rs2, .. }
            | RiscvInst::AmominW { rd, rs1, rs2, .. }
            | RiscvInst::AmominD { rd, rs1, rs2, .. }
            | RiscvInst::AmomaxW { rd, rs1, rs2, .. }
            | RiscvInst::AmomaxD { rd, rs1, rs2, .. }
            | RiscvInst::AmominuW { rd, rs1, rs2, .. }
            | RiscvInst::AmominuD { rd, rs1, rs2, .. }
            | RiscvInst::AmomaxuW { rd, rs1, rs2, .. }
            | RiscvInst::AmomaxuD { rd, rs1, rs2, .. } => (rd, rs1, rs2),

            RiscvInst::Flw { rs1, .. } | RiscvInst::Fld { rs1, .. } => (0, rs1, 0),

            RiscvInst::Fsw { rs1, .. } | RiscvInst::Fsd { rs1, .. } => (0, rs1, 0),

            RiscvInst::FaddS { .. }
            | RiscvInst::FsubS { .. }
            | RiscvInst::FmulS { .. }
            | RiscvInst::FdivS { .. }
            | RiscvInst::FsgnjS { .. }
            | RiscvInst::FsgnjnS { .. }
            | RiscvInst::FsgnjxS { .. }
            | RiscvInst::FminS { .. }
            | RiscvInst::FmaxS { .. }
            | RiscvInst::FaddD { .. }
            | RiscvInst::FsubD { .. }
            | RiscvInst::FmulD { .. }
            | RiscvInst::FdivD { .. }
            | RiscvInst::FsgnjD { .. }
            | RiscvInst::FsgnjnD { .. }
            | RiscvInst::FsgnjxD { .. }
            | RiscvInst::FminD { .. }
            | RiscvInst::FmaxD { .. } => (0, 0, 0),

            RiscvInst::FsqrtS { .. }
            | RiscvInst::FsqrtD { .. }
            | RiscvInst::FcvtSD { .. }
            | RiscvInst::FcvtDS { .. } => (0, 0, 0),

            RiscvInst::FcvtWS { rd, .. }
            | RiscvInst::FcvtWuS { rd, .. }
            | RiscvInst::FcvtLS { rd, .. }
            | RiscvInst::FcvtLuS { rd, .. }
            | RiscvInst::FmvXW { rd, .. }
            | RiscvInst::FclassS { rd, .. }
            | RiscvInst::FcvtWD { rd, .. }
            | RiscvInst::FcvtWuD { rd, .. }
            | RiscvInst::FcvtLD { rd, .. }
            | RiscvInst::FcvtLuD { rd, .. }
            | RiscvInst::FmvXD { rd, .. }
            | RiscvInst::FclassD { rd, .. } => (rd, 0, 0),

            RiscvInst::FcvtSW { rs1, .. }
            | RiscvInst::FcvtSWu { rs1, .. }
            | RiscvInst::FcvtSL { rs1, .. }
            | RiscvInst::FcvtSLu { rs1, .. }
            | RiscvInst::FmvWX { rs1, .. }
            | RiscvInst::FcvtDW { rs1, .. }
            | RiscvInst::FcvtDWu { rs1, .. }
            | RiscvInst::FcvtDL { rs1, .. }
            | RiscvInst::FcvtDLu { rs1, .. }
            | RiscvInst::FmvDX { rs1, .. } => (0, rs1, 0),

            RiscvInst::FeqS { rd, .. }
            | RiscvInst::FltS { rd, .. }
            | RiscvInst::FleS { rd, .. }
            | RiscvInst::FeqD { rd, .. }
            | RiscvInst::FltD { rd, .. }
            | RiscvInst::FleD { rd, .. } => (rd, 0, 0),

            RiscvInst::FmaddS { .. }
            | RiscvInst::FmsubS { .. }
            | RiscvInst::FnmsubS { .. }
            | RiscvInst::FnmaddS { .. }
            | RiscvInst::FmaddD { .. }
            | RiscvInst::FmsubD { .. }
            | RiscvInst::FnmsubD { .. }
            | RiscvInst::FnmaddD { .. } => (0, 0, 0),
        }
    }

    /// Return the mnemonic of this op withouth the suffix.
    pub fn mnemonic(&self) -> &'static str {
        match *self {
            RiscvInst::Illegal { .. } => "illegal",
            RiscvInst::Lb { .. } => "lb",
            RiscvInst::Lh { .. } => "lh",
            RiscvInst::Lw { .. } => "lw",
            RiscvInst::Ld { .. } => "ld",
            RiscvInst::Lbu { .. } => "lbu",
            RiscvInst::Lhu { .. } => "lhu",
            RiscvInst::Lwu { .. } => "lwu",
            RiscvInst::Fence { .. } => "fence",
            RiscvInst::FenceI { .. } => "fence.i",
            RiscvInst::Addi { .. } => "addi",
            RiscvInst::Slli { .. } => "slli",
            RiscvInst::Slti { .. } => "slti",
            RiscvInst::Sltiu { .. } => "sltiu",
            RiscvInst::Xori { .. } => "xori",
            RiscvInst::Srli { .. } => "srli",
            RiscvInst::Srai { .. } => "srai",
            RiscvInst::Ori { .. } => "ori",
            RiscvInst::Andi { .. } => "andi",
            RiscvInst::Auipc { .. } => "auipc",
            RiscvInst::Addiw { .. } => "addiw",
            RiscvInst::Slliw { .. } => "slliw",
            RiscvInst::Srliw { .. } => "srliw",
            RiscvInst::Sraiw { .. } => "sraiw",
            RiscvInst::Sb { .. } => "sb",
            RiscvInst::Sh { .. } => "sh",
            RiscvInst::Sw { .. } => "sw",
            RiscvInst::Sd { .. } => "sd",
            RiscvInst::Add { .. } => "add",
            RiscvInst::Sub { .. } => "sub",
            RiscvInst::Sll { .. } => "sll",
            RiscvInst::Slt { .. } => "slt",
            RiscvInst::Sltu { .. } => "sltu",
            RiscvInst::Xor { .. } => "xor",
            RiscvInst::Srl { .. } => "srl",
            RiscvInst::Sra { .. } => "sra",
            RiscvInst::Or { .. } => "or",
            RiscvInst::And { .. } => "and",
            RiscvInst::Lui { .. } => "lui",
            RiscvInst::Addw { .. } => "addw",
            RiscvInst::Subw { .. } => "subw",
            RiscvInst::Sllw { .. } => "sllw",
            RiscvInst::Srlw { .. } => "srlw",
            RiscvInst::Sraw { .. } => "sraw",
            RiscvInst::Beq { .. } => "beq",
            RiscvInst::Bne { .. } => "bne",
            RiscvInst::Blt { .. } => "blt",
            RiscvInst::Bge { .. } => "bge",
            RiscvInst::Bltu { .. } => "bltu",
            RiscvInst::Bgeu { .. } => "bgeu",
            RiscvInst::Jalr { .. } => "jalr",
            RiscvInst::Jal { .. } => "jal",
            RiscvInst::Ecall { .. } => "ecall",
            RiscvInst::Ebreak { .. } => "ebreak",
            RiscvInst::Csrrw { .. } => "csrrw",
            RiscvInst::Csrrs { .. } => "csrrs",
            RiscvInst::Csrrc { .. } => "csrrc",
            RiscvInst::Csrrwi { .. } => "csrrwi",
            RiscvInst::Csrrsi { .. } => "csrrsi",
            RiscvInst::Csrrci { .. } => "csrrci",
            RiscvInst::Mul { .. } => "mul",
            RiscvInst::Mulh { .. } => "mulh",
            RiscvInst::Mulhsu { .. } => "mulhsu",
            RiscvInst::Mulhu { .. } => "mulhu",
            RiscvInst::Div { .. } => "div",
            RiscvInst::Divu { .. } => "divu",
            RiscvInst::Rem { .. } => "rem",
            RiscvInst::Remu { .. } => "remu",
            RiscvInst::Mulw { .. } => "mulw",
            RiscvInst::Divw { .. } => "divw",
            RiscvInst::Divuw { .. } => "divuw",
            RiscvInst::Remw { .. } => "remw",
            RiscvInst::Remuw { .. } => "remuw",
            RiscvInst::LrW { .. } => "lr.w",
            RiscvInst::LrD { .. } => "lr.d",
            RiscvInst::ScW { .. } => "sc.w",
            RiscvInst::ScD { .. } => "sc.d",
            RiscvInst::AmoswapW { .. } => "amoswap.w",
            RiscvInst::AmoswapD { .. } => "amoswap.d",
            RiscvInst::AmoaddW { .. } => "amoadd.w",
            RiscvInst::AmoaddD { .. } => "amoadd.d",
            RiscvInst::AmoxorW { .. } => "amoxor.w",
            RiscvInst::AmoxorD { .. } => "amoxor.d",
            RiscvInst::AmoandW { .. } => "amoand.w",
            RiscvInst::AmoandD { .. } => "amoand.d",
            RiscvInst::AmoorW { .. } => "amoor.w",
            RiscvInst::AmoorD { .. } => "amoor.d",
            RiscvInst::AmominW { .. } => "amomin.w",
            RiscvInst::AmominD { .. } => "amomin.d",
            RiscvInst::AmomaxW { .. } => "amomax.w",
            RiscvInst::AmomaxD { .. } => "amomax.d",
            RiscvInst::AmominuW { .. } => "amominu.w",
            RiscvInst::AmominuD { .. } => "amominu.d",
            RiscvInst::AmomaxuW { .. } => "amomaxu.w",
            RiscvInst::AmomaxuD { .. } => "amomaxu.d",
            RiscvInst::Flw { .. } => "flw",
            RiscvInst::Fsw { .. } => "fsw",
            RiscvInst::FaddS { .. } => "fadd.s",
            RiscvInst::FsubS { .. } => "fsub.s",
            RiscvInst::FmulS { .. } => "fmul.s",
            RiscvInst::FdivS { .. } => "fdiv.s",
            RiscvInst::FsqrtS { .. } => "fsqrt.s",
            RiscvInst::FsgnjS { .. } => "fsgnj.s",
            RiscvInst::FsgnjnS { .. } => "fsgnjn.s",
            RiscvInst::FsgnjxS { .. } => "fsgnjx.s",
            RiscvInst::FminS { .. } => "fmin.s",
            RiscvInst::FmaxS { .. } => "fmax.s",
            RiscvInst::FcvtWS { .. } => "fcvt.w.s",
            RiscvInst::FcvtWuS { .. } => "fcvt.wu.s",
            RiscvInst::FcvtLS { .. } => "fcvt.l.s",
            RiscvInst::FcvtLuS { .. } => "fcvt.lu.s",
            RiscvInst::FmvXW { .. } => "fmv.x.w",
            RiscvInst::FclassS { .. } => "fclass.s",
            RiscvInst::FeqS { .. } => "feq.s",
            RiscvInst::FltS { .. } => "flt.s",
            RiscvInst::FleS { .. } => "fle.s",
            RiscvInst::FcvtSW { .. } => "fcvt.s.w",
            RiscvInst::FcvtSWu { .. } => "fcvt.s.wu",
            RiscvInst::FcvtSL { .. } => "fcvt.s.l",
            RiscvInst::FcvtSLu { .. } => "fcvt.s.lu",
            RiscvInst::FmvWX { .. } => "fmv.w.x",
            RiscvInst::FmaddS { .. } => "fmadd.s",
            RiscvInst::FmsubS { .. } => "fmsub.s",
            RiscvInst::FnmsubS { .. } => "fnmsub.s",
            RiscvInst::FnmaddS { .. } => "fnmadd.s",
            RiscvInst::Fld { .. } => "fld",
            RiscvInst::Fsd { .. } => "fsd",
            RiscvInst::FaddD { .. } => "fadd.d",
            RiscvInst::FsubD { .. } => "fsub.d",
            RiscvInst::FmulD { .. } => "fmul.d",
            RiscvInst::FdivD { .. } => "fdiv.d",
            RiscvInst::FsqrtD { .. } => "fsqrt.d",
            RiscvInst::FsgnjD { .. } => "fsgnj.d",
            RiscvInst::FsgnjnD { .. } => "fsgnjn.d",
            RiscvInst::FsgnjxD { .. } => "fsgnjx.d",
            RiscvInst::FminD { .. } => "fmin.d",
            RiscvInst::FmaxD { .. } => "fmax.d",
            RiscvInst::FcvtSD { .. } => "fcvt.s.d",
            RiscvInst::FcvtDS { .. } => "fcvt.d.s",
            RiscvInst::FcvtWD { .. } => "fcvt.w.d",
            RiscvInst::FcvtWuD { .. } => "fcvt.wu.d",
            RiscvInst::FcvtLD { .. } => "fcvt.l.d",
            RiscvInst::FcvtLuD { .. } => "fcvt.lu.d",
            RiscvInst::FmvXD { .. } => "fmv.x.d",
            RiscvInst::FclassD { .. } => "fclass.d",
            RiscvInst::FeqD { .. } => "feq.d",
            RiscvInst::FltD { .. } => "flt.d",
            RiscvInst::FleD { .. } => "fle.d",
            RiscvInst::FcvtDW { .. } => "fcvt.d.w",
            RiscvInst::FcvtDWu { .. } => "fcvt.d.wu",
            RiscvInst::FcvtDL { .. } => "fcvt.d.l",
            RiscvInst::FcvtDLu { .. } => "fcvt.d.lu",
            RiscvInst::FmvDX { .. } => "fmv.d.x",
            RiscvInst::FmaddD { .. } => "fmadd.d",
            RiscvInst::FmsubD { .. } => "fmsub.d",
            RiscvInst::FnmsubD { .. } => "fnmsub.d",
            RiscvInst::FnmaddD { .. } => "fnmadd.d",
            RiscvInst::Mret { .. } => "mret",
            RiscvInst::Sret { .. } => "sret",
            RiscvInst::Wfi { .. } => "wfi",
            RiscvInst::SfenceVma { .. } => "sfence.vma",
        }
    }

    /// Return the suffix annotation of this op. This returns ".aqrl" for "amoswap.w.aqrl"
    pub fn suffix(&self) -> &'static str {
        match *self {
            RiscvInst::LrW { aqrl, .. }
            | RiscvInst::LrD { aqrl, .. }
            | RiscvInst::ScW { aqrl, .. }
            | RiscvInst::ScD { aqrl, .. }
            | RiscvInst::AmoswapW { aqrl, .. }
            | RiscvInst::AmoswapD { aqrl, .. }
            | RiscvInst::AmoaddW { aqrl, .. }
            | RiscvInst::AmoaddD { aqrl, .. }
            | RiscvInst::AmoxorW { aqrl, .. }
            | RiscvInst::AmoxorD { aqrl, .. }
            | RiscvInst::AmoandW { aqrl, .. }
            | RiscvInst::AmoandD { aqrl, .. }
            | RiscvInst::AmoorW { aqrl, .. }
            | RiscvInst::AmoorD { aqrl, .. }
            | RiscvInst::AmominW { aqrl, .. }
            | RiscvInst::AmominD { aqrl, .. }
            | RiscvInst::AmomaxW { aqrl, .. }
            | RiscvInst::AmomaxD { aqrl, .. }
            | RiscvInst::AmominuW { aqrl, .. }
            | RiscvInst::AmominuD { aqrl, .. }
            | RiscvInst::AmomaxuW { aqrl, .. }
            | RiscvInst::AmomaxuD { aqrl, .. } => match aqrl {
                Ordering::Relaxed => "",
                Ordering::Acquire => ".aq",
                Ordering::Release => ".rl",
                Ordering::SeqCst => ".aqrl",
            },
            _ => "",
        }
    }

    /// Print the instruction with optional pc information.
    fn print(&self, fmt: &mut fmt::Formatter, pc: Option<u64>) -> fmt::Result {
        let mnemonic = self.mnemonic();
        let suffix = self.suffix();
        let len = mnemonic.len() + suffix.len();
        write!(fmt, "{}{}", mnemonic, suffix)?;

        // Pad to 8-byte align. At least pad 1 space.
        write!(fmt, "{:1$}", "", 8 - len % 8)?;

        match *self {
            RiscvInst::Illegal => (),

            RiscvInst::Lui { rd, imm } | RiscvInst::Auipc { rd, imm } => {
                write!(fmt, "{}, {:#x}", register_name(rd), (imm as u32) >> 12)?
            }

            RiscvInst::Jal { rd, imm } => {
                let (sign, uimm) = if imm < 0 { ('-', -imm) } else { ('+', imm) };
                write!(fmt, "{}, pc {} {}", register_name(rd), sign, uimm)?;
                if let Some(pc) = pc {
                    let target_pc = pc.wrapping_add(imm as u64);
                    write!(fmt, " <{:x}>", target_pc)?;
                }
            }

            RiscvInst::Beq { rs1, rs2, imm }
            | RiscvInst::Bne { rs1, rs2, imm }
            | RiscvInst::Blt { rs1, rs2, imm }
            | RiscvInst::Bge { rs1, rs2, imm }
            | RiscvInst::Bltu { rs1, rs2, imm }
            | RiscvInst::Bgeu { rs1, rs2, imm } => {
                let (sign, uimm) = if imm < 0 { ('-', -imm) } else { ('+', imm) };
                write!(
                    fmt,
                    "{}, {}, pc {} {}",
                    register_name(rs1),
                    register_name(rs2),
                    sign,
                    uimm
                )?;
                if let Some(pc) = pc {
                    let target_pc = pc.wrapping_add(imm as u64);
                    write!(fmt, " <{:x}>", target_pc)?;
                }
            }

            RiscvInst::Lb { rd, rs1, imm }
            | RiscvInst::Lh { rd, rs1, imm }
            | RiscvInst::Lw { rd, rs1, imm }
            | RiscvInst::Ld { rd, rs1, imm }
            | RiscvInst::Lbu { rd, rs1, imm }
            | RiscvInst::Lhu { rd, rs1, imm }
            | RiscvInst::Lwu { rd, rs1, imm }
            | RiscvInst::Jalr { rd, rs1, imm } => write!(
                fmt,
                "{}, {}({})",
                register_name(rd),
                imm,
                register_name(rs1)
            )?,

            RiscvInst::Fence
            | RiscvInst::FenceI
            | RiscvInst::Ecall
            | RiscvInst::Ebreak
            | RiscvInst::Mret
            | RiscvInst::Sret
            | RiscvInst::Wfi => (),
            RiscvInst::SfenceVma { rs1, rs2 } => {
                write!(fmt, "{}, {}", register_name(rs1), register_name(rs2))?
            }

            RiscvInst::Sb { rs1, rs2, imm }
            | RiscvInst::Sh { rs1, rs2, imm }
            | RiscvInst::Sw { rs1, rs2, imm }
            | RiscvInst::Sd { rs1, rs2, imm } => write!(
                fmt,
                "{}, {}({})",
                register_name(rs2),
                imm,
                register_name(rs1)
            )?,

            RiscvInst::Addi { rd, rs1, imm }
            | RiscvInst::Slti { rd, rs1, imm }
            | RiscvInst::Sltiu { rd, rs1, imm }
            | RiscvInst::Xori { rd, rs1, imm }
            | RiscvInst::Ori { rd, rs1, imm }
            | RiscvInst::Andi { rd, rs1, imm }
            | RiscvInst::Addiw { rd, rs1, imm }
            | RiscvInst::Slli { rd, rs1, imm }
            | RiscvInst::Srli { rd, rs1, imm }
            | RiscvInst::Srai { rd, rs1, imm }
            | RiscvInst::Slliw { rd, rs1, imm }
            | RiscvInst::Srliw { rd, rs1, imm }
            | RiscvInst::Sraiw { rd, rs1, imm } => write!(
                fmt,
                "{}, {}, {}",
                register_name(rd),
                register_name(rs1),
                imm
            )?,

            RiscvInst::Add { rd, rs1, rs2 }
            | RiscvInst::Sub { rd, rs1, rs2 }
            | RiscvInst::Sll { rd, rs1, rs2 }
            | RiscvInst::Slt { rd, rs1, rs2 }
            | RiscvInst::Sltu { rd, rs1, rs2 }
            | RiscvInst::Xor { rd, rs1, rs2 }
            | RiscvInst::Srl { rd, rs1, rs2 }
            | RiscvInst::Sra { rd, rs1, rs2 }
            | RiscvInst::Or { rd, rs1, rs2 }
            | RiscvInst::And { rd, rs1, rs2 }
            | RiscvInst::Addw { rd, rs1, rs2 }
            | RiscvInst::Subw { rd, rs1, rs2 }
            | RiscvInst::Sllw { rd, rs1, rs2 }
            | RiscvInst::Srlw { rd, rs1, rs2 }
            | RiscvInst::Sraw { rd, rs1, rs2 }
            | RiscvInst::Mul { rd, rs1, rs2 }
            | RiscvInst::Mulh { rd, rs1, rs2 }
            | RiscvInst::Mulhsu { rd, rs1, rs2 }
            | RiscvInst::Mulhu { rd, rs1, rs2 }
            | RiscvInst::Div { rd, rs1, rs2 }
            | RiscvInst::Divu { rd, rs1, rs2 }
            | RiscvInst::Rem { rd, rs1, rs2 }
            | RiscvInst::Remu { rd, rs1, rs2 }
            | RiscvInst::Mulw { rd, rs1, rs2 }
            | RiscvInst::Divw { rd, rs1, rs2 }
            | RiscvInst::Divuw { rd, rs1, rs2 }
            | RiscvInst::Remw { rd, rs1, rs2 }
            | RiscvInst::Remuw { rd, rs1, rs2 } => write!(
                fmt,
                "{}, {}, {}",
                register_name(rd),
                register_name(rs1),
                register_name(rs2)
            )?,

            RiscvInst::Csrrw { rd, rs1, csr }
            | RiscvInst::Csrrs { rd, rs1, csr }
            | RiscvInst::Csrrc { rd, rs1, csr } => write!(
                fmt,
                "{}, #{}, {}",
                register_name(rd),
                csr,
                register_name(rs1)
            )?,

            RiscvInst::Csrrwi { rd, imm, csr }
            | RiscvInst::Csrrsi { rd, imm, csr }
            | RiscvInst::Csrrci { rd, imm, csr } => {
                write!(fmt, "{}, #{}, {}", register_name(rd), csr, imm)?
            }

            RiscvInst::LrW { rd, rs1, .. } | RiscvInst::LrD { rd, rs1, .. } => {
                write!(fmt, "{}, ({})", register_name(rd), register_name(rs1))?
            }

            RiscvInst::ScW { rd, rs1, rs2, .. }
            | RiscvInst::ScD { rd, rs1, rs2, .. }
            | RiscvInst::AmoswapW { rd, rs1, rs2, .. }
            | RiscvInst::AmoswapD { rd, rs1, rs2, .. }
            | RiscvInst::AmoaddW { rd, rs1, rs2, .. }
            | RiscvInst::AmoaddD { rd, rs1, rs2, .. }
            | RiscvInst::AmoxorW { rd, rs1, rs2, .. }
            | RiscvInst::AmoxorD { rd, rs1, rs2, .. }
            | RiscvInst::AmoandW { rd, rs1, rs2, .. }
            | RiscvInst::AmoandD { rd, rs1, rs2, .. }
            | RiscvInst::AmoorW { rd, rs1, rs2, .. }
            | RiscvInst::AmoorD { rd, rs1, rs2, .. }
            | RiscvInst::AmominW { rd, rs1, rs2, .. }
            | RiscvInst::AmominD { rd, rs1, rs2, .. }
            | RiscvInst::AmomaxW { rd, rs1, rs2, .. }
            | RiscvInst::AmomaxD { rd, rs1, rs2, .. }
            | RiscvInst::AmominuW { rd, rs1, rs2, .. }
            | RiscvInst::AmominuD { rd, rs1, rs2, .. }
            | RiscvInst::AmomaxuW { rd, rs1, rs2, .. }
            | RiscvInst::AmomaxuD { rd, rs1, rs2, .. } => write!(
                fmt,
                "{}, {}, ({})",
                register_name(rd),
                register_name(rs2),
                register_name(rs1)
            )?,

            RiscvInst::Flw { frd, rs1, imm } | RiscvInst::Fld { frd, rs1, imm } => {
                write!(fmt, "f{}, {}({})", frd, imm, register_name(rs1))?
            }

            RiscvInst::Fsw { rs1, frs2, imm } | RiscvInst::Fsd { rs1, frs2, imm } => {
                write!(fmt, "f{}, {}({})", frs2, imm, register_name(rs1))?
            }

            RiscvInst::FaddS {
                frd, frs1, frs2, ..
            }
            | RiscvInst::FsubS {
                frd, frs1, frs2, ..
            }
            | RiscvInst::FmulS {
                frd, frs1, frs2, ..
            }
            | RiscvInst::FdivS {
                frd, frs1, frs2, ..
            }
            | RiscvInst::FsgnjS { frd, frs1, frs2 }
            | RiscvInst::FsgnjnS { frd, frs1, frs2 }
            | RiscvInst::FsgnjxS { frd, frs1, frs2 }
            | RiscvInst::FminS { frd, frs1, frs2 }
            | RiscvInst::FmaxS { frd, frs1, frs2 }
            | RiscvInst::FaddD {
                frd, frs1, frs2, ..
            }
            | RiscvInst::FsubD {
                frd, frs1, frs2, ..
            }
            | RiscvInst::FmulD {
                frd, frs1, frs2, ..
            }
            | RiscvInst::FdivD {
                frd, frs1, frs2, ..
            }
            | RiscvInst::FsgnjD { frd, frs1, frs2 }
            | RiscvInst::FsgnjnD { frd, frs1, frs2 }
            | RiscvInst::FsgnjxD { frd, frs1, frs2 }
            | RiscvInst::FminD { frd, frs1, frs2 }
            | RiscvInst::FmaxD { frd, frs1, frs2 } => {
                write!(fmt, "f{}, f{}, f{}", frd, frs1, frs2)?
            }

            RiscvInst::FsqrtS { frd, frs1, .. }
            | RiscvInst::FsqrtD { frd, frs1, .. }
            | RiscvInst::FcvtSD { frd, frs1, .. }
            | RiscvInst::FcvtDS { frd, frs1, .. } => write!(fmt, "f{}, f{}", frd, frs1)?,

            RiscvInst::FcvtWS { rd, frs1, .. }
            | RiscvInst::FcvtWuS { rd, frs1, .. }
            | RiscvInst::FcvtLS { rd, frs1, .. }
            | RiscvInst::FcvtLuS { rd, frs1, .. }
            | RiscvInst::FmvXW { rd, frs1 }
            | RiscvInst::FclassS { rd, frs1 }
            | RiscvInst::FcvtWD { rd, frs1, .. }
            | RiscvInst::FcvtWuD { rd, frs1, .. }
            | RiscvInst::FcvtLD { rd, frs1, .. }
            | RiscvInst::FcvtLuD { rd, frs1, .. }
            | RiscvInst::FmvXD { rd, frs1 }
            | RiscvInst::FclassD { rd, frs1 } => write!(fmt, "{}, f{}", register_name(rd), frs1)?,

            RiscvInst::FcvtSW { frd, rs1, .. }
            | RiscvInst::FcvtSWu { frd, rs1, .. }
            | RiscvInst::FcvtSL { frd, rs1, .. }
            | RiscvInst::FcvtSLu { frd, rs1, .. }
            | RiscvInst::FmvWX { frd, rs1 }
            | RiscvInst::FcvtDW { frd, rs1, .. }
            | RiscvInst::FcvtDWu { frd, rs1, .. }
            | RiscvInst::FcvtDL { frd, rs1, .. }
            | RiscvInst::FcvtDLu { frd, rs1, .. }
            | RiscvInst::FmvDX { frd, rs1 } => write!(fmt, "f{}, {}", frd, register_name(rs1))?,

            RiscvInst::FeqS { rd, frs1, frs2 }
            | RiscvInst::FltS { rd, frs1, frs2 }
            | RiscvInst::FleS { rd, frs1, frs2 }
            | RiscvInst::FeqD { rd, frs1, frs2 }
            | RiscvInst::FltD { rd, frs1, frs2 }
            | RiscvInst::FleD { rd, frs1, frs2 } => {
                write!(fmt, "{}, f{}, f{}", register_name(rd), frs1, frs2)?
            }

            RiscvInst::FmaddS {
                frd,
                frs1,
                frs2,
                frs3,
                ..
            }
            | RiscvInst::FmsubS {
                frd,
                frs1,
                frs2,
                frs3,
                ..
            }
            | RiscvInst::FnmsubS {
                frd,
                frs1,
                frs2,
                frs3,
                ..
            }
            | RiscvInst::FnmaddS {
                frd,
                frs1,
                frs2,
                frs3,
                ..
            }
            | RiscvInst::FmaddD {
                frd,
                frs1,
                frs2,
                frs3,
                ..
            }
            | RiscvInst::FmsubD {
                frd,
                frs1,
                frs2,
                frs3,
                ..
            }
            | RiscvInst::FnmsubD {
                frd,
                frs1,
                frs2,
                frs3,
                ..
            }
            | RiscvInst::FnmaddD {
                frd,
                frs1,
                frs2,
                frs3,
                ..
            } => write!(fmt, "f{}, f{}, f{}, f{}", frd, frs1, frs2, frs3)?,
        }

        Ok(())
    }

    /// Pretty-print the assembly with program counter and binary instrumentation
    pub fn pretty_print<'a>(&'a self, pc: u64, bits: u32) -> impl fmt::Display + 'a {
        Disasm { pc, bits, op: self }
    }
}

/// Be cautious if you want to rely on the printed the information from this trait implementation.
/// For compressed jump and branches, the immediate will be incorrect. Use `RiscvInst::pretty_print` instead.
impl fmt::Display for RiscvInst {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.print(fmt, None)
    }
}

struct Disasm<'a> {
    pc: u64,
    bits: u32,
    op: &'a RiscvInst,
}

impl<'a> fmt::Display for Disasm<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if (self.pc & 0xFFFFFFFF) == self.pc {
            write!(fmt, "{:8x}:       ", self.pc)?;
        } else {
            write!(fmt, "{:16x}:       ", self.pc)?;
        }

        if self.bits & 3 == 3 {
            write!(fmt, "{:08x}", self.bits)?;
        } else {
            write!(fmt, "{:04x}    ", self.bits & 0xFFFF)?;
        }

        write!(fmt, "        ")?;
        self.op.print(fmt, Some(self.pc))
    }
}
