use crate::vm::{
    mem::{Alloc, HeapRef, HeapStorage},
};

pub struct State<A>
where A: Sized + Alloc<Ref=HeapRef>
{
    heap: HeapStorage<A>,
}
