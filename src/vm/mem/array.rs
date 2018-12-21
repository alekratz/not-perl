use std::{
    alloc::Layout,
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut},
};
use crate::vm::mem::{Addr, Alloc, VmNew, VmSized, HeapRef, VmRef};

const ALIGN: usize = mem::size_of::<usize>();
const GROWTH_FACTOR: f64 = 1.5;

/// A resizable array implementation.
#[derive(Debug, Clone)]
pub struct ArrayList<T: Sized> {
    array_ref: ArrayRef<T>,
    len: usize,
}

impl<T: Sized> ArrayList<T> {
    /// Pushes a value to the end of this array.
    pub fn push(&mut self, value: T) {
        if self.len() == self.capacity() {
            // grow the array
            let new_size = (self.len() as f64 * GROWTH_FACTOR) as usize;
            self.resize(new_size);
        }
        assert!(self.len() < self.capacity(), "array is at capacity length after a resize");
        let index = self.len();
        self.array_ref[index] = value;
        self.len += 1;
    }

    /// Gets a value from the given index of this array.
    pub fn get(&self, index: usize) -> Option<&T> {
        if self.len() <= index {
            None
        } else {
            Some(&self.array_ref[index])
        }
    }

    /// The capacity of the array backing this array list.
    pub fn capacity(&self) -> usize {
        self.array_ref.len()
    }

    /// The length of this array list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Resizes this array, allocating a new backing array if necessary.
    pub fn resize(&mut self, new_size: usize) {
        if new_size == self.len() {
            return;
        } else if new_size <= self.capacity() {
            self.len = new_size;
        } else {
            // re-allocate
        }
    }

    /// Resizes this array, filling any new cells with the given value.
    pub fn resize_with(&mut self, new_size: usize, value: T)
        where T: Clone
    {
        if new_size < self.len() {
            // simple resize with no copying
            self.resize(new_size);
        } else {
            let start = self.len();
            self.resize(new_size);
            for i in start .. new_size {
                self.array_ref[i] = value.clone();
            }
        }
    }

    /// Creates a new array list with the given capacity.
    pub fn with_capacity<A: Alloc<Ref=HeapRef>>(alloc: &mut A, len: usize) -> Option<Self> {
        Some(ArrayList {
            array_ref: ArrayRef::with_len(alloc, len)?,
            len: 0,
        })
    }
}

impl<T: Sized> VmNew for ArrayList<T> {
    fn new<A: Alloc<Ref=HeapRef>>(alloc: &mut A) -> Option<Self> {
        Self::with_capacity(alloc, 0)
    }
}

/// A reference to a fixed-size contiguous block of memory in the heap.
#[derive(Debug, Clone)]
pub struct ArrayRef<T: Sized> {
    /// Reference to the memory where this array lives.
    heap_ref: HeapRef,
    _ty: PhantomData<T>,
}

impl<T: Sized> ArrayRef<T> {
    pub fn len(&self) -> usize {
        unsafe { self.as_repr().len }
    }

    unsafe fn as_repr(&self) -> &ArrayRepr<T> {
        self.heap_ref.deref()
    }

    pub fn with_len<A: Alloc<Ref=HeapRef>>(alloc: &mut A, len: usize) -> Option<Self> {
        let size = len * mem::size_of::<T>();
        let layout = Layout::from_size_align(size, ALIGN)
            .ok()?;
        let mut heap_ref;
        unsafe {
            heap_ref = alloc.alloc(layout)?;
            let array: &mut ArrayRepr<T> = heap_ref.deref_mut();
            array.len = len;
        }
        Some(ArrayRef { heap_ref, _ty: PhantomData::<T>, })
    }

    pub fn get(&self, index: usize) -> &T {
        unsafe {
            &*(self.as_repr().at(index) as *const T)
        }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        unsafe {
            &mut *(self.as_repr().at(index) as *mut T)
        }
    }

    pub fn set(&mut self, index: usize, value: T) {
        let rf = self.get_mut(index);
        *rf = value;
    }
}

impl<T: Sized> VmNew for ArrayRef<T> {
    /// Creates a new, empty array.
    fn new<A: Alloc<Ref=HeapRef>>(alloc: &mut A) -> Option<Self> {
        ArrayRef::with_len(alloc, 0)
    }
}

impl<T: Sized> VmSized for ArrayRef<T> {
    fn size_of(&self) -> usize {
        unsafe { self.as_repr() }.size_of()
    }
}


impl<T: Sized> Index<usize> for ArrayRef<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<T: Sized> IndexMut<usize> for ArrayRef<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}

#[derive(Debug, Clone)]
struct ArrayRepr<T: Sized> {
    /// Reference to the memory where this array lives.
    len: usize,
    _ty: PhantomData<T>,
}

impl<T: Sized> ArrayRepr<T> {
    unsafe fn at(&self, index: usize) -> *mut T {
        if self.len <= index {
            panic!("index out of range: {}", index);
        }

        let self_offset = mem::size_of::<Self>() as isize;
        let base = (self as *const _ as Addr).offset(self_offset) as *mut T;
        base.offset(index as isize)
    }
}

impl<T: Sized> VmSized for ArrayRepr<T> {
    fn size_of(&self) -> usize {
        (self.len * mem::size_of::<T>()) + mem::size_of_val(&self.len)
    }
}
