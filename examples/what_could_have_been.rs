struct Ui {
    canary: usize,
    nodes: Vec<Node>,
    parent_stack: Vec<usize>,
    _not_send: std::marker::PhantomData<*mut ()>,
}

struct Node {
    value: i32,
    first_child: Option<usize>,
    next_sibling: Option<usize>,
}

struct UiParent {
    ui: *mut Ui,
    idx: usize,
    expected_canary: usize,
}

impl Ui {
    fn new() -> Self {
        let root = Node { value: 0, first_child: None, next_sibling: None };
        Ui {
            canary: random_canary(),
            nodes: vec![root],
            parent_stack: vec![0],
            _not_send: std::marker::PhantomData,
        }
    }

    fn add(&mut self, value: i32) -> UiParent {
        let idx = self.nodes.len();
        self.nodes.push(Node { value, first_child: None, next_sibling: None });
        let parent_idx = *self.parent_stack.last().unwrap();
        if let Some(last) = self.nodes[parent_idx].first_child {
            self.nodes[last].next_sibling = Some(idx);
        } else {
            self.nodes[parent_idx].first_child = Some(idx);
        }
        UiParent { ui: self as *mut Ui, idx, expected_canary: self.canary }
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

fn random_canary() -> usize {
    use std::hash::{Hash, Hasher};
    let mut h = std::hash::DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut h);
    let local = 0usize;
    (&local as *const usize as usize).hash(&mut h);
    h.finish() as usize
}

impl Drop for Ui {
    fn drop(&mut self) {
        self.canary = !self.canary;
    }
}

impl UiParent {
    fn check(&self) {
        // Safety: not safe, this is just an example.
        let canary = unsafe { *(self.ui as *const usize) };
        assert_eq!(canary, self.expected_canary, "Ui was dropped before concluding a nest() block");
    }

    fn nest(self, f: impl FnOnce()) {
        // Safety: not safe, this is just an example.
        unsafe {
            self.check();
            (*self.ui).parent_stack.push(self.idx);
            f();
            self.check();
            (*self.ui).parent_stack.pop();
        }
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

    // In practice it gets caught by the canary and panics cleanly. But we're already deep into theoretical UB either way and we will be sent to the borrow gulags for even thinking about it.
    let mut ui2 = Ui::new();
    ui2.add(8).nest(|| {
        drop(ui2);
    }); 
}