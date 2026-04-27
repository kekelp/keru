use crate::*;

pub(crate) type KeyScopeKey = NodeKey;

/// A struct referring to a key scope created with [`Ui::key_scope()`] or [`Ui::named_key_scope()`].
///  
/// Use [`start()`](Self::start()) to start the key scope. 
pub struct UiKeyScope {
    pub(crate) key: KeyScopeKey,
}

impl Ui {
    /// Create a key scope. 
    /// 
    /// To start the key scope and run Ui code inside it, use [`UiKeyScope::start()`].
    /// 
    /// ```no_run
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
    /// fn component(ui: &mut Ui) {
    ///     ui.key_scope().start(|| {
    ///         // define private keys and use them
    ///         #[node_key] const INTERNAL_BUTTON: NodeKey;
    ///         if ui.is_clicked(INTERNAL_BUTTON) {
    ///             // ...
    ///         }
    ///     });
    /// }
    /// ```
    /// 
    /// Within a key scope, all [`NodeKey`](NodeKey) are "private": a [`NodeKey`] will never collide with another [`NodeKey`] in a different key scope.
    /// 
    /// [`NodeKey`]s are unique identifiers for GUI nodes.
    /// If we call the `component()` function above multiple times without a key scope, the `INTERNAL_BUTTON` key would be the same key every time, 
    /// so we can't expect it to point to the different nodes that we end up adding. 
    /// 
    /// But If we use a key scope, it will work automatically, and we can call the function from multiple places without problems. 
    /// On every separate call, `ui.key_scope()` will create a separate scope, and the same key will refer to a different node identified by both the key and the key scope it's in.
    /// 
    /// See the [`Component`] trait for a more robust way to create reusable components. `Component` creates a private key scope for each instance automatically.
    /// In addition, `Component`s can manage their own state.
    pub fn key_scope(&mut self) -> UiKeyScope {
        let key_scope_id = Id(self.current_tree_hash());
        let key = KeyScopeKey::new(key_scope_id, "");
        return UiKeyScope {
            key,
        };
    }

    /// Like [`Ui::key_scope()`], but starts a named key scope identified by a [`NodeKey`].
    /// 
    /// This is usually not needed, but it allows to access a node in a key scope from outside of it:
    /// ```no_run
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
    /// # #[node_key] const CUSTOM_RENDERED_NODE: NodeKey;
    /// # #[node_key] const WIDGET_KEY_1: NodeKey;
    /// #
    /// ui.named_key_scope(WIDGET_KEY_1).start(|| {
    ///     // some complicated GUI code that uses CUSTOM_RENDERED_NODE
    /// });
    /// ```
    /// 
    /// Somewhere else in the code, we want to get the custom node's rectangle,
    ///   so we can run our custom render code.
    /// 
    /// But we can't just do `ui.render_rect(CUSTOM_RENDERED_NODE);`:
    /// That node was defined inside a private key scope, and we are outside of it.
    /// So, we re-enter the _same_ named key scope using the same key as before:
    /// 
    /// ```no_run
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
    /// # #[node_key] const CUSTOM_RENDERED_NODE: NodeKey;
    /// # #[node_key] const WIDGET_KEY_1: NodeKey;
    /// #
    /// ui.named_key_scope(WIDGET_KEY_1).start(|| {
    ///     let render_rect = ui.get_node(CUSTOM_RENDERED_NODE).unwrap().render_rect();
    ///     // ... render the custom widget
    /// });
    /// ```
    /// 
    /// Usually there are other ways to accomplish the same thing without using named key scopes. In the example, we could have got the render rect when still inside the first key scope, returned it from the function or stored it somewhere, and passed it to the render code.
    pub fn named_key_scope(&mut self, key: KeyScopeKey) -> UiKeyScope {
        return UiKeyScope {
            key,
        };
    }

    pub(crate) fn component_key_scope<C>(&mut self, key: ComponentKey<C>) -> UiKeyScope {
        return UiKeyScope {
            key: key.as_normal_key(),
        };
    }
}

impl UiKeyScope {
    /// Start the key scope.
    pub fn start<T>(&mut self, content: impl FnOnce() -> T) -> T {
        let key_scope_id = self.key.id_with_key_scope();
        
        thread_local::push_key_scope(key_scope_id);
        
        let result = content();

        thread_local::pop_key_scope();

        return result;
    }
}