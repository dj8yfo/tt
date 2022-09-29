use std::{rc::Rc, cell::RefCell};

const BYTE_VALS: usize = 256;

pub struct Node<V> {
    terminal: Option<Rc<V>>,

    branches: [Option<Rc<RefCell<Node<V>>>>; BYTE_VALS ]
}

