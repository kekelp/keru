use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;
use slab::Slab;


struct Panes {
    slab: Slab<Pane>,
    next_tab_id: usize,
}

#[derive(Debug)]
enum PaneKind {
    Split { axis: Axis },
    Content { active_tab: Option<usize> },
    Tab { label: String, id: usize },
}

#[derive(Debug)]
struct Pane {
    kind: PaneKind,
    weight: f32,
    first_child: Option<usize>,
    next_sibling: Option<usize>,
    parent: Option<usize>,
}

impl Panes {
    fn new_content(&mut self, weight: f32, next_sibling: Option<usize>, parent: Option<usize>) -> usize {
        let content = self.slab.insert(Pane {
            kind: PaneKind::Content { active_tab: None },
            weight,
            first_child: None,
            next_sibling,
            parent,
        });
        self.add_tab(content);
        content
    }

    // Removes tab_index from content's list. Returns (prev, old_next).
    fn detach_tab(&mut self, content_index: usize, tab_index: usize) -> (Option<usize>, Option<usize>) {
        let old_next = self.slab[tab_index].next_sibling;
        let mut prev = None;
        let mut cur = self.slab[content_index].first_child;
        while let Some(i) = cur {
            if i == tab_index { break; }
            prev = Some(i);
            cur = self.slab[i].next_sibling;
        }
        match prev {
            None => self.slab[content_index].first_child = old_next,
            Some(p) => self.slab[p].next_sibling = old_next,
        }
        self.slab[tab_index].next_sibling = None;
        (prev, old_next)
    }

    // Inserts tab_index into content's list at insertion_index (clamped to end).
    fn insert_tab_at(&mut self, content_index: usize, tab_index: usize, insertion_index: usize) {
        let mut idx = 0;
        let mut prev_node: Option<usize> = None;
        let mut cur = self.slab[content_index].first_child;
        while let Some(i) = cur {
            if idx == insertion_index { break; }
            idx += 1;
            prev_node = Some(i);
            cur = self.slab[i].next_sibling;
        }
        let next_node = match prev_node {
            None => self.slab[content_index].first_child,
            Some(p) => self.slab[p].next_sibling,
        };
        self.slab[tab_index].next_sibling = next_node;
        match prev_node {
            None => self.slab[content_index].first_child = Some(tab_index),
            Some(p) => self.slab[p].next_sibling = Some(tab_index),
        }
    }

    fn add_tab(&mut self, content_index: usize) {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        let tab = self.slab.insert(Pane {
            kind: PaneKind::Tab { label: format!("Tab {id}"), id },
            weight: 1.0,
            first_child: None,
            next_sibling: None,
            parent: Some(content_index),
        });
        self.insert_tab_at(content_index, tab, usize::MAX);
        let PaneKind::Content { active_tab } = &mut self.slab[content_index].kind else { return };
        *active_tab = Some(tab);
    }

    fn remove_tab(&mut self, content_index: usize, tab_index: usize) {
        let (prev, old_next) = self.detach_tab(content_index, tab_index);
        let PaneKind::Content { active_tab } = &mut self.slab[content_index].kind else { return };
        if *active_tab == Some(tab_index) {
            *active_tab = old_next.or(prev);
        }
        self.slab.remove(tab_index);
        if self.slab[content_index].first_child.is_none() {
            self.remove(content_index);
        }
    }

    fn split(&mut self, content_index: usize, axis: Axis, after: bool) -> usize {
        let parent = self.slab[content_index].parent.expect("content must have parent");
        let parent_axis = match self.slab[parent].kind {
            PaneKind::Split { axis } => axis,
            _ => unreachable!(),
        };
        let neighbor_weight = self.slab[content_index].weight;

        if parent_axis == axis {
            let half = neighbor_weight / 2.0;
            self.slab[content_index].weight = half;
            let new_content = self.new_content(half, None, Some(parent));
            if after {
                let old_next = self.slab[content_index].next_sibling;
                self.slab[new_content].next_sibling = old_next;
                self.slab[content_index].next_sibling = Some(new_content);
            } else {
                self.slab[new_content].next_sibling = Some(content_index);
                self.redirect_child(parent, content_index, Some(new_content));
            }
            new_content
        } else {
            let old_next = self.slab[content_index].next_sibling;
            let old_weight = self.slab[content_index].weight;

            let new_content = self.new_content(1.0, None, None);
            self.slab[content_index].weight = 1.0;

            let (first, second) = if after { (content_index, new_content) } else { (new_content, content_index) };
            self.slab[first].next_sibling = Some(second);
            self.slab[second].next_sibling = None;

            let new_split = self.slab.insert(Pane {
                kind: PaneKind::Split { axis },
                weight: old_weight,
                first_child: Some(first),
                next_sibling: old_next,
                parent: Some(parent),
            });

            self.slab[content_index].parent = Some(new_split);
            self.slab[new_content].parent = Some(new_split);

            self.redirect_child(parent, content_index, Some(new_split));
            new_content
        }
    }

