use crate::*;

/// Start a subtree.
/// 
/// Within a subtree, all [`NodeKeys`](NodeKey) are "private": a [`NodeKey`] will never collide with another [`NodeKey`] in a different subtree.
/// 
/// This is the main way to make GUI code reusable, and to create "widgets".
/// 
/// To see why this function is needed, consider this example: if we wrap some GUI code in a function:
/// ```
/// # use keru::*;
/// fn widget(ui: &mut Ui) {
///     #[node_key] pub const WIDGET_NODE: NodeKey;    
///     // some complicated GUI code that uses WIDGET_NODE  
/// }
/// ```
/// If we call `widget()` in multiple places, it would still be using the same `WIDGET_NODE` key every time. This will probably cause things to not work as intended.
/// 
/// To solve this, just wrap the code in a subtree:
/// ```
/// # use keru::*;
/// fn widget(ui: &mut Ui) {
///     subtree(|| {
///         #[node_key] pub const WIDGET_NODE: NodeKey;    
///         // some complicated GUI code that uses WIDGET_NODE
///     });
/// }
/// ```
/// 
/// Now, we can call the function from multiple places without problems: on every call, `subtree()` will create a distinct subtree. Within each one, the same key refers to a different node identified by both the key and the subtree it's in.
/// 
/// 
pub fn subtree<T>(subtree_block: impl FnOnce() -> T) -> T {
    // todo: maybe this should use track caller, or something else?
    // maybe both?
    let subtree_id = Id(thread_local::current_tree_hash());
    
    thread_local::push_subtree(subtree_id);
    
    let block_result = subtree_block();

    thread_local::pop_subtree();

    return block_result;
}

/// Like [`subtree()`], but starts a named subtree identified by a [`NodeKey`].
/// 
/// This is usually not needed, but it allows to access a node in a subtree from outside of it:
/// ```
/// # use keru::*;
/// fn custom_rendered_widget(ui: &mut Ui, key: NodeKey) {
///     named_subtree(key, || {
///         #[node_key] pub const CUSTOM_RENDERED_NODE: NodeKey;    
///         // some complicated GUI code that uses CUSTOM_RENDERED_NODE
///     });
/// }
/// 
/// # fn test_fn(ui: &mut Ui) {
/// #[node_key] pub const WIDGET_KEY_1: NodeKey; 
/// custom_rendered_widget(ui, WIDGET_KEY_1);
/// # }
/// 
/// // Somewhere else in the code, we want to get the custom node's rectangle, 
/// //   so we can run our custom render code.
/// // But we can't just do `ui.get_node(CUSTOM_RENDERED_NODE)?.render_rect();`:
/// // That node was defined inside a private subtree, and we are outside of it.
/// // So, we re-enter the _same_ named subtree, using the same key as before:
/// # #[node_key] pub const CUSTOM_RENDERED_NODE: NodeKey;
/// # #[node_key] pub const WIDGET_KEY_1: NodeKey; 
/// # fn test_fn2(ui: &mut Ui) -> Option<()> {
/// named_subtree(WIDGET_KEY_1, || {
///     let render_rect = ui.get_node(CUSTOM_RENDERED_NODE)?.render_rect();
///     // ... render the custom widget
///     # return Some(());
/// })
/// # }
/// ```
/// 
/// Usually there are other ways to accomplish the same thing without using named subtrees. In the example, we could have got the render rect when still inside the first subtree, returned it from the function, and passed it to the render code.
/// 
pub fn named_subtree<T>(key: NodeKey, subtree_block: impl FnOnce() -> T) -> T {
    let subtree_id = key.id_with_subtree();
    
    thread_local::push_subtree(subtree_id);
    
    let result = subtree_block();

    thread_local::pop_subtree();

    return result;
}

/// Temporarily exit a subtree, and then re-renter it.
/// 
/// This is useful when creating "container widgets" that take a closure for the content in the same way [`nest()`](UiPlacedNode::nest()) does.
/// 
/// ```
/// # use keru::*;
/// fn custom_container(ui: &mut Ui, content: impl FnOnce()) {
///     // start a subtree
///     subtree(|| {
///         // build a fancy border or something
///         exit_subtree(|| {
///             // run the content code provided from outside
///             content();
///         });
///         // re-enter the subtree and build some more border elements
///     });
/// }
/// ```
/// 
/// I haven't really tried this out yet.
pub fn exit_subtree(out_of_subtree_block: impl FnOnce()) {       
    if let Some(last_subtree_id) = thread_local::last_subtree() {
        thread_local::pop_subtree();
    
        out_of_subtree_block();
        
        thread_local::push_subtree(last_subtree_id);

    } else {
        log::error!("exit_subtree, was called, but no subtree was ever entered!");
        out_of_subtree_block();
    };
}