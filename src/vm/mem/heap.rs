mod buddy;
pub use self::buddy::*;

use crate::{
    vm::{
        VmString,
        mem::{VmRef, Alloc, Addr, ArrayRef},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeapRef {
    pub addr: Addr,
    pub mark: bool,
}

impl HeapRef {
    pub fn new(addr: Addr) -> Self {
        HeapRef { addr, mark: false }
    }
}

unsafe impl VmRef for HeapRef {
    unsafe fn deref<T: Sized>(&self) -> &T {
        &*(self.addr as *const T)
    }

    unsafe fn deref_mut<T: Sized>(&mut self) -> &mut T {
        &mut *(self.addr as *mut T)
    }
}

/// VM heap storage.
pub struct HeapStorage<A>
    where A: Sized + Alloc<Ref=HeapRef>
{
    /// The allocator that manages our owned memory.
    alloc: A,

    /// Heap storage.
    heap: Vec<u8>,
}

impl<A> HeapStorage<A>
    where A: Sized + Alloc<Ref=HeapRef>
{
    pub fn new(heap_size: usize) -> Self {
        let heap = vec!(0u8; heap_size);
        let alloc = unsafe { A::new(heap.as_slice().as_ptr(), heap_size) };
        HeapStorage {
            alloc,
            heap,
        }
    }

    pub fn alloc_string(&mut self) -> Option<VmString> {
        unimplemented!()
    }

    pub fn alloc_array<T: Sized>(&mut self, len: usize) -> Option<ArrayRef<T>> {
        ArrayRef::with_len(&mut self.alloc, len)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_alloc_array_basic() {
        let heap_size = 4096 * 4096;
        let array_size = 5000;
        let mut heap: HeapStorage<BuddyAllocator> = HeapStorage::new(heap_size);
        let mut array = heap.alloc_array(array_size)
            .unwrap();

        assert_eq!(array[0], 0);
        array[10] = 5;
        assert_eq!(array[10], 5);
        array[array_size - 1] = 99;
        assert_eq!(array[array_size - 1], 99);
    }

    #[test]
    fn test_alloc_array_objects() {
        #[derive(PartialEq, Debug, Clone)]
        enum SomeValue {
            A(i64),
            B(f64),
            C(char),
            D,
        }
        let heap_size = 4096 * 4096;
        let array_size = 5000;
        let mut heap: HeapStorage<BuddyAllocator> = HeapStorage::new(heap_size);
        let mut array: ArrayRef<SomeValue> = heap.alloc_array(array_size)
            .unwrap();

        let vals = &[
            SomeValue::A(10),
            SomeValue::B(-0.15),
            SomeValue::C('c'),
            SomeValue::D,
        ];
        for (i, val) in vals.iter().enumerate() {
            array[i] = val.clone();
            array[array_size - i - 1] = val.clone();
        }
        for (i, val) in vals.iter().enumerate() {
            let other = &array[i];
            assert_eq!(&array[i], val);
            assert_eq!(&array[array_size - i - 1], val);
        }
    }
}
