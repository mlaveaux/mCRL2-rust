use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use rustc_hash::FxHashMap;


struct Symbol {
    set: Rc<RefCell<SymbolSetShared>>,
    index: usize,
}

/// The underlying data for a function symbol
#[derive(Eq, Hash, PartialEq)]
struct SymbolData {
    name: String,
    arity: usize,
}

/// Stores the function symbols
struct SymbolSetShared {
    table: FxHashMap<SymbolData, usize>,
    unique_table: Vec<SymbolData>,

}

struct SymbolSet {
    shared: Rc<RefCell<SymbolSetShared>>,
}

impl SymbolSet {
    
    /// Create a function symbol with the given name and arity
    pub fn create(&mut self, name: &str, arity: usize) -> Symbol {

        let mut shared = (*self.shared).borrow_mut();

        let index = shared.table.entry(SymbolData { name: name.to_string(), arity }).or_insert_with(|| {
            let index = shared.unique_table.len();
            shared.unique_table.push(SymbolData { name: name.to_string(), arity });
            index
        });

        Symbol {
            set: self.shared.clone(),
            index: *index,
        }
    }
}
