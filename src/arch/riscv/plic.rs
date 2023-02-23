use super::{
    bus::*,
    cpu::{HART_COUNT, MAX_HART_COUNT},
    exception::Exception,
};

const SOURCE_COUNT: usize = 32;
const MAX_SOURCE_COUNT: u64 = 1024;

#[derive(Debug, Clone, Copy)]
pub struct PlicContext {
    pub priority_threshold: u32,
    pub claim_or_complete: u32,
    pub enable_bits: [u32; (SOURCE_COUNT - 1) / 32 + 1],
}

impl PlicContext {
    pub fn new() -> Self {
        PlicContext {
            priority_threshold: 0,
            claim_or_complete: 0,
            enable_bits: [0; (SOURCE_COUNT - 1) / 32 + 1],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Plic {
    pending: u64,
    senable: u64,
    spriority: u64,
    sclaim: u64,

    source_priority: [u32; (SOURCE_COUNT - 1) / 32 + 1],
    pending_bits: [u32; (SOURCE_COUNT - 1) / 32 + 1],
    context: [PlicContext; HART_COUNT],
}

const INT_PRIORITY_BASE: u64 = 0x0;
const INT_PRIORITY_STRIDE: u64 = 0x4;
const INT_PRIORITY_END: u64 = INT_PRIORITY_BASE + INT_PRIORITY_STRIDE * MAX_SOURCE_COUNT - 1;

const INT_PENDING_BASE: u64 = 0x1000;
const INT_PENDING_STRIDE: u64 = 0x4;
const INT_PENDING_COUNT: u64 = MAX_SOURCE_COUNT / 32;
const INT_PENDING_END: u64 = INT_PENDING_BASE + INT_PENDING_STRIDE * INT_PENDING_COUNT - 1;

const INT_ENABLE_BITS_BASE: u64 = 0x2000;
const INT_ENABLE_BITS_STRIDE: u64 = 0x80;
const INT_ENABLE_BITS_END: u64 = INT_ENABLE_BITS_BASE + INT_ENABLE_BITS_STRIDE * MAX_HART_COUNT - 1;

const INT_PRIORITY_THRESHOLD_BASE: u64 = 0x200000;
const INT_PRIORITY_THRESHOLD_STRIDE: u64 = 0x1000;
const INT_PRIORITY_THRESHOLD_END: u64 =
    INT_PRIORITY_THRESHOLD_BASE + INT_PRIORITY_THRESHOLD_STRIDE * MAX_HART_COUNT - 1;

#[derive(Debug, PartialEq)]
enum PlicOp {
    InterruptPriorityOfSource(u32),
    InterruptPendingBit(u32),
    EnableBitsForSourcesAndOnContext(u32, u32),
    PriorityThresholdForContext(u32),
    ClaimOrCompleteForContext(u32),
}

fn parse_addr(addr: u64) -> Result<PlicOp, ()> {
    let relative = addr - PLIC_BASE;
    match relative {
        INT_PRIORITY_BASE..=INT_PRIORITY_END => {
            let source = ((relative - INT_PRIORITY_BASE) / INT_PRIORITY_STRIDE) as u32;
            Ok(PlicOp::InterruptPriorityOfSource(source))
        }
        INT_PENDING_BASE..=INT_PENDING_END => {
            let source = ((relative - INT_PENDING_BASE) / INT_PENDING_STRIDE) as u32;
            Ok(PlicOp::InterruptPendingBit(source))
        }
        INT_ENABLE_BITS_BASE..=INT_ENABLE_BITS_END => {
            let source = ((relative - INT_ENABLE_BITS_BASE) / INT_ENABLE_BITS_STRIDE) as u32;
            let context = ((relative - INT_ENABLE_BITS_BASE) % INT_ENABLE_BITS_STRIDE) as u32;
            Ok(PlicOp::EnableBitsForSourcesAndOnContext(source, context))
        }
        INT_PRIORITY_THRESHOLD_BASE..=INT_PRIORITY_THRESHOLD_END => {
            let context =
                ((relative - INT_PRIORITY_THRESHOLD_BASE) / INT_PRIORITY_THRESHOLD_STRIDE) as u32;
            let threshold =
                ((relative - INT_PRIORITY_THRESHOLD_BASE) % INT_PRIORITY_THRESHOLD_STRIDE) as u32;
            match threshold {
                0 => Ok(PlicOp::PriorityThresholdForContext(context)),
                4 => Ok(PlicOp::ClaimOrCompleteForContext(context)),
                _ => Err(()),
            }
        }
        _ => Err(()),
    }
}

impl Plic {
    pub fn new() -> Self {
        Self {
            pending: 0,
            senable: 0,
            spriority: 0,
            sclaim: 0,

            pending_bits: [0; (SOURCE_COUNT - 1) / 32 + 1],
            source_priority: [0; (SOURCE_COUNT - 1) / 32 + 1],
            context: [PlicContext::new(); HART_COUNT],
        }
    }

    pub fn load(&self, addr: u64, size: u64) -> Result<u64, Exception> {
        if size != 4 {
            return Err(Exception::LoadAccessFault(addr));
        }
        match parse_addr(addr) {
            Ok(PlicOp::InterruptPriorityOfSource(source)) => {
                Ok(self.source_priority[source as usize] as u64)
            }
            Ok(PlicOp::InterruptPendingBit(source)) => {
                Ok(self.pending_bits[source as usize] as u64)
            }
            Ok(PlicOp::EnableBitsForSourcesAndOnContext(source, context)) => {
                Ok(self.context[context as usize].enable_bits[source as usize] as u64)
            }
            Ok(PlicOp::PriorityThresholdForContext(context)) => {
                Ok(self.context[context as usize].priority_threshold as u64)
            }
            Ok(PlicOp::ClaimOrCompleteForContext(context)) => {
                Ok(self.context[context as usize].claim_or_complete as u64)
            }
            Err(_) => Ok(0),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), Exception> {
        if size != 4 {
            return Err(Exception::StoreAMOAccessFault(addr));
        }
        match parse_addr(addr) {
            Ok(PlicOp::InterruptPriorityOfSource(source)) => Ok({
                self.source_priority[source as usize] = value as u32;
            }),
            Ok(PlicOp::InterruptPendingBit(source)) => Ok({
                self.pending_bits[source as usize] = value as u32;
            }),
            Ok(PlicOp::EnableBitsForSourcesAndOnContext(source, context)) => Ok({
                self.context[context as usize].enable_bits[source as usize] = value as u32;
            }),
            Ok(PlicOp::PriorityThresholdForContext(context)) => Ok({
                self.context[context as usize].priority_threshold = value as u32;
            }),
            Ok(PlicOp::ClaimOrCompleteForContext(context)) => Ok({
                self.context[context as usize].claim_or_complete = value as u32;
            }),
            Err(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::arch::riscv::bus::PLIC_BASE;

    #[test]
    fn test_parse_addr() {
        assert_eq!(
            super::parse_addr(PLIC_BASE + 0x000FFC),
            Ok(super::PlicOp::InterruptPriorityOfSource(0x3FF))
        );

        assert_eq!(
            super::parse_addr(PLIC_BASE + 0x002084),
            Ok(super::PlicOp::EnableBitsForSourcesAndOnContext(0x1, 0x4))
        );

        assert_eq!(
            super::parse_addr(PLIC_BASE + 0x201000),
            Ok(super::PlicOp::PriorityThresholdForContext(0x1))
        );

        assert_eq!(
            super::parse_addr(PLIC_BASE + 0x3FFF004),
            Ok(super::PlicOp::ClaimOrCompleteForContext(0x3DFF))
        );
    }
}
