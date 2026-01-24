use std::{fmt::Debug, hash::{Hash, Hasher}, marker::PhantomData};

use crate as keru;
use keru::*;
use keru::Size::*;
use keru::Position::*;

#[derive(Debug)]
pub struct ComponentKey<ComponentType: ?Sized> {
    id: Id,
    debug_name: &'static str,
    phantom: PhantomData<ComponentType>
}
impl<C> ComponentKey<C> {
    /// Create "siblings" of a key dynamically at runtime, based on a hashable value.
    pub fn sibling<H: Hash>(self, value: H) -> Self {
        let mut hasher = ahasher();
        self.id.0.hash(&mut hasher);
        value.hash(&mut hasher);
        let new_id = hasher.finish();

        return Self {
            id: Id(new_id),
            debug_name: self.debug_name,
            phantom: PhantomData::<C>,
        };
    }

    /// Create a key manually.
    /// 
    /// This is usually not needed: use the [`macro@component_key`] macro for static keys, and [`ComponentKey::sibling`] for dynamic keys.
    pub const fn new(id: Id, debug_name: &'static str) -> Self {
        return Self {
            id,
            debug_name,
            phantom: PhantomData::<C>
        };
    }

    pub const fn debug_name(&self) -> &'static str {
        return self.debug_name;
    }

    // Private function that removes the type marker.
    pub(crate) fn as_normal_key(&self) -> NodeKey {
        NodeKey::new(self.id, self.debug_name)
    }
}

// The key should be Copy even if the component params struct (C) isn't. Because of how derive(C) works, this needs to be impl'd manually.
impl<C> Clone for ComponentKey<C> {
    fn clone(&self) -> Self {
        Self { id: self.id, debug_name: self.debug_name, phantom: self.phantom }
    }
}
impl<C> Copy for ComponentKey<C> {}


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

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        None
    }
}

impl Ui {
    #[track_caller]
    pub fn add_component<T: ComponentParams>(&mut self, component_params: T) -> T::AddResult {
        let key = match component_params.component_key() {
            Some(key) => key.as_normal_key(),
            None => NodeKey::new(Id(caller_location_id()), ""),
        };

        // todo use normal add()??
        let (i, id) = self.add_or_update_node(key);
        self.set_params(i, &COMPONENT_ROOT.into());
        let res = UiParent::new(i).nest(|| {
    
            thread_local::push_subtree(id);
            
            let res = T::add_to_ui(component_params, self);
            
            thread_local::pop_subtree();
            
            return res;
        });

        return res;
    }

    pub fn component_output<T: ComponentParams>(&mut self, component_key: ComponentKey<T>) -> Option<T::ComponentOutput> {
        // No twinning here, so use this old closure one which does twinning
        self.component_key_subtree(component_key).start(|| {
            T::component_output(self)
        })
    }
}

pub struct SliderParams<'a> {
    pub value: &'a mut f32,
    pub min: f32,
    pub max: f32,
    pub clamp: bool,
}


impl ComponentParams for SliderParams<'_> {
    type ComponentOutput = ();
    type AddResult = ();
    
    fn add_to_ui(self, ui: &mut Ui) {
        with_arena(|arena| {

            #[node_key] const SLIDER_FILL: NodeKey;
            #[node_key] const SLIDER_LABEL: NodeKey;
            #[node_key] const SLIDER_CONTAINER: NodeKey;
                
            let mut new_value = *self.value;
            if let Some(drag) = ui.is_dragged(SLIDER_CONTAINER) {
                new_value += drag.relative_delta.x as f32 * (self.min - self.max);
            }

            if new_value.is_finite() {
                if self.clamp {
                    new_value = new_value.clamp(self.min, self.max);
                }
                *self.value = new_value;
            }

            let filled_frac = (*self.value - self.min) / (self.max - self.min);

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


            let text = bumpalo::format!(in arena, "{:.2}", self.value);
            let label = TEXT.text(&text).key(SLIDER_LABEL);

            ui.add(slider_container).nest(|| {
                ui.add(slider_fill);
                ui.add(label);
            });

        });
    }
}

impl<'a> SliderParams<'a> {
    pub fn new(value: &'a mut f32, min: f32, max: f32, clamp: bool) -> Self {
        Self { value, min, max, clamp }
    }
}


pub trait ComponentParams2<ADDRESULT> {
    type ComponentOutput;
    
    fn add_to_ui(self, ui: &mut Ui) -> ADDRESULT;

    // this returns an Option mostly just so that we can default impl it with None, but maybe that's useful in other ways?
    // as in, if the component is not currently added, maybe Ui::component_output can just see that and return None, instead of running the function anyway and (hopefully) getting a None?
    // todo: figure this out 
    fn component_output(_ui: &mut Ui) -> Option<Self::ComponentOutput> {
        None
    }

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        None
    }
}

pub trait ComponentParams2Simple {
    type ComponentOutput;
    
    fn add_to_ui(self, ui: &mut Ui);

    // this returns an Option mostly just so that we can default impl it with None, but maybe that's useful in other ways?
    // as in, if the component is not currently added, maybe Ui::component_output can just see that and return None, instead of running the function anyway and (hopefully) getting a None?
    // todo: figure this out 
    fn component_output(_ui: &mut Ui) -> Option<Self::ComponentOutput> {
        None
    }

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        None
    }
}

impl<T> ComponentParams2<()> for T
where T: ComponentParams2Simple
 {
    type ComponentOutput = ();

    fn add_to_ui(self, ui: &mut Ui) -> () {
        Self::add_to_ui(self, ui);
    }
}


use std::cell::RefCell;
use bumpalo::Bump;

thread_local! {
    /// Thread local bump arena for temporary allocations
    static THREAD_ARENA: RefCell<Bump> = RefCell::new(Bump::new());
}

/// Access keru's thread-local bump arena for temporary allocations.
/// Useful for small local allocations without passing an arena around, like formatting strings to show in the gui.
///
/// The arena is reset at the end of each frame, when [`Ui::finish_frame()`] is called.
/// 
/// This function is useful when implementing a reusable component with the [`ComponentParams`] trait, since you can't easily access all of your state from within the trait impl. In other cases, it might be more convenient to use your own arena.
///
/// # Panics
/// Panics if [`Ui::finish_frame()`] is called from inside the passes closure.
///
/// # Example
/// ```no_run
/// # use keru::*;
/// # let mut ui: Ui = unimplemented!();
/// # let float_value = 6.7;
/// with_arena(|a| {
///     let text = bumpalo::format!(in a, "{:.2}", float_value);
///     ui.add(LABEL.text(&text)); // Great
///     // ui.finish_frame(); // Don't do this.
/// });
/// 
/// ui.finish_frame(); // Now it's fine.
/// ```
pub fn with_arena<F, R>(f: F) -> R
where
    F: FnOnce(&Bump) -> R,
{
    THREAD_ARENA.with(|arena| {
        f(&arena.borrow())
    })
}

pub(crate) fn reset_arena() {
    THREAD_ARENA.with(|arena| {
        arena.borrow_mut().reset();
    });
}
