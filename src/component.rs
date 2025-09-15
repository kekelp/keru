use crate as keru;
use keru::*;
use keru::Size::*;
use keru::Position::*;

// #[derive(Clone, Copy, Debug)]
// pub struct ComponentKey {
//     id: Id,
//     debug_name: &'static str,
// }

pub trait ComponentParams {
    type AddResult;
    type ComponentOutput;
    
    fn add_to_ui(self, ui: &mut Ui) -> Self::AddResult;

    // this returns an Option mostly just so that we can default impl it with None, but maybe that's useful in other ways?
    // as in, if the component is not currently added, maybe Ui::component_output can just see that and return None, instead of running the function anyway and (hopefully) getting a None?
    // todo: figure this out 
    fn component_output(_ui: &mut Ui) -> Option<Self::ComponentOutput> {
        None
    }

    fn component_key(&self) -> Option<NodeKey> {
        None
    }
}

impl Ui {
    #[track_caller]
    pub fn add_component<W: ComponentParams>(&mut self, component_params: W) -> W::AddResult {
        let key_opt = component_params.component_key();
        let component_key = match key_opt {
            Some(key) => key,
            None => NodeKey::new(Id(caller_location_id()), ""),
        };
        self.named_subtree(component_key).start(|| {
            W::add_to_ui(component_params, self)
        })
    }

    pub fn component_output<W: ComponentParams>(&mut self, component_key: NodeKey) -> Option<W::ComponentOutput> {
        self.named_subtree(component_key).start(|| {
            W::component_output(self)
        })
    }
}

pub struct SliderParams<'a> {
    pub value: &'a mut f32,
    pub min: f32,
    pub max: f32,
    // adding a key even though it's probably not needed, just to test it out.
    pub key: Option<NodeKey>,
}

#[node_key] const SLIDER_FILL: NodeKey;
#[node_key] const SLIDER_LABEL: NodeKey;

impl ComponentParams for SliderParams<'_> {
    type ComponentOutput = String;
    type AddResult = ();

    fn add_to_ui(self, ui: &mut Ui) {
        let mut new_value = *self.value;
        if let Some(drag) = ui.is_dragged(SLIDER_CONTAINER) {
            new_value += drag.relative_delta.x as f32 * (self.min - self.max);
        }

        if new_value.is_finite() {
            new_value = new_value.clamp(self.min, self.max);
            *self.value = new_value;
        }

        let filled_frac = (*self.value - self.min) / (self.max - self.min);

        #[node_key] const SLIDER_CONTAINER: NodeKey;
        let slider_container = PANEL
            .size_x(Size::Fill)
            .size_y(Size::Pixels(45))
            .sense_drag(true)
            // .shape(Shape::Rectangle { corner_radius: 36.0 })
            .key(SLIDER_CONTAINER);
        
        let slider_fill = PANEL
            .size_y(Fill)
            .size_x(Size::Frac(filled_frac))
            .color(Color::KERU_RED)
            .position_x(Start)
            .padding_x(1)
            .absorbs_clicks(false)
            // .shape(Shape::Rectangle { corner_radius: 16.0 })
            .key(SLIDER_FILL);

        // todo: don't allocate here
        let text = format!("{:.2}", self.value);

        let label = TEXT.text(&text).key(SLIDER_LABEL);

        ui.add(slider_container).nest(|| {
            ui.add(slider_fill);
            ui.add(label);
        });
    }

    fn component_key(&self) -> Option<NodeKey> {
        self.key
    }

    fn component_output(ui: &mut Ui) -> Option<Self::ComponentOutput> {
        ui.get_text(SLIDER_LABEL).map(|x| x.to_string())
    }
}