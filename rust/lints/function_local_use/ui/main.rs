use std::collections::BTreeMap;

fn local_import() -> usize {
    use std::collections::HashMap;
    HashMap::<u8, u8>::new().len()
}

fn nested_block() -> usize {
    let n = {
        use std::collections::BTreeSet;
        BTreeSet::<u8>::new().len()
    };
    n + BTreeMap::<u8, u8>::new().len()
}

fn glob_is_allowed() -> Ordering {
    use std::cmp::Ordering::*;
    Less
}

use std::cmp::Ordering;

mod inline {
    use std::collections::VecDeque;

    pub fn module_level_is_fine() -> usize {
        VecDeque::<u8>::new().len()
    }
}

fn main() {
    let _ = (local_import(), nested_block(), glob_is_allowed(), inline::module_level_is_fine());
}
