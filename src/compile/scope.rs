use crate::{
    common::scope::*,
    compile::BlockSymbolAlloc,
    vm::Label,
};

pub type LabelScope = AllocScope<Label, BlockSymbolAlloc>;
