use std::{collections::HashMap, usize};

enum Command {
    Add(usize),
    Delete(usize),
}

pub struct Subs {
    registry: HashMap<String, Vec<usize>>,
}

impl Subs {
    pub fn new() -> Subs {
        let registry = HashMap::<String, Vec<usize>>::new();
        Subs { registry }
    }
}
