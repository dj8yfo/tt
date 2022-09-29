use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::slice::Iter;

const BYTE_VALS: usize = 256;

#[derive(Clone)]
pub struct Node<V> {
    terminal: Option<Rc<V>>,

    // the refcell part is only needed to manually implement drop
    // to avoid stack overflows
    //
    // safety: borrow_mut should only ocur under node and VMapTree mutable methods
    branches: [Option<Rc<RefCell<Node<V>>>>; BYTE_VALS],
}

impl<V> Node<V> {
    pub fn new() -> Self {
        let array = [(); BYTE_VALS].map(|_| None);
        Node {
            terminal: None,
            branches: array,
        }
    }

    pub fn drop_references(&mut self, res: &mut VecDeque<Rc<RefCell<Node<V>>>>) {
        for i in 0..self.branches.len() {
            let opt = self.branches[i].take();
            match opt {
                None => {}
                Some(rc) => {
                    if Rc::strong_count(&rc) > 1 {
                        continue;
                    };
                    res.push_back(rc);
                }
            }
        }
    }
}

pub(super) fn vacuum_clean<V>(node: &mut Node<V>) {
    let mut res = VecDeque::new();
    node.drop_references(&mut res);
    let mut count = 0;
    while let Some(front) = res.pop_front() {
        front.borrow_mut().drop_references(&mut res);
        count += 1;
    }
    println!("{} dropped", count);
}

impl<V: Clone> Node<V> {
    pub fn insert(&mut self, mut iter: Iter<'_, u8>, v: V) -> Option<V> {
        match iter.next() {
            None => {
                let result = self.terminal.take().map(|rc| (*rc).clone());
                self.terminal = Some(Rc::new(v));
                result
            }
            Some(b) => {
                let prev = self.branches[*b as usize].take();

                match prev {
                    None => {
                        let mut node = Node::new();
                        let result = node.insert(iter, v);
                        self.branches[*b as usize] = Some(Rc::new(RefCell::new(node)));
                        result
                    }
                    Some(rc) => {
                        let slight_copy = (*rc).clone();
                        let result = slight_copy.borrow_mut().insert(iter, v);
                        self.branches[*b as usize] = Some(Rc::new(slight_copy));
                        result
                    }
                }
            }
        }
    }

    pub fn get(&self, mut iter: Iter<'_, u8>) -> Option<&V> {
        match iter.next() {
            None => self.terminal.as_deref(),
            Some(b) => match &self.branches[*b as usize] {
                None => None,
                Some(rc) => unsafe { rc.try_borrow_unguarded().unwrap().get(iter) },
            },
        }
    }
}
