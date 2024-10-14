use core::{marker::PhantomData, mem::size_of};

use crate::{
    sdt::{SdtHeader, Signature},
    AcpiTable,
};

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Mpam {
    header: SdtHeader,
    // An array of MPAM node structures that describes MSCs in the system.
}

/// ### Safety: This is safe to implement.
unsafe impl AcpiTable for Mpam {
    const SIGNATURE: Signature = Signature::MPAM;

    fn header(&self) -> &SdtHeader {
        &self.header
    }
}

use core::fmt;
impl fmt::Display for Mpam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MAPM: {:#x?}", self.header)?;
        for node in self.nodes() {
            write!(f, "\n{:#x?}", node)?;
        }
        Ok(())
    }
}

impl Mpam {
    /// Returns an iterator over the MPAM node structures.
    pub fn nodes(&self) -> MscNodeIter {
        let pointer = unsafe { (self as *const Mpam).add(1) as *const u8 };
        let remaining_length = self.header.length as u32 - size_of::<Mpam>() as u32;

        MscNodeIter { pointer, remaining_length, _phantom: PhantomData }
    }

    pub fn memory_bases(&self) -> impl Iterator<Item = u64> + '_ {
        self.nodes().filter_map(|node| match node.if_type {
            2 => Some(node.base_address),
            _ => None,
        })
    }

    pub fn cache_bases(&self) -> impl Iterator<Item = u64> + '_ {
        self.nodes().filter_map(|node| match node.if_type {
            1 => Some(node.base_address),
            _ => None,
        })
    }
}

pub struct MscNodeIter<'a> {
    pointer: *const u8,
    remaining_length: u32,
    _phantom: PhantomData<&'a ()>,
}

impl Iterator for MscNodeIter<'_> {
    type Item = MscNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_length <= 0 {
            return None;
        }

        let node = unsafe { &*(self.pointer as *const MscNode) };
        let node_length = node.length as u32;

        self.pointer = unsafe { self.pointer.add(node_length as usize) };
        self.remaining_length -= node_length;

        Some(*node)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MscNode {
    // MPAM node structure
    pub if_type: u8,
    pub length: u16,
    pub reserved: u8,
    pub base_address: u64,

    pub overflow_interrupt: u32,
    pub overflow_interrupt_flags: u32,

    pub error_interrupt: u32,
    pub error_interrupt_flags: u32,

    pub max_nrdy_usec: u32,
    pub offset: u32,
    // Resource node list
    // after the resource node list, there is resource specific data, if the length after the resource node list is not 0
}
