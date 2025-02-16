use crate::*;

pub struct UiSubtree {
    key: SubtreeKey,
}

impl Ui {
    /// Create a subtree. 
    /// 
    /// To start the subtree and run Ui code inside it, use [`UiSubtree::start()`].
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
    ///     // some complicated GUI code that uses WIDGET_NODE  
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
    pub fn subtree(&mut self) -> UiSubtree {
        let subtree_id = Id(thread_local::current_tree_hash());
        let key = SubtreeKey::new(subtree_id, "Anon subtree");
        return UiSubtree {
            key,
        };
    }


    /// Like [`Uisubtree()`], but starts a named subtree identified by a [`NodeKey`].
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
    /// // But we can't just do `ui.get_node(CUSTOM_RENDERED_NODE)?.render_rect();`:
    /// // That node was defined inside a private subtree, and we are outside of it.
    /// // So, we re-enter the _same_ named subtree, using the same key as before:
    /// # #[node_key] const CUSTOM_RENDERED_NODE: NodeKey;
    /// # #[node_key] const WIDGET_KEY_1: NodeKey; 
    /// # fn test_fn2(ui: &mut Ui) -> Option<()> {
    /// ui.named_subtree(WIDGET_KEY_1).start(|| {
    ///     let render_rect = ui.get_node(CUSTOM_RENDERED_NODE)?.render_rect();
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
}

/// A struct referring to a subtree created with [`Ui::subtree()`] or [`Ui::named_subtree()`].
///  
/// Use [`start()`](Self::start()) to start the subtree. 
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