    fn reorder_tab(&mut self, content_index: usize, tab_index: usize, insertion_index: usize) {
        self.detach_tab(content_index, tab_index);
        self.insert_tab_at(content_index, tab_index, insertion_index);
    }

    fn move_tab(&mut self, tab_index: usize, from_content: usize, to_content: usize, insertion_index: usize) {
        if from_content == to_content { return; }

        let (prev, old_next) = self.detach_tab(from_content, tab_index);

        let PaneKind::Content { active_tab } = &mut self.slab[from_content].kind else { return };
        if *active_tab == Some(tab_index) {
            *active_tab = old_next.or(prev);
        }

        self.slab[tab_index].parent = Some(to_content);
        self.insert_tab_at(to_content, tab_index, insertion_index);

        let PaneKind::Content { active_tab } = &mut self.slab[to_content].kind else { return };
        *active_tab = Some(tab_index);

        if self.slab[from_content].first_child.is_none() {
            self.remove(from_content);
        }
    }

    fn remove(&mut self, content_index: usize) {
        let parent = self.slab[content_index].parent.expect("content must have parent");

        // Remove all tab children
        let mut cur = self.slab[content_index].first_child;
        while let Some(tab_i) = cur {
            cur = self.slab[tab_i].next_sibling;
            self.slab.remove(tab_i);
        }

        let old_next = self.slab[content_index].next_sibling;
        self.redirect_child(parent, content_index, old_next);
        self.slab.remove(content_index);

        let mut count = 0;
        let mut only_child = None;
        let mut cur = self.slab[parent].first_child;
        while let Some(i) = cur {
            count += 1;
            only_child = Some(i);
            cur = self.slab[i].next_sibling;
        }

        if count == 1 && let Some(grandparent) = self.slab[parent].parent {
            let child = only_child.unwrap();
            self.slab[child].parent = Some(grandparent);
            self.slab[child].weight = self.slab[parent].weight;
            self.slab[child].next_sibling = self.slab[parent].next_sibling;
            self.redirect_child(grandparent, parent, Some(child));
            self.slab.remove(parent);
        }
    }

    fn redirect_child(&mut self, parent: usize, from: usize, to: Option<usize>) {
        if self.slab[parent].first_child == Some(from) {
            self.slab[parent].first_child = to;
        } else {
            let mut cur = self.slab[parent].first_child;
            while let Some(i) = cur {
                if self.slab[i].next_sibling == Some(from) {
                    self.slab[i].next_sibling = to;
                    break;
                }
                cur = self.slab[i].next_sibling;
            }
        }
    }

