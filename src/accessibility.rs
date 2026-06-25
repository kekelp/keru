//! Building an AccessKit tree from the Keru node tree, and handling the
//! accessibility actions that come back from a screen reader.
//!
//! The queue-based adapter wrapper itself lives in [`crate::keru_accesskit`];
//! this module is the bridge between that adapter and the actual Keru UI state.

use accesskit::{Action, ActionRequest, Node as AccesskitNode, NodeId, Rect, Role, Tree, TreeId, TreeUpdate};

use crate::*;

impl Ui {
    /// Builds a full AccessKit [`TreeUpdate`] mirroring the current Keru node
    /// tree. The Keru root node becomes the AccessKit window root, and every
    /// visible node below it becomes a child node.
    ///
    /// Node ids are reused directly: an AccessKit [`NodeId`] is the Keru
    /// [`Id`]'s inner `u64`, so actions coming back from the screen reader can
    /// be mapped straight back onto a Keru node.
    pub(crate) fn build_accesskit_tree(&mut self) -> TreeUpdate {
        let mut nodes = Vec::with_capacity(20);
        self.build_accesskit_subtree(ROOT_I, &mut nodes);

        // The focused node must actually be present in the emitted tree (a
        // focused node could have since been hidden), otherwise AccessKit
        // rejects the update. Fall back to the window root.
        let focus = self
            .sys
            .focused
            .map(|id| NodeId(id.0))
            .filter(|focus_id| nodes.iter().any(|(id, _)| id == focus_id))
            .unwrap_or(WINDOW_NODE_ID);

        TreeUpdate {
            nodes,
            tree: Some(Tree::new(WINDOW_NODE_ID)),
            tree_id: TreeId::ROOT,
            focus,
        }
    }

    /// Builds a minimal [`TreeUpdate`] that only updates the focus.
    pub(crate) fn update_accesskit_focus_if_active(&mut self) {
        if let Some(accesskit) = &mut self.sys.accesskit {
            accesskit.update_if_active(|| {
                let focus = self.sys.focused.map(|id| NodeId(id.0)).unwrap_or(WINDOW_NODE_ID);
                TreeUpdate {
                    nodes: Vec::new(),
                    tree: None,
                    tree_id: TreeId::ROOT,
                    focus,
                }
            });
        }
    }

    /// Recursively builds the AccessKit node for `i` and all of its visible
    /// descendants, pushing each `(NodeId, Node)` pair onto `out` in the order
    /// AccessKit expects (parent before children).
    fn build_accesskit_subtree(&mut self, i: NodeI, out: &mut Vec<(NodeId, AccesskitNode)>) {
        let keru_node = &self.sys.nodes[i];
        let is_root = i == ROOT_I;

        let mut children = Vec::with_capacity(5);
        for_each_child!(self, self.sys.nodes[i], child, {
            children.push(child);
        });

        // The tree root must be a Window for AccessKit/UIA to anchor on it.
        let role = if is_root { Role::Window } else { keru_node.params.accessibility.role };


        // We have to make a decision of either using the node's text as a label for the accesskit node, or to add a full subtree of TextRun nodes so that screen readers can read it word by word or character by character.
        // Looking at the default behavior on the windows narrator app itself, it seems that buttons shouldn't have the word-by-word stuff.
        // I would have expected AccessKit's Paragraph to be the one that can be read word by word, but apparently that one can't even be navigated to at all. It just gets skipped.
        // todo: maybe Heading should also count.

        let build_subtree_for_word_by_word_reading = matches!(role, Role::Label);

        let mut node = AccesskitNode::new(role);

        let size = self.sys.size;
        let rect = self.sys.nodes[i].real_rect;
        node.set_bounds(Rect {
            x0: (rect[X][0] * size[X]) as f64,
            y0: (rect[Y][0] * size[Y]) as f64,
            x1: (rect[X][1] * size[X]) as f64,
            y1: (rect[Y][1] * size[Y]) as f64,
        });

        if is_root {
            node.set_label("Keru window");
        }

        if keru_node.params.interact.focusable {
            node.add_action(Action::Focus);
        }
        if keru_node.params.interact.senses.contains(Sense::CLICK) {
            node.add_action(Action::Click);
        }
        // Selectable roles (Tab, ListItem, ...) report their selected state.
        // We only set it when true; a selectable-role node defaults to
        // "not selected", so unselected siblings need no explicit marking.
        if self.sys.nodes[i].params.accessibility.selected {
            node.set_selected(true);
        }

        if let Some(numeric_value) = self.sys.nodes[i].params.accessibility.numeric_value {
            node.set_numeric_value(numeric_value.value);
            node.set_min_numeric_value(numeric_value.min);
            node.set_max_numeric_value(numeric_value.max);
        }
        // todo: we could try to add this just for nodes that actually have a scrollable parent or grandparent, but I don't know if it's worth the trouble right now.
        if ! is_root {
            node.add_action(Action::ScrollIntoView);
        }

        let scrollable = self.sys.nodes[i].params.layout.scrollable;
        if scrollable[Y] {
            node.add_action(Action::ScrollUp);
            node.add_action(Action::ScrollDown);
        }
        if scrollable[X] {
            node.add_action(Action::ScrollLeft);
            node.add_action(Action::ScrollRight);
        }

        for action in self.sys.nodes[i].params.accessibility.actions.iter() {
            if let Some(action) = action.to_accesskit() {
                node.add_action(action);
            }
        }

        for &c in &children {
            node.push_child(NodeId(self.sys.nodes[c].id.0));
        }

        let node_id = if is_root { WINDOW_NODE_ID } else { NodeId(self.sys.nodes[i].id.0) };

        match &self.sys.nodes[i].text_i {
            Some(TextI::TextEdit(handle)) => {
                let edit = self.sys.renderer.text.get_text_edit_mut(handle);
                node.set_value(edit.raw_text());
                if let Some(placeholder) = edit.placeholder() {
                    node.set_placeholder(placeholder);
                }
                if !edit.showing_placeholder() {
                    edit.build_accesskit_nodes(&mut node, out, keru_text::accessibility::next_node_id);
                }
            }
            Some(TextI::TextBox(handle)) => {
                if build_subtree_for_word_by_word_reading {
                    self.sys.renderer.text.get_text_box_mut(handle).build_accesskit_nodes(&mut node, out, keru_text::accessibility::next_node_id);
                } else {
                    let name = self.sys.renderer.text.get_text_box_mut(handle).text();
                    node.set_label(name);
                }
            }
            None => {}
        }

        out.push((node_id, node));

        for c in children {
            self.build_accesskit_subtree(c, out);
        }
    }

