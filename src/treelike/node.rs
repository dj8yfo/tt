use std::rc::Rc;
use std::slice::Iter;

const BYTE_VALS: usize = 256;

#[derive(Clone)]
pub struct Node<V> {
    terminal: Option<Rc<V>>,

    branches: [Option<Rc<Node<V>>>; BYTE_VALS ]
}

impl<V> Node<V> {
    pub fn new() -> Self {
        let array = [(); BYTE_VALS].map(|_| None);
        Node {
            terminal: None,
            branches: array,
        }
    }
}

impl<V: Clone> Node<V> {


    pub fn insert(&mut self, mut iter: Iter<'_, u8>, v: V) -> Option<V> {
        match iter.next(){
            None => {
                let result = self.terminal.take().map(|rc|{
                    (*rc).clone()
                });
                self.terminal = Some(Rc::new(v));
                result
            },
            Some(b) => {
                let prev = self.branches[*b as usize].take();

                match prev {
                    None => {
                        let mut node = Node::new();
                        let result = node.insert(iter, v);
                        self.branches[*b as usize] = Some(Rc::new(node)); 
                        result
                    }, 
                    Some(rc) => {
                        let mut slight_copy = (*rc).clone();
                        let result = slight_copy.insert(iter, v);
                        self.branches[*b as usize] = Some(Rc::new(slight_copy)); 
                        result
                    }, 
                }
            }
        }

    }
}