    fn tab_node(tab_id: usize, is_active: bool, animate_position: bool) -> Node<'static> {
        BUTTON.key(TAB.sibling(tab_id))
            .animate_position(animate_position)
            .size(Size::Pixels(TAB_WIDTH), Size::Pixels(TAB_BAR_HEIGHT))
            .sense_drag(true)
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::TOP, corner_radius: 10.0 })
            .color(if is_active { Color::KERU_BLUE } else { Color::GREY })
    }

    fn render_pane(&self, index: usize, size_along: Size, ui: &mut Ui, parent_axis: Option<Axis>, drag_state: &mut Option<TabDragState>) {
        let (size_x, size_y) = match parent_axis {
            Some(Axis::X) => (size_along, Size::Fill),
            Some(Axis::Y) => (Size::Fill, size_along),
            None => (Size::Fill, Size::Fill),
        };

        match &self.slab[index].kind {
            PaneKind::Split { axis } => {
                let axis = *axis;
                let mut child = self.slab[index].first_child;

                let total: f32 = {
                    let mut sum = 0.0;
                    let mut cur = child;
                    while let Some(i) = cur { sum += self.slab[i].weight; cur = self.slab[i].next_sibling; }
                    sum
                };

                let container = match axis {
                    Axis::X => H_STACK,
                    Axis::Y => V_STACK,
                }.animate_position(true).size_x(size_x).size_y(size_y).stack_spacing(0.0).key(SPLIT_CONTAINER.sibling(index));

                let wall = match axis {
                    Axis::X => PANEL.size_x(Size::Pixels(WALL_THICKNESS)).size_y(Size::Fill),
                    Axis::Y => PANEL.size_x(Size::Fill).size_y(Size::Pixels(WALL_THICKNESS)),
                }.color(Color::GREY).animate_position(true);

                let hitbox = match axis {
                    Axis::X => node_library::CONTAINER.size_x(Size::Pixels(WALL_HITBOX_THICKNESS)).size_y(Size::Fill).position(Pos::Center, Pos::Center),
                    Axis::Y => node_library::CONTAINER.size_x(Size::Fill).size_y(Size::Pixels(WALL_HITBOX_THICKNESS)).position(Pos::Center, Pos::Center),
                }.sense_drag(true);

                let insert_hitbox = match axis {
                    Axis::X => PANEL.color(Color::GREEN.with_alpha(0.5))
                        .size_x(Size::Pixels(WALL_HITBOX_THICKNESS)).size_y(Size::Frac(0.3))
                        .anchor_symm(Anchor::Center).free_placement(true).z_index(10.0).sense_drag_drop_target(true),
                    Axis::Y => PANEL.color(Color::GREEN.with_alpha(0.5))
                        .size_x(Size::Frac(0.3)).size_y(Size::Pixels(WALL_HITBOX_THICKNESS))
                        .anchor_symm(Anchor::Center).free_placement(true).z_index(10.0).sense_drag_drop_target(true),
                };

                ui.add(container).nest(|| {
                    if drag_state.is_some() {
                        ui.add(match axis {
                            Axis::X => insert_hitbox.position(Pos::Frac(0.0), Pos::Center),
                            Axis::Y => insert_hitbox.position(Pos::Center, Pos::Frac(0.0)),
                        }.key(WALL_INSERT_FIRST.sibling(index)));
                    }

                    let mut total_weight = 0.0;
                    let mut wall_idx = 0usize;
                    while let Some(i) = child {
                        let pane_size = Size::Frac(self.slab[i].weight / total);
                        self.render_pane(i, pane_size, ui, Some(axis), drag_state);
                        total_weight += self.slab[i].weight;
                        if self.slab[i].next_sibling.is_some() {
                            ui.add(wall).nest(|| { ui.add(hitbox.key(WALL.sibling(index).sibling(wall_idx).sibling(axis))); });
                            if drag_state.is_some() {
                                let frac = total_weight / total;
                                ui.add(match axis {
                                    Axis::X => insert_hitbox.position(Pos::Frac(frac), Pos::Center),
                                    Axis::Y => insert_hitbox.position(Pos::Center, Pos::Frac(frac)),
                                }.key(WALL_INSERT_INNER.sibling(i)));
                            }
                            wall_idx += 1;
                        }
                        child = self.slab[i].next_sibling;
                    }

                    if drag_state.is_some() {
                        ui.add(match axis {
                            Axis::X => insert_hitbox.position(Pos::Frac(1.0), Pos::Center),
                            Axis::Y => insert_hitbox.position(Pos::Center, Pos::Frac(1.0)),
                        }.key(WALL_INSERT_LAST.sibling(index)));
                    }
                });
            }
            PaneKind::Content { active_tab } => {
                let active_tab = *active_tab;

                let stack = V_STACK.size_x(size_x).size_y(size_y).stack_arrange(Arrange::Start).padding(0.0).stack_spacing(0.0).key(CONTENT_PANE.sibling(index)).animate_position(true).children_can_hide(true).grow_from_left();

                ui.add(stack).nest(|| {
                    let tab_bar_hitbox = node_library::CONTAINER
                        .size_x(Size::Fill).size_y(Size::Pixels(TAB_BAR_HEIGHT * 2.0))
                        .position(Pos::Center, Pos::Center)
                        .absorbs_clicks(false)
                        .free_placement(true)
                        .sense_drag_drop_target(true)
                        .z_index(10.0)
                        .key(TAB_BAR_HITBOX.sibling(index));

                    let tab_bar = H_SCROLL_STACK
                        .size_x(Size::Fill).size_y(Size::Pixels(TAB_BAR_HEIGHT))
                        .stack_arrange(Arrange::Start)
                        .stack_spacing(0.0)
                        .key(TAB_BAR.sibling(index));

                    ui.add(tab_bar).nest(|| {
                        ui.add(tab_bar_hitbox);

                        let spacer = SPACER.size_x(Size::Pixels(TAB_WIDTH)).size_y(Size::FitContent);
                        let show_spacer = |ui: &mut Ui, render_idx: usize| {
                            if let Some(ds) = drag_state.as_ref() {
                                if ds.hovered_content == Some(index) && ds.insertion_index == render_idx {
                                    ui.add(spacer);
                                }
                            }
                        };

                        let mut render_idx = 0;
                        let mut tab = self.slab[index].first_child;
                        while let Some(t) = tab {
                            let PaneKind::Tab { label, id: tab_id } = &self.slab[t].kind else { break };
                            let (tab_id, label) = (*tab_id, label.clone());
                            let is_active = active_tab == Some(t);
                            let is_dragged = drag_state.as_ref().map_or(false, |ds| ds.tab_index == t);

                            if !is_dragged {
                                show_spacer(ui, render_idx);
                                ui.add(Panes::tab_node(tab_id, is_active, true)).nest(|| {
                                    ui.add(H_STACK).nest(|| {
                                        ui.add(TEXT.text(label.as_str()).text_size(18.0).text_selectable(false));
                                        ui.add(BUTTON.key(CLOSE_TAB.sibling(tab_id)).text("✕").text_size(18.0).padding(5.0).color(Color::KERU_RED.with_alpha(0.3)).position_x(Pos::End));
                                    });
                                });
                                render_idx += 1;
                            }

                            tab = self.slab[t].next_sibling;
                        }
                        show_spacer(ui, render_idx);
                        ui.add(BUTTON.animate_position(true).key(ADD_TAB.sibling(index)).padding(5.0).text_size(18.0).size_symm(Size::Pixels(TAB_BAR_HEIGHT - 4.0)).text("+"));
                    });

                    let active_tab_id = active_tab.and_then(|t| {
                        if let PaneKind::Tab { id, .. } = &self.slab[t].kind { Some(*id) } else { None }
                    });
                    let body = PANEL.size_x(Size::Fill).size_y(Size::Fill).shape(Shape::Rectangle { rounded_corners: RoundedCorners::BOTTOM, corner_radius: 10.0 }).absorbs_clicks(false)
                        .key(CONTENT_BODY.sibling(active_tab_id.unwrap_or(usize::MAX))).animate_position(true).sense_drag_drop_target(true);

                    ui.add(body).nest(|| {
                        ui.add(H_STACK).nest(|| {
                            ui.add(BUTTON.animate_position(true).key(SPLIT_LEFT.sibling(index)).text("←"));
                            ui.add(BUTTON.animate_position(true).key(SPLIT_RIGHT.sibling(index)).text("→"));
                            ui.add(BUTTON.animate_position(true).key(SPLIT_UP.sibling(index)).text("↑"));
                            ui.add(BUTTON.animate_position(true).key(SPLIT_DOWN.sibling(index)).text("↓"));
                            ui.add(BUTTON.animate_position(true).key(REMOVE_PANE.sibling(index)).text("✕").color(Color::KERU_RED));
                        });


                        if drag_state.is_some() {
                            let h_edge = PANEL
                                .color(Color::GREY.with_alpha(0.5))
                                .size_x(Size::Pixels(SPLIT_EDGE_SIZE)).size_y(Size::Frac(0.3)).anchor_x(Anchor::Center)
                                .free_placement(true).sense_drag_drop_target(true).absorbs_clicks(false);
                            let v_edge = PANEL
                                .color(Color::GREY.with_alpha(0.5))
                                .size_x(Size::Frac(0.3)).size_y(Size::Pixels(SPLIT_EDGE_SIZE)).anchor_y(Anchor::Center)
                                .free_placement(true).sense_drag_drop_target(true).absorbs_clicks(false);

                            ui.add(h_edge.position(Pos::Frac(0.25), Pos::Center).key(SPLIT_EDGE_LEFT.sibling(index)));
                            ui.add(h_edge.position(Pos::Frac(0.75), Pos::Center).key(SPLIT_EDGE_RIGHT.sibling(index)));
                            ui.add(v_edge.position(Pos::Center, Pos::Frac(0.25)).key(SPLIT_EDGE_TOP.sibling(index)));
                            ui.add(v_edge.position(Pos::Center, Pos::Frac(0.75)).key(SPLIT_EDGE_BOTTOM.sibling(index)));
                        }
                    });
                });
            }
            PaneKind::Tab { .. } => {}
        }
    }
}


