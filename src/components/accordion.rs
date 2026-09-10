use crate as keru;
use keru::*;
use keru::node_library::*;

pub struct Accordion<'a, T: Copy> {
    pub sections: &'a [T],
    /// If `true`, show a plus/minus icon in each header that reflects the open/closed state.
    pub indicators: bool,
}

impl<'a, T: Copy> Accordion<'a, T> {
    pub fn new(sections: &'a [T]) -> Self {
        Self { sections, indicators: true }
    }

    /// Set whether to show the plus/minus open/close indicators.
    pub fn indicators(mut self, indicators: bool) -> Self {
        self.indicators = indicators;
        self
    }
}

pub struct AccordionSection<T> {
    pub value: T,
    pub header: UiParent,
    pub body: Option<UiParent>,
    pub expanded: bool,
}

pub struct AccordionResult<T> {
    pub sections: Vec<AccordionSection<T>>,
}

impl<'a, T: Copy> Component for Accordion<'a, T> {
    type State = Vec<bool>;
    type AddResult = AccordionResult<T>;
    type ComponentOutput = ();

    fn add_to_ui(&mut self, ui: &mut Ui, expanded: &mut Vec<bool>) -> Self::AddResult {
        #[node_key] const SECTION: NodeKey;
        #[node_key] const HEADER: NodeKey;
        #[node_key] const BODY: NodeKey;

        expanded.resize(self.sections.len(), false);

        for i in 0..self.sections.len() {
            if ui.is_clicked(HEADER.sibling(i)) {
                expanded[i] = !expanded[i];
            }
        }

        let outer_stack = V_STACK
            .size_x(Size::Fill)
            .size_y(Size::FitContent)
            .stack_spacing(4.0)
            .animate_layout(true);

        let header = BUTTON
            .size_x(Size::Fill)
            .size_y(Size::FitContent)
            .sense_click(true)
            .animate_layout(true)
            .fill(ui.theme().muted_background);

        let body = PANEL
            .size_x(Size::Fill)
            .size_y(Size::FitContent)
            .grow_from_top()
            .shrink_to_top()
            .animate_layout(true)
            .clip_children_y(true)
            .fill(ui.theme().background);

        let indicator = ICON
            .size_symm(Size::Pixels(20.0))
            .fill(ui.theme().text_primary);

        let content_slot = CONTAINER
            .size_x(Size::Fill)
            .size_y(Size::FitContent);


        let row = H_STACK
            .size_x(Size::Fill)
            .size_y(Size::FitContent);

        let mut sections = Vec::with_capacity(self.sections.len());

        ui.add(outer_stack).nest(|| {
            for i in 0..self.sections.len() {
                let is_open = expanded[i];
                let header = header.key(HEADER.sibling(i)).accessibility_selected(is_open);

                let header_button_slot = ui.add(header);
                let header_parent = if self.indicators {
                    header_button_slot.nest(|| {
                        ui.add(row).nest(|| {
                            let icon = if is_open { ICON_UP } else { ICON_DOWN };
                            ui.add(indicator.static_svg(icon));
                            ui.add(content_slot)
                        })
                    })
                } else {
                    header_button_slot
                };
                
                let body_parent = if is_open {
                    Some(ui.add(body.key(BODY.sibling(i))))
                } else {
                    None
                };
                sections.push(AccordionSection {
                    value: self.sections[i],
                    header: header_parent,
                    body: body_parent,
                    expanded: is_open,
                });
            }
        });

        AccordionResult { sections }
    }
}
