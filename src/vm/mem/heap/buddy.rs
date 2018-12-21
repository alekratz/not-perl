use std::{
    mem,
    alloc::Layout,
};
use crate::{
    util::log2,
    vm::mem::{ConstAddr, Alloc, HeapRef},
};

/// Minimum block size for this allocator.
const MIN_BLOCK_SIZE: usize = 64;

/// A "buddy block" that determines the order of the current memory block.
///
/// This is used for bookkeeping for memory requests.
#[repr(C)]
struct BuddyBlock {
    order: u8,
    used: bool,
    top: bool,
    _padding: [u8; mem::size_of::<usize>() - 3],
}

impl BuddyBlock {
    /// Gets whether this block is the bottom half of its buddy.
    fn is_bottom(&self) -> bool {
        !self.top
    }

    /// Gets this block from an address.
    #[inline]
    unsafe fn from_address(addr: usize) -> &'static mut Self {
        if addr % mem::size_of::<Self>() != 0 {
            panic!("Buddy address was not a multiple of block size: {:#x}", addr);
        }
        &mut *(addr as *mut Self)
    }

    /// Gets this block's buddy.
    #[inline]
    unsafe fn buddy(&self) -> &'static mut Self {
        let block_size: usize = 1 << (self.order as u32);
        let addr = self as *const _ as usize;
        let mut buddy_addr = addr ^ block_size;

        // TODO : clean this up, we don't need to branch
        if self.top && addr < buddy_addr {
            buddy_addr -= block_size * 2;
        } else if !self.top && buddy_addr < addr {
            buddy_addr += block_size * 2;
        }

        assert!(addr != buddy_addr);
        Self::from_address(buddy_addr)
    }

    /// Gets the block whose address is lower between it and its buddy.
    unsafe fn first_half(&self) -> &'static mut Self {
        let buddy = self.buddy();
        if buddy.address() > self.address() {
            &mut *(self as *const _ as *mut _)
        } else {
            buddy
        }
    }

    /// Splits this block, returning the new block made.
    unsafe fn split(&mut self) -> &'static mut Self {
        assert!(!self.used);
        self.order -= 1;
        let buddy = self.buddy();
        buddy.order = self.order;
        buddy.used = false;
        buddy.top = !self.top;
        buddy
    }

    /// Gets the "cousin" to this block - that is, the block adjacent to this one, one order of
    /// magnitude up.
    unsafe fn next_adjacent(&mut self) -> &'static mut Self {
        let offset = if self.is_bottom() {
            2 << self.order
        } else {
            1 << self.order
        };
        BuddyBlock::from_address(self.address() + offset)
    }
    
    #[inline]
    fn address(&self) -> usize {
        self as *const _ as usize
    }
}

const_assert!(buddy_block_size; mem::size_of::<BuddyBlock>() == mem::size_of::<usize>());

/// An allocator that splits blocks in half when more memory is needed.
pub struct BuddyAllocator {
    /// Whether this allocator is ready for allocations.
    ///
    /// This is necessary since some extra set-up is required at run-time, and the allocator is
    /// constructed at compile-time - limiting the usefulness of things we can do.
    ready: bool,

    /// Start of the heap in memory.
    heap_start: usize,

    /// End of the heap in memory.
    heap_end: usize,

    /// Max block order.
    ///
    /// This is the largest order of a memory block.
    max_block_size: usize,

    /// Min block size.
    ///
    /// This is usually going to be 64, defined by the MIN_BLOCK_SIZE constant.
    min_block_size: usize,

    /// The maximum order that a block may have.
    max_block_order: usize,

    /// The minimum order that a block may have.
    min_block_order: usize,
}

impl BuddyAllocator {
    /// Initializes this heap.
    unsafe fn init(&mut self) {
        assert!(!self.ready, "Attempted to initialize heap twice");
        let heap_size = self.heap_end - self.heap_start + 1;
        if heap_size.is_power_of_two() {
            self.max_block_size = heap_size;
        } else {
            //self.max_block_size = 1 << log2(heap_size);
            unimplemented!("Heap size must be a power of 2 for the time being");
        }

        self.max_block_order = log2(self.max_block_size);
        self.min_block_order = log2(self.min_block_size);

        // zero all blocks
        let mut addr = self.heap_start;
        while addr < self.heap_end {
            let ptr = addr as *mut usize;
            *ptr = 0;
            addr += mem::size_of::<usize>();
        }

        // set up the first block and its buddy
        let block = BuddyBlock::from_address(self.heap_start);
        block.order = self.max_block_order as u8;

        //let buddy_address = block.buddy().address();
        block.top = false;
        let buddy_address = block.buddy().address();
        assert!(self.heap_start <= buddy_address, "buddy address is below heap start");
        // TODO : set up block buddies if the heap size is not a power of 2
        self.ready = true;
    }