pub struct State {
    panes: Panes,
}

#[node_key] const SPLIT_LEFT: NodeKey;
#[node_key] const SPLIT_RIGHT: NodeKey;
#[node_key] const SPLIT_UP: NodeKey;
#[node_key] const SPLIT_DOWN: NodeKey;
#[node_key] const REMOVE_PANE: NodeKey;
#[node_key] const SPLIT_CONTAINER: NodeKey;
#[node_key] const WALL: NodeKey;
#[node_key] const TAB: NodeKey;
#[node_key] const TAB_BAR: NodeKey;
#[node_key] const CLOSE_TAB: NodeKey;
#[node_key] const ADD_TAB: NodeKey;
#[node_key] const TAB_BAR_HITBOX: NodeKey;
#[node_key] const CONTENT_BODY: NodeKey;
#[node_key] const SPLIT_EDGE_LEFT: NodeKey;
#[node_key] const SPLIT_EDGE_RIGHT: NodeKey;
#[node_key] const SPLIT_EDGE_TOP: NodeKey;
#[node_key] const SPLIT_EDGE_BOTTOM: NodeKey;
#[node_key] const WALL_INSERT_FIRST: NodeKey;
#[node_key] const WALL_INSERT_INNER: NodeKey;
#[node_key] const WALL_INSERT_LAST: NodeKey;
#[node_key] const CONTENT_PANE: NodeKey;
#[node_key] const DEST_INDICATOR: NodeKey;

