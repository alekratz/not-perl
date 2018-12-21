mod bc;
mod value;
pub mod mem;
mod state;
mod pool;
mod symbol;

pub use self::bc::*;
pub use self::value::*;
pub use self::state::*;
pub use self::pool::*;
pub use self::symbol::*;

/// A string that the VM uses.
pub type VmString = mem::String32;