    /// Finds the next block of the given order, if any are available.
    ///
    /// This will split blocks as necessary.
    ///
    /// # Arguments
    unsafe fn next_block(&self, order: usize, block_address: usize) -> Option<&BuddyBlock> {
        let mut block = BuddyBlock::from_address(block_address);
        let order = order as u8;

        loop {
            // break if the block's address has gone past the heap, our search is over
            if block.address() >= self.heap_end {
                break None;
            }

            assert!((block.order as usize) <= self.max_block_order && (block.order as usize) >= self.min_block_order,
                    "Invalid block order at {:#x}: {}", block.address(), block.order);

            if block.used {
                // block is used
                if block.order == order {
                    if block.is_bottom() {
                        let buddy = block.buddy();
                        if buddy.used {
                            block = block.next_adjacent();
                        } else {
                            buddy.used = true;
                            break Some(buddy);
                        }
                    } else {
                        block = block.next_adjacent();
                    }
                } else if block.order < order {
                    block = block.next_adjacent();
                } else {
                    if block.is_bottom() {
                        let buddy = block.buddy();
                        if buddy.used {
                            block = block.next_adjacent();
                        } else {
                            block = buddy;
                        }
                    } else {
                        block = block.next_adjacent();
                    }
                }
            } else {
                // block is free
                if block.order == order {
                    block.used = true;
                    break Some(block);
                } else if block.order < order {
                    block = block.next_adjacent();
                } else {
                    block.split();
                }
            }
        }
    }
}

unsafe impl Alloc for BuddyAllocator {
    type Ref = HeapRef;

    unsafe fn alloc(&mut self, layout: Layout) -> Option<Self::Ref> {
        assert!(self.ready, "Attempted to use heap before it is initialized");
        // request size needs to include the size of bookkeeping
        let request_size = layout.size() + mem::size_of::<BuddyBlock>();
        let order = if request_size <= self.min_block_size {
            self.min_block_order
        } else {
            log2(request_size) + 1
        };

        if order > self.max_block_order {
            return None;
        }

        // find the next block of the desired order
        if let Some(block) = self.next_block(order, self.heap_start) {
            let block_addr = block as *const _ as usize;
            assert!(block_addr < self.heap_end);
            // offset by the bookkeeping size
            Some(HeapRef::new((block_addr + mem::size_of::<BuddyBlock>()) as *mut u8))
        } else {
            None
        }
    }

    unsafe fn free(&mut self, rf: Self::Ref) {
        if cfg!(debug) && !rf.mark {
            warn!("ref at {:#x} not marked for deletion, but freed anyway", rf.addr as usize);
        }
        let ptr = rf.addr;
        let mut block = &mut *((ptr as usize - mem::size_of::<BuddyBlock>()) as *mut BuddyBlock);
        block.used = false;
        let mut buddy = block.buddy();
        // merge if this block's buddy is not being used either
        while !buddy.used && (block.order as usize) < self.max_block_order && buddy.order == block.order {
            // find the first one in memory and increment its order, and unset the buddy's order
            block = block.first_half();
            buddy = block.buddy();
            block.order += 1;
            if buddy.address() <= self.heap_end {
                buddy.order = 0;
            }
        }
    }

    unsafe fn new(heap_start: ConstAddr, heap_size: usize) -> Self {
        let heap_start = heap_start as usize;
        let mut alloc = BuddyAllocator {
            ready: false,
            heap_start,
            heap_end: heap_start + heap_size - 1,
            max_block_size: 0,
            min_block_size: MIN_BLOCK_SIZE,
            max_block_order: 0,
            min_block_order: 0,
        };
        alloc.init();
        alloc
    }
}