const WALL_THICKNESS: f32 = 10.0;
const WALL_HITBOX_THICKNESS: f32 = 40.0;
const TAB_BAR_HEIGHT: f32 = 40.0;
const TAB_WIDTH: f32 = 100.0;
const SPLIT_EDGE_SIZE: f32 = 60.0;

struct TabDragState {
    tab_index: usize,
    tab_id: usize,
    content_index: usize,
    drag: Drag,
    locked_y: Option<f32>,
    hovered_content: Option<usize>,
    insertion_index: usize,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    // ui.set_global_animation_speed(0.2);
    
    let mut drag_state: Option<TabDragState> = None;
    for (i, pane) in &state.panes.slab {
        let PaneKind::Tab { id: tab_id, .. } = pane.kind else { continue };
        let Some(drag) = ui.is_dragged(TAB.sibling(tab_id)) else { continue };
        drag_state = Some(TabDragState {
            tab_index: i,
            tab_id,
            content_index: pane.parent.unwrap(),
            drag,
            locked_y: None,
            hovered_content: None,
            insertion_index: 0,
        });
        break;
    }

    if let Some(drag_state) = &mut drag_state {
        for (i, pane) in &state.panes.slab {
            let PaneKind::Content { .. } = pane.kind else { continue };
            if ui.is_drag_hovered_onto(TAB.sibling(drag_state.tab_id), TAB_BAR_HITBOX.sibling(i)).is_some() {
                drag_state.locked_y = ui.get_node(TAB_BAR.sibling(i))
                    .map(|n| { let r = n.rect(); (r[Axis::Y][0] + r[Axis::Y][1]) / 2.0 });
                drag_state.hovered_content = Some(i);

                let cursor_x = ui.cursor_position().x;
                let tab_bar_x = ui.get_node(TAB_BAR.sibling(i))
                    .map(|n| n.rect()[Axis::X][0])
                    .unwrap_or(0.0);
                let num_tabs = {
                    let mut count = 0;
                    let mut cur = state.panes.slab[i].first_child;
                    while let Some(t) = cur {
                        if t != drag_state.tab_index { count += 1; }
                        cur = state.panes.slab[t].next_sibling;
                    }
                    count
                };
                let raw = ((cursor_x - tab_bar_x) / TAB_WIDTH).max(0.0) as usize;
                drag_state.insertion_index = raw.min(num_tabs);
                break;
            }
        }
    }

    state.panes.render_pane(0, Size::Fill, ui, None, &mut drag_state);