    /// Handles an action requested by a screen reader, mapping it back onto the
    /// corresponding Keru node.
    pub(crate) fn handle_accesskit_action(&mut self, request: ActionRequest) {
        let id = Id(request.target_node.0);
        let Some(i) = self.sys.nodes.get_by_id(id) else {
            return;
        };

        self.sys.accesskit_actions.push((id, request.action));

        match request.action {
            Action::Focus => {
                self.set_focus_node(i, true);
                self.sys.scroll_node_into_view(i, 0.0, true);
            },
            Action::ScrollIntoView => {
                // we're using this event as a generic substitute for an "Action::NavigateTo" that doesn't exist.
                self.set_focus_node(i, true);
                self.sys.scroll_node_into_view(i, 0.0, true);
            },
            Action::Blur => {
                if self.sys.focused == Some(self.sys.nodes[i].id) {
                    self.unfocus();
                }
            },
            Action::Click => self.sys.push_synthetic_click(i),

            Action::ScrollUp => self.push_synthetic_scroll(i, Xy::new(0.0, ACCESSKIT_SYNTHETIC_SCROLL_STEP)),
            Action::ScrollDown => self.push_synthetic_scroll(i, Xy::new(0.0, -ACCESSKIT_SYNTHETIC_SCROLL_STEP)),
            Action::ScrollLeft => self.push_synthetic_scroll(i, Xy::new(ACCESSKIT_SYNTHETIC_SCROLL_STEP, 0.0)),
            Action::ScrollRight => self.push_synthetic_scroll(i, Xy::new(-ACCESSKIT_SYNTHETIC_SCROLL_STEP, 0.0)),

            Action::Collapse => {},
            Action::Expand => {},
            Action::CustomAction => {},
            Action::Decrement => {},
            Action::Increment => {},
            Action::HideTooltip => {},
            Action::ShowTooltip => {},
            Action::ReplaceSelectedText => {},

            Action::ScrollToPoint => {},
            Action::SetScrollOffset => {},
            Action::SetTextSelection => {},
            Action::SetSequentialFocusNavigationStartingPoint => {},
            Action::SetValue => {},
            Action::ShowContextMenu => {},

        }
        self.set_new_ui_input();
    }
}

const ACCESSKIT_SYNTHETIC_SCROLL_STEP: f32 = 0.3;

/// Exhaustive match to check that Accesskit doesn't add any new actions.
#[allow(dead_code)]
fn accesskit_action_to_flag(action: accesskit::Action) -> AccessibilityActions {
    match action {
        accesskit::Action::Click => AccessibilityActions::CLICK,
        accesskit::Action::Focus => AccessibilityActions::FOCUS,
        accesskit::Action::Blur => AccessibilityActions::BLUR,
        accesskit::Action::Collapse => AccessibilityActions::COLLAPSE,
        accesskit::Action::Expand => AccessibilityActions::EXPAND,
        accesskit::Action::CustomAction => AccessibilityActions::CUSTOM_ACTION,
        accesskit::Action::Decrement => AccessibilityActions::DECREMENT,
        accesskit::Action::Increment => AccessibilityActions::INCREMENT,
        accesskit::Action::HideTooltip => AccessibilityActions::HIDE_TOOLTIP,
        accesskit::Action::ShowTooltip => AccessibilityActions::SHOW_TOOLTIP,
        accesskit::Action::ReplaceSelectedText => AccessibilityActions::REPLACE_SELECTED_TEXT,
        accesskit::Action::ScrollDown => AccessibilityActions::SCROLL_DOWN,
        accesskit::Action::ScrollLeft => AccessibilityActions::SCROLL_LEFT,
        accesskit::Action::ScrollRight => AccessibilityActions::SCROLL_RIGHT,
        accesskit::Action::ScrollUp => AccessibilityActions::SCROLL_UP,
        accesskit::Action::ScrollIntoView => AccessibilityActions::SCROLL_INTO_VIEW,
        accesskit::Action::ScrollToPoint => AccessibilityActions::SCROLL_TO_POINT,
        accesskit::Action::SetScrollOffset => AccessibilityActions::SET_SCROLL_OFFSET,
        accesskit::Action::SetTextSelection => AccessibilityActions::SET_TEXT_SELECTION,
        accesskit::Action::SetSequentialFocusNavigationStartingPoint => AccessibilityActions::SET_SEQUENTIAL_FOCUS_NAVIGATION_STARTING_POINT,
        accesskit::Action::SetValue => AccessibilityActions::SET_VALUE,
        accesskit::Action::ShowContextMenu => AccessibilityActions::SHOW_CONTEXT_MENU,
    }
}
