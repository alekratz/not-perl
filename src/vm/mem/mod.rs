mod heap;
mod stack;
mod string32;
mod array;

pub use self::heap::*;
pub use self::stack::*;
pub use self::string32::*;
pub use self::array::*;

use std::alloc::Layout;
use crate::vm::Value;

pub type Addr = *mut u8;
pub type ConstAddr = *const u8;

/// An outline for an allocator of some kind of storage.
pub unsafe trait Alloc {
    type Ref: VmRef;

    /// Allocates a value.
    unsafe fn alloc(&mut self, layout: Layout) -> Option<Self::Ref>;

    /// Frees a reference.
    unsafe fn free(&mut self, rf: Self::Ref);

    unsafe fn new(start_addr: ConstAddr, size: usize) -> Self;
}

/// A type for values that can be allocated by a VM allocator.
pub trait VmNew: Sized {
    fn new<A: Alloc<Ref=HeapRef>>(alloc: &mut A) -> Option<Self>;
}

pub trait VmSized {
    fn size_of(&self) -> usize;
}

pub unsafe trait VmRef {
    unsafe fn deref<T: Sized>(&self) -> &T;
    unsafe fn deref_mut<T: Sized>(&mut self) -> &mut T;
}