    if let Some(dragged) = &drag_state {
        let py = dragged.locked_y.unwrap_or(dragged.drag.absolute_pos.y);
        let px = dragged.drag.absolute_pos.x;

        let PaneKind::Content { active_tab } = &state.panes.slab[dragged.content_index].kind else { unreachable!() };
        let is_active = *active_tab == Some(dragged.tab_index);
        let PaneKind::Tab { label, .. } = &state.panes.slab[dragged.tab_index].kind else { unreachable!() };

        let tab_node = Panes::tab_node(dragged.tab_id, is_active, false);

        ui.jump_to_root().nest(|| {
            ui.add(tab_node.absorbs_clicks(false).animate_position(true).anchor_symm(Anchor::Center).position(Pos::Pixels(px), Pos::Pixels(py)).z_index(1.0)).nest(|| {
                ui.add(H_STACK).nest(|| {
                    ui.add(TEXT.text(label.as_str()).text_size(18.0).text_selectable(false));
                    ui.add(BUTTON.key(CLOSE_TAB.sibling(dragged.tab_id)).text("✕").text_size(18.0).color(Color::KERU_RED.with_alpha(0.3)).absorbs_clicks(false).position_x(Pos::End));
                });
            });
        });

        // Destination indicator: default is a tab-sized rect under the dragged tab.
        // Overridden by split-edge hover (shows actual half-pane area) or wall-insert hover (shows strip near wall).
        let mut ind: (f32, f32, f32, f32) = (
            px - TAB_WIDTH / 2.0, py - TAB_BAR_HEIGHT / 2.0,
            px + TAB_WIDTH / 2.0, py + TAB_BAR_HEIGHT / 2.0,
        );

        let content_list: Vec<usize> = state.panes.slab.iter()
            .filter_map(|(i, p)| if matches!(p.kind, PaneKind::Content { .. }) { Some(i) } else { None })
            .collect();

        let split_list: Vec<(usize, Axis)> = state.panes.slab.iter()
            .filter_map(|(i, p)| if let PaneKind::Split { axis } = p.kind { Some((i, axis)) } else { None })
            .collect();

        let mut found = false;

        'split_edge: for target_content in &content_list {
            for (edge_key, axis, after) in [
                (SPLIT_EDGE_LEFT,   Axis::X, false),
                (SPLIT_EDGE_RIGHT,  Axis::X, true),
                (SPLIT_EDGE_TOP,    Axis::Y, false),
                (SPLIT_EDGE_BOTTOM, Axis::Y, true),
            ] {
                if ui.is_drag_hovered_onto(TAB.sibling(dragged.tab_id), edge_key.sibling(*target_content)).is_some() {
                    if let Some(n) = ui.get_node(CONTENT_PANE.sibling(*target_content)) {
                        let r = n.rect();
                        let (x0, x1) = (r[Axis::X][0], r[Axis::X][1]);
                        let (y0, y1) = (r[Axis::Y][0], r[Axis::Y][1]);
                        let mx = (x0 + x1) / 2.0;
                        let my = (y0 + y1) / 2.0;
                        ind = match (axis, after) {
                            (Axis::X, false) => (x0, y0, mx, y1),
                            (Axis::X, true)  => (mx, y0, x1, y1),
                            (Axis::Y, false) => (x0, y0, x1, my),
                            (Axis::Y, true)  => (x0, my, x1, y1),
                        };
                        found = true;
                    }
                    break 'split_edge;
                }
            }
        }

        if !found {
            'wall: for (split_i, axis) in &split_list {
                // For wall inserts, expand the indicator to fill the full cross-axis of the container.
                let cross = axis.other();
                let container_cross = ui.get_node(SPLIT_CONTAINER.sibling(*split_i))
                    .map(|n| { let r = n.rect(); (r[cross][0], r[cross][1]) });

                let expand_cross = |r: Xy<[f32; 2]>| -> (f32, f32, f32, f32) {
                    let (c0, c1) = container_cross.unwrap_or((r[cross][0], r[cross][1]));
                    match cross {
                        Axis::X => (c0, r[Axis::Y][0], c1, r[Axis::Y][1]),
                        Axis::Y => (r[Axis::X][0], c0, r[Axis::X][1], c1),
                    }
                };

                if ui.is_drag_hovered_onto(TAB.sibling(dragged.tab_id), WALL_INSERT_FIRST.sibling(*split_i)).is_some() {
                    if let Some(n) = ui.get_node(WALL_INSERT_FIRST.sibling(*split_i)) {
                        ind = expand_cross(n.rect());
                    }
                    break 'wall;
                }
                if ui.is_drag_hovered_onto(TAB.sibling(dragged.tab_id), WALL_INSERT_LAST.sibling(*split_i)).is_some() {
                    if let Some(n) = ui.get_node(WALL_INSERT_LAST.sibling(*split_i)) {
                        ind = expand_cross(n.rect());
                    }
                    break 'wall;
                }
                let mut cur = state.panes.slab[*split_i].first_child;
                while let Some(child_i) = cur {
                    if state.panes.slab[child_i].next_sibling.is_some() {
                        if ui.is_drag_hovered_onto(TAB.sibling(dragged.tab_id), WALL_INSERT_INNER.sibling(child_i)).is_some() {
                            if let Some(n) = ui.get_node(WALL_INSERT_INNER.sibling(child_i)) {
                                ind = expand_cross(n.rect());
                            }
                            break 'wall;
                        }
                    }
                    cur = state.panes.slab[child_i].next_sibling;
                }
            }
        }

        let (ix0, iy0, ix1, iy1) = ind;
        let (cx, cy) = ((ix0 + ix1) / 2.0, (iy0 + iy1) / 2.0);
        ui.jump_to_root().nest(|| {
            ui.add(PANEL
                .color(Color::KERU_BLUE.with_alpha(0.7))
                .free_placement(true)
                .anchor_symm(Anchor::Center)
                .position(Pos::Pixels(cx), Pos::Pixels(cy))
                .size_x(Size::Pixels(ix1 - ix0))
                .size_y(Size::Pixels(iy1 - iy0))
                .z_index(4.0)
                .absorbs_clicks(false)
                .animate_position(true)
                .key(DEST_INDICATOR));
        });
    }

    let split_indices: Vec<(usize, Axis)> = state.panes.slab.iter()
        .filter_map(|(i, p)| if let PaneKind::Split { axis } = p.kind { Some((i, axis)) } else { None })
        .collect();

    for (split_i, axis) in split_indices {
        let Some(container_px_size) = ui.get_node(SPLIT_CONTAINER.sibling(split_i))
            .map(|n| { let r = n.rect(); r[axis][1] - r[axis][0] })
            .filter(|&s| s > 0.0) else { continue };

        let mut cur = state.panes.slab[split_i].first_child;
        let mut wall_idx = 0usize;
        while let Some(left_i) = cur {
            let Some(right_i) = state.panes.slab[left_i].next_sibling else { break };
            if let Some(drag) = ui.is_dragged(WALL.sibling(split_i).sibling(wall_idx).sibling(axis)) {
                let delta_px = match axis { Axis::X => drag.absolute_delta.x, Axis::Y => drag.absolute_delta.y };
                let total = state.panes.slab[left_i].weight + state.panes.slab[right_i].weight;
                let delta_w = delta_px / container_px_size * total;
                let min = (TAB_BAR_HEIGHT * 2.0) / container_px_size * total;
                let new_left = (state.panes.slab[left_i].weight + delta_w).clamp(min, total - min);
                state.panes.slab[left_i].weight = new_left;
                state.panes.slab[right_i].weight = total - new_left;
            }
            wall_idx += 1;
            cur = state.panes.slab[left_i].next_sibling;
        }
    }

    let content_indices: Vec<usize> = state.panes.slab.iter()
        .filter_map(|(i, p)| if matches!(p.kind, PaneKind::Content { .. }) { Some(i) } else { None })
        .collect();

    for i in content_indices {
        if ui.is_clicked(SPLIT_LEFT.sibling(i)) { let _ = state.panes.split(i, Axis::X, false); }
        else if ui.is_clicked(SPLIT_RIGHT.sibling(i)) { let _ = state.panes.split(i, Axis::X, true); }
        else if ui.is_clicked(SPLIT_UP.sibling(i)) { let _ = state.panes.split(i, Axis::Y, false); }
        else if ui.is_clicked(SPLIT_DOWN.sibling(i)) { let _ = state.panes.split(i, Axis::Y, true); }
        else if ui.is_clicked(REMOVE_PANE.sibling(i)) { state.panes.remove(i); }
        else if ui.is_clicked(ADD_TAB.sibling(i)) {
            state.panes.add_tab(i);
        }
    }

    let tab_indices: Vec<(usize, usize, usize)> = state.panes.slab.iter()
        .filter_map(|(i, p)| if let PaneKind::Tab { id, .. } = p.kind { Some((i, id, p.parent.unwrap())) } else { None })
        .collect();

    for (tab_i, tab_id, content_i) in tab_indices {
        if ui.is_clicked(TAB.sibling(tab_id)) {
            let PaneKind::Content { active_tab } = &mut state.panes.slab[content_i].kind else { continue };
            *active_tab = Some(tab_i);
        } else if ui.is_click_released(CLOSE_TAB.sibling(tab_id)) {
            state.panes.remove_tab(content_i, tab_i);
        }
    }

    if let Some(dragged) = &drag_state {
        if let Some(hovered) = dragged.hovered_content {
            if ui.is_drag_released_onto(TAB.sibling(dragged.tab_id), TAB_BAR_HITBOX.sibling(hovered)).is_some() {
                let insertion_index = dragged.insertion_index;
                if hovered == dragged.content_index {
                    state.panes.reorder_tab(hovered, dragged.tab_index, insertion_index);
                } else {
                    state.panes.move_tab(dragged.tab_index, dragged.content_index, hovered, insertion_index);
                }
            }
        }

        let content_indices: Vec<usize> = state.panes.slab.iter()
            .filter_map(|(i, p)| if matches!(p.kind, PaneKind::Content { .. }) { Some(i) } else { None })
            .collect();

        let edges = [
            (SPLIT_EDGE_LEFT,   Axis::X, false),
            (SPLIT_EDGE_RIGHT,  Axis::X, true),
            (SPLIT_EDGE_TOP,    Axis::Y, false),
            (SPLIT_EDGE_BOTTOM, Axis::Y, true),
        ];

        'edge: for target_content in content_indices {
            for (edge_key, axis, after) in edges {
                if ui.is_drag_released_onto(TAB.sibling(dragged.tab_id), edge_key.sibling(target_content)).is_some() {
                    let new_content = state.panes.split(target_content, axis, after);
                    let placeholder = state.panes.slab[new_content].first_child.unwrap();
                    state.panes.detach_tab(new_content, placeholder);
                    state.panes.slab.remove(placeholder);
                    state.panes.move_tab(dragged.tab_index, dragged.content_index, new_content, 0);
                    break 'edge;
                }
            }
        }

        let split_info: Vec<(usize, Axis)> = state.panes.slab.iter()
            .filter_map(|(i, p)| if let PaneKind::Split { axis } = p.kind { Some((i, axis)) } else { None })
            .collect();

        'insert: for (split_i, axis) in split_info {
            let Some(first_child) = state.panes.slab[split_i].first_child else { continue };

            if ui.is_drag_released_onto(TAB.sibling(dragged.tab_id), WALL_INSERT_FIRST.sibling(split_i)).is_some() {
                let new_content = state.panes.split(first_child, axis, false);
                let placeholder = state.panes.slab[new_content].first_child.unwrap();
                state.panes.detach_tab(new_content, placeholder);
                state.panes.slab.remove(placeholder);
                state.panes.move_tab(dragged.tab_index, dragged.content_index, new_content, 0);
                break 'insert;
            }

            let mut cur = Some(first_child);
            while let Some(child_i) = cur {
                let next = state.panes.slab[child_i].next_sibling;
                let (key, target_child) = if next.is_some() {
                    (WALL_INSERT_INNER.sibling(child_i), child_i)
                } else {
                    (WALL_INSERT_LAST.sibling(split_i), child_i)
                };
                if ui.is_drag_released_onto(TAB.sibling(dragged.tab_id), key).is_some() {
                    let new_content = state.panes.split(target_child, axis, true);
                    let placeholder = state.panes.slab[new_content].first_child.unwrap();
                    state.panes.detach_tab(new_content, placeholder);
                    state.panes.slab.remove(placeholder);
                    state.panes.move_tab(dragged.tab_index, dragged.content_index, new_content, 0);
                    break 'insert;
                }
                cur = next;
            }
        }
    }
}

fn main() {
    let mut panes = Panes { slab: Slab::with_capacity(16), next_tab_id: 0 };
    let root = panes.slab.insert(Pane {
        kind: PaneKind::Split { axis: Axis::X },
        weight: 1.0,
        first_child: None,
        next_sibling: None,
        parent: None,
    });
    let v_split = panes.slab.insert(Pane {
        kind: PaneKind::Split { axis: Axis::Y },
        weight: 1.0,
        first_child: None,
        next_sibling: None,
        parent: Some(root),
    });
    panes.slab[root].first_child = Some(v_split);
    let c1 = panes.new_content(1.0, None, Some(v_split));
    panes.slab[v_split].first_child = Some(c1);

    run_example_loop(State { panes }, update_ui);
}
