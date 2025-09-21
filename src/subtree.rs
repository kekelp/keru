use crate::*;

/// A struct referring to a subtree created with [`Ui::subtree()`] or [`Ui::named_subtree()`].
///  
/// Use [`start()`](Self::start()) to start the subtree. 
pub struct UiSubtree {
    key: SubtreeKey,
}

impl Ui {
    /// Create a subtree. 
    /// 
    /// To start the subtree and run Ui code inside it, use [`UiSubtree::start()`].
    /// 
    /// ```no_run
    /// # use keru::*;
    /// fn component(ui: &mut Ui) {
    ///     ui.subtree().start(|| {
    ///         // define private keys and use them
    ///     });
    /// }
    /// ```
    /// 
    /// Within a subtree, all [`NodeKeys`](NodeKey) are "private": a [`NodeKey`] will never collide with another [`NodeKey`] in a different subtree.
    /// 
    /// This is the main way to make Ui code reusable, and to create "components".
    /// 
    /// To see why this function is needed, consider this example: if we wrap some Ui code in a function:
    /// ```
    /// # use keru::*;
    /// fn widget(ui: &mut Ui) {
    ///     #[node_key] const WIDGET_NODE: NodeKey;    
    ///     // some complicated Ui code that uses WIDGET_NODE  
    /// }
    /// ```
    /// If we call `widget()` in multiple places, it would still be using the same `WIDGET_NODE` key every time. This will probably cause things to not work as intended.
    /// 
    /// To solve this, just wrap the code in a subtree:
    /// ```
    /// # use keru::*;
    /// fn widget(ui: &mut Ui) {
    ///     ui.subtree().start(|| {
    ///         #[node_key] const WIDGET_NODE: NodeKey;    
    ///         // some complicated GUI code that uses WIDGET_NODE
    ///     });
    /// }
    /// ```
    /// 
    /// Now, we can call the function from multiple places without problems: on every call, `Uisubtree()` will create a distinct subtree. Within each one, the same key refers to a different node identified by both the key and the subtree it's in.
    /// 
    // /// 
    pub fn subtree_old(&mut self) -> UiSubtree {
        let subtree_id = Id(self.current_tree_hash());
        let key = SubtreeKey::new(subtree_id, "Anon subtree");
        return UiSubtree {
            key,
        };
    }

    /// Like [`Ui::subtree()`], but starts a named subtree identified by a [`NodeKey`].
    /// 
    /// This is usually not needed, but it allows to access a node in a subtree from outside of it:
    /// ```
    /// # use keru::*;
    /// fn custom_rendered_widget(ui: &mut Ui, key: NodeKey) {
    /// ui.named_subtree(key).start(|| {
    ///         #[node_key] const CUSTOM_RENDERED_NODE: NodeKey;    
    ///         // some complicated GUI code that uses CUSTOM_RENDERED_NODE
    ///     });
    /// }
    /// 
    /// # fn test_fn(ui: &mut Ui) {
    /// #[node_key] const WIDGET_KEY_1: NodeKey; 
    /// custom_rendered_widget(ui, WIDGET_KEY_1);
    /// # }
    /// 
    /// // Somewhere else in the code, we want to get the custom node's rectangle, 
    /// //   so we can run our custom render code.
    /// // But we can't just do `ui.render_rect(CUSTOM_RENDERED_NODE);`:
    /// // That node was defined inside a private subtree, and we are outside of it.
    /// // So, we re-enter the _same_ named subtree, using the same key as before:
    /// # #[node_key] const CUSTOM_RENDERED_NODE: NodeKey;
    /// # #[node_key] const WIDGET_KEY_1: NodeKey; 
    /// # fn test_fn2(ui: &mut Ui) -> Option<()> {
    /// ui.named_subtree(WIDGET_KEY_1).start(|| {
    ///     let render_rect = ui.render_rect(CUSTOM_RENDERED_NODE)?;
    ///     // ... render the custom widget
    ///     # return Some(());
    /// })
    /// # }
    /// ```
    /// 
    /// Usually there are other ways to accomplish the same thing without using named subtrees. In the example, we could have got the render rect when still inside the first subtree, returned it from the function, and passed it to the render code.
    pub fn named_subtree(&mut self, key: SubtreeKey) -> UiSubtree {
        return UiSubtree {
            key,
        };
    }

    pub(crate) fn component_key_subtree<C>(&mut self, key: ComponentKey<C>) -> UiSubtree {
        return UiSubtree {
            key: key.as_normal_key(),
        };
    }

    /// Starts a subtree where keys can be used without conflicts with the outside world, like in components. 
    /// 
    /// It can be used to create reusable "components" more concisely than with the [`ComponentParams`] trait.
    /// 
    /// This function must be called from functions marked with `#[track_caller]`!  
    #[track_caller]
    pub fn subtree(&mut self) -> UiSubtree {
        let key = NodeKey::new(Id(caller_location_id()), "");
        return UiSubtree { key };
    }
}

impl UiSubtree {
    /// Start a subtree created with [`Ui::subtree()`] or [`Ui::named_subtree()`].
    pub fn start<T>(&mut self, subtree_content: impl FnOnce() -> T) -> T {
        let subtree_id = self.key.id_with_subtree();
        
        thread_local::push_subtree(subtree_id);
        
        let result = subtree_content();

        thread_local::pop_subtree();

        return result;
    }
}