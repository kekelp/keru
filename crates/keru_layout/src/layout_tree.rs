use crate::*;
use slotmap::{SlotMap, new_key_type};

new_key_type! {
    /// The id of a node in a `LayoutTree`.
    pub struct NodeId;
}

#[derive(Debug, Clone)]
pub(crate) struct Node<T> {
    pub(crate) parent: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) last_child: Option<NodeId>,
    pub(crate) prev_sibling: Option<NodeId>,
    pub(crate) next_sibling: Option<NodeId>,
    pub(crate) params: LayoutParams,
    pub(crate) layout: NodeLayoutData,
    pub(crate) rect: XyRect,
    pub(crate) context: Option<T>,
}

/// A tree of layout nodes. It records the changes to the nodes, and `compute_layout` uses them to do an incremental relayout when it can.
///
/// `T` is the type of the context of each node, for example its text. The measure function of `compute_layout_with_measure` receives it.
///
/// The tree always has a root node, which takes the size of the window. All other nodes are descendants of the root.
///
/// The functions panic if they receive the id of a removed node.
#[derive(Debug, Clone)]
pub struct LayoutTree<T = ()> {
    pub(crate) nodes: SlotMap<NodeId, Node<T>>,
    root: NodeId,
    pub(crate) layout_data: LayoutData<NodeId>,
    warnings: Vec<LayoutWarning<NodeId>>,
    window_size: Xy<f32>,
}

impl<T> Default for LayoutTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> LayoutTree<T> {
    /// Makes a tree that contains only the root.
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    /// Makes a tree that contains only the root, with space for `capacity` nodes.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut nodes = SlotMap::with_capacity_and_key(capacity);
        let root = nodes.insert(Node::new(LayoutParams::DEFAULT));
        LayoutTree { nodes, root, layout_data: LayoutData::new(), warnings: Vec::new(), window_size: Xy::new(0.0, 0.0) }
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Makes a node, and adds it as the last child of `parent`.
    pub fn add_node(&mut self, parent: NodeId, params: LayoutParams) -> NodeId {
        self.add_node_inner(parent, params, None)
    }

    /// Makes a node with a context, and adds it as the last child of `parent`.
    pub fn add_node_with_context(&mut self, parent: NodeId, params: LayoutParams, context: T) -> NodeId {
        self.add_node_inner(parent, params, Some(context))
    }

    /// Moves a node and its subtree, placing it as a child of `parent` and right  before `before_sibling`, or to the end if `before_sibling` is `None`.
    pub fn move_node(&mut self, i: NodeId, parent: NodeId, before_sibling: Option<NodeId>) {
        if let Some(sibling) = before_sibling {
            assert!(sibling != i, "Can't move node {i:?} before itself.");
            assert!(self.nodes[sibling].parent == Some(parent), "Node {sibling:?} is not a child of {parent:?}.");
        }
        if self.nodes[i].parent == Some(parent) && self.nodes[i].next_sibling == before_sibling {
            return;
        }
        let mut ancestor = Some(parent);
        while let Some(a) = ancestor {
            assert!(a != i, "Can't move node {i:?} into {parent:?}: it is the same node or one of its ancestors.");
            ancestor = self.nodes[a].parent;
        }
        self.unlink(i);
        let prev = match before_sibling {
            Some(sibling) => self.nodes[sibling].prev_sibling,
            None => self.nodes[parent].last_child,
        };
        self.link(i, parent, prev, before_sibling);
    }

    /// Changes the params of a node. If the parent is disabled, the node stays disabled. A change of `disabled` also applies to all the descendants. Equal params cause no relayout.
    pub fn update_node_params(&mut self, i: NodeId, mut params: LayoutParams) {
        if let Some(parent) = self.nodes[i].parent {
            params.disabled |= self.nodes[parent].params.disabled;
        }
        if self.nodes[i].params == params {
            return;
        }
        let old = std::mem::replace(&mut self.nodes[i].params, params);
        if old.disabled != params.disabled {
            self.set_subtree_disabled(i, params.disabled);
        }
        self.layout_data.record_node_change(i, &old, &params);
    }

    /// Removes a node and all its descendants.
    pub fn remove(&mut self, i: NodeId) {
        assert!(i != self.root, "Can't remove the root.");
        self.unlink(i);
        let mut stack = vec![i];
        while let Some(i) = stack.pop() {
            stack.extend(self.children(i));
            self.nodes.remove(i);
        }
        self.layout_data.request_full_solve(IncrementalRelayoutOutcome::FallbackStructuralChange);
    }

    pub fn params(&self, i: NodeId) -> &LayoutParams {
        &self.nodes[i].params
    }

