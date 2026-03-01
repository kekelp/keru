use std::{cell::RefCell, rc::Rc};

struct Ui {
    nodes: Vec<Node>,
    parent_stack: Rc<RefCell<Vec<usize>>>,
}

struct Node {
    value: i32,
    first_child: Option<usize>,
    next_sibling: Option<usize>,
}

struct UiParent {
    parent_stack: Rc<RefCell<Vec<usize>>>,
    idx: usize,
}

impl Ui {
    fn new() -> Self {
        let root = Node { value: 0, first_child: None, next_sibling: None };
        Ui {
            nodes: vec![root],
            parent_stack: Rc::new(RefCell::new(vec![0])),
        }
    }

    fn add(&mut self, value: i32) -> UiParent {
        let idx = self.nodes.len();
        self.nodes.push(Node { value, first_child: None, next_sibling: None });
        let parent_idx = *self.parent_stack.borrow().last().unwrap();
        if let Some(last) = self.nodes[parent_idx].first_child {
            self.nodes[last].next_sibling = Some(idx);
        } else {
            self.nodes[parent_idx].first_child = Some(idx);
        }
        UiParent { idx, parent_stack: Rc::clone(&self.parent_stack) }
    }

    fn print_tree(&self) {
        let mut idx = Some(0);
        while let Some(i) = idx {
            self.print_node(i, 0);
            idx = self.nodes[i].next_sibling;
        }
    }

    fn print_node(&self, idx: usize, depth: usize) {
        let node = &self.nodes[idx];
        println!("{}{}", "  ".repeat(depth), node.value);
        let mut child = node.first_child;
        while let Some(c) = child {
            self.print_node(c, depth + 1);
            child = self.nodes[c].next_sibling;
        }
    }
}

impl UiParent {
    fn nest(self, f: impl FnOnce()) {
        self.parent_stack.borrow_mut().push(self.idx);
        f();
        self.parent_stack.borrow_mut().pop();
    }
}

fn main() {
    let mut ui = Ui::new();
    ui.add(1).nest(|| {
        ui.add(2).nest(|| {
            ui.add(4).nest(|| {
                ui.add(7);
            });
            ui.add(5);
        });
        ui.add(3).nest(|| {
            ui.add(6);
        });
    });
    ui.print_tree();

    let mut ui2 = Ui::new();
    ui2.add(8).nest(|| {
        drop(ui2);
    });
}