    /// Updates the context of a node. The next `compute_layout` measures the text of the node again.
    pub fn update_node_context(&mut self, i: NodeId, context: Option<T>) {
        self.nodes[i].context = context;
        self.layout_data.contents.push(i);
    }

    /// Makes the next `compute_layout` measure the text of a node again. Use it when the text changes and the context stays the same, for example when the context is an id and the text it points to changes.
    pub fn mark_text_changed(&mut self, i: NodeId) {
        self.layout_data.contents.push(i);
    }

    /// Makes the next `compute_layout` a full solve. Use it when the measure function changes, for example after a font change.
    pub fn invalidate(&mut self) {
        self.layout_data.request_full_solve(IncrementalRelayoutOutcome::FallbackInvalidated);
    }

    /// The parent of a node. Only the root has no parent.
    pub fn parent(&self, i: NodeId) -> Option<NodeId> {
        self.nodes[i].parent
    }

    pub fn children(&self, i: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        std::iter::successors(self.nodes[i].first_child, |&child| self.nodes[child].next_sibling)
    }

    /// The rect of a node from the last `compute_layout`, in logical pixels.
    pub fn layout(&self, i: NodeId) -> XyRect {
        self.nodes[i].rect
    }

    /// The warnings from the last `compute_layout`. An incremental relayout reports only the warnings of the slots that it computes again.
    pub fn warnings(&self) -> &[LayoutWarning<NodeId>] {
        &self.warnings
    }

    pub fn window_size(&self) -> Xy<f32> {
        self.window_size
    }

    /// Sets the size of the window in logical pixels. The root takes this size. It starts as zero.
    pub fn set_window_size(&mut self, window_size: Xy<f32>) {
        self.window_size = window_size;
    }

    /// Solves the layout. The text of all nodes measures as zero.
    pub fn solve_layout(&mut self) -> IncrementalRelayoutOutcome {
        self.solve_layout_with_measure(|_, _, _| Xy::new(0.0, 0.0))
    }

    /// Solves the layout. `measure` gives the size of the text of a node, with its width optionally fixed to a wrap width.
    pub fn solve_layout_with_measure(&mut self, measure: impl FnMut(NodeId, Option<&mut T>, Option<f32>) -> Xy<f32>) -> IncrementalRelayoutOutcome {
        self.warnings.clear();
        Adapter { tree: self, measure }.solve_layout_incremental()
    }

    fn add_node_inner(&mut self, parent: NodeId, params: LayoutParams, context: Option<T>) -> NodeId {
        let mut node = Node::new(params);
        node.context = context;
        let i = self.nodes.insert(node);
        let last = self.nodes[parent].last_child;
        self.link(i, parent, last, None);
        i
    }

    fn link(&mut self, i: NodeId, parent: NodeId, prev: Option<NodeId>, next: Option<NodeId>) {
        let node = &mut self.nodes[i];
        node.parent = Some(parent);
        node.prev_sibling = prev;
        node.next_sibling = next;
        match prev {
            Some(prev) => self.nodes[prev].next_sibling = Some(i),
            None => self.nodes[parent].first_child = Some(i),
        }
        match next {
            Some(next) => self.nodes[next].prev_sibling = Some(i),
            None => self.nodes[parent].last_child = Some(i),
        }
        if self.nodes[parent].params.disabled {
            self.set_subtree_disabled(i, true);
        }
        self.layout_data.request_full_solve(IncrementalRelayoutOutcome::FallbackStructuralChange);
    }

    /// Takes a node out of the children of its parent.
    fn unlink(&mut self, i: NodeId) {
        let parent = self.nodes[i].parent.take().expect("The root has no parent.");
        let prev = self.nodes[i].prev_sibling.take();
        let next = self.nodes[i].next_sibling.take();
        match prev {
            Some(prev) => self.nodes[prev].next_sibling = next,
            None => self.nodes[parent].first_child = next,
        }
        match next {
            Some(next) => self.nodes[next].prev_sibling = prev,
            None => self.nodes[parent].last_child = prev,
        }
    }

    fn set_subtree_disabled(&mut self, i: NodeId, disabled: bool) {
        let mut stack = vec![i];
        while let Some(i) = stack.pop() {
            stack.extend(self.children(i));
            self.nodes[i].params.disabled = disabled;
        }
    }
}

impl<T> Node<T> {
    fn new(params: LayoutParams) -> Self {
        Node {
            parent: None,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
            params,
            layout: NodeLayoutData::EMPTY,
            rect: Xy::new([0.0, 0.0], [0.0, 0.0]),
            context: None,
        }
    }
}

/// Gives a `LayoutTree` and a measure function to the solver.
struct Adapter<'a, T, M> {
    tree: &'a mut LayoutTree<T>,
    measure: M,
}

impl<T, M: FnMut(NodeId, Option<&mut T>, Option<f32>) -> Xy<f32>> TreeAdapter for Adapter<'_, T, M> {
    type NodeId = NodeId;

    #[inline(always)]
    fn root(&self) -> NodeId {
        self.tree.root
    }

    #[inline(always)]
    fn parent(&self, i: NodeId) -> NodeId {
        self.tree.nodes[i].parent.unwrap_or(self.tree.root)
    }

    #[inline(always)]
    fn first_child(&self, i: NodeId) -> Option<NodeId> {
        self.tree.nodes[i].first_child
    }

    #[inline(always)]
    fn next_sibling(&self, i: NodeId) -> Option<NodeId> {
        self.tree.nodes[i].next_sibling
    }

    #[inline(always)]
    fn node_layout_params(&self, i: NodeId) -> &LayoutParams {
        &self.tree.nodes[i].params
    }

    #[inline(always)]
    fn node_layout_data(&self, i: NodeId) -> &NodeLayoutData {
        &self.tree.nodes[i].layout
    }

    #[inline(always)]
    fn node_layout_data_mut(&mut self, i: NodeId) -> &mut NodeLayoutData {
        &mut self.tree.nodes[i].layout
    }

    #[inline(always)]
    fn layout_data(&mut self) -> &mut LayoutData<NodeId> {
        &mut self.tree.layout_data
    }

    fn measure_text_width(&mut self, i: NodeId) -> f32 {
        (self.measure)(i, self.tree.nodes[i].context.as_mut(), None).x
    }

    fn measure_text_height(&mut self, i: NodeId, wrap_width: Option<f32>) -> f32 {
        (self.measure)(i, self.tree.nodes[i].context.as_mut(), wrap_width).y
    }

    #[inline(always)]
    fn window_size(&self) -> Xy<f32> {
        self.tree.window_size
    }

    #[inline(always)]
    fn write_output_rect(&mut self, i: NodeId, rect: XyRect, _local_rect: XyRect) {
        self.tree.nodes[i].rect = rect;
    }

    fn layout_warning(&mut self, warning: LayoutWarning<NodeId>) {
        self.tree.warnings.push(warning);
    }
}

impl<T> LayoutTree<T> {
    /// Draws the layout from the last `compute_layout()` as an SVG image.
    pub fn to_svg(&self) -> String {
        use std::fmt::Write;
        let window = self.nodes[self.root].rect;
        let mut svg = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" font-family=\"monospace\" font-size=\"5\">\n", window.x[1], window.y[1]);
        let _ = writeln!(svg, "<rect width=\"100%\" height=\"100%\" fill=\"#faf8f5\"/>");
        let mut stack = vec![self.root];
        while let Some(i) = stack.pop() {
            let node = &self.nodes[i];
            if node.params.disabled {
                continue;
            }
            let color = node_color(i);
            let rect = node.rect;
            let size = node.params.size;
            let id = format!("{i:?}");
            let id = format!("[{}]", id.trim_start_matches("NodeId(").trim_end_matches(')'));
            let _ = writeln!(svg, "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{color}\" fill-opacity=\"0.06\" stroke=\"{color}\"/>", rect.x[0], rect.y[0], rect.x[1] - rect.x[0], rect.y[1] - rect.y[0]);
            let _ = writeln!(svg, "<text x=\"{}\" y=\"{}\" fill=\"{color}\">{id} {} × {}</text>", rect.x[0] + 1.5, rect.y[0] + 6.0, size_label(size.x), size_label(size.y));
            stack.extend(self.children(i));
        }
        svg.push_str("</svg>\n");
        svg
    }
}

fn node_color(i: NodeId) -> String {
    use slotmap::Key;
    let mut hash = i.data().as_ffi().wrapping_add(0x9e37_79b9_7f4a_7c15);
    hash = (hash ^ (hash >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    hash = (hash ^ (hash >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    hash ^= hash >> 31;
    let hue = hash % 360;
    let saturation = 45 + (hash >> 16) % 25;
    let lightness = 32 + (hash >> 32) % 14;
    format!("hsl({hue}, {saturation}%, {lightness}%)")
}

fn size_label(size: Size) -> String {
    let round = |value: f32| (value * 100.0).round() / 100.0;
    match size {
        Size::Pixels(pixels) => format!("{}px", round(pixels)),
        Size::Frac(frac) => format!("{}fr", round(frac)),
        Size::Fill => "fill".to_string(),
        Size::FitContent => "fit".to_string(),
        Size::AspectRatio(ratio) => format!("ratio {}", round(ratio)),
    }
}