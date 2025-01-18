use crate::*;

/// Start a subtree.
/// 
/// Within a subtree, all [`NodeKeys`](NodeKey) are "private": a [`NodeKey`] will never collide with another [`NodeKey`] in a different subtree.
/// 
/// This is the main way to make GUI code reusable, and to create "widgets".
/// 
/// To see why this function is needed, consider this example: if we wrap some GUI code in a function:
/// ```
/// fn my_slider(&mut ui: Ui) {
///     #[node_key] pub const SLIDER_BUTTON: NodeKey;    
///     // some complicated GUI code that uses SLIDER_BUTTON  
/// }
/// ```
/// If we call `my_slider()` in multiple places, it would still be using the same `SLIDER_BUTTON` key every time. This will probably cause things to not work as intended.
/// 
/// To solve this, just wrap the code in a subtree:
/// ```
/// fn my_slider(&mut ui: Ui) -> f32 {
///     subtree(|| {
///         #[node_key] pub const SLIDER_BUTTON: NodeKey;    
///         // some complicated GUI code that uses SLIDER_BUTTON
///     });
/// }
/// ```
/// 
/// Now, we can call the function from multiple places without problems: on every call, `subtree()` will create a distinct subtree. Within each one, the same key refers to a different node identified by both the key and the subtree it's in.
/// 
pub fn subtree<T>(subtree_block: impl FnOnce() -> T) -> T {
    // todo: maybe this should use track caller, or something else?
    // maybe both?
    let subtree_id = Id(thread_local::current_tree_hash());
    
    thread_local::push_subtree(subtree_id);
    
    let result = subtree_block();

    thread_local::pop_subtree();

    return result;
}

/// Like [`subtree()`], but starts a named subtree identified by a [`NodeKey`].
/// 
/// This is usually not needed, but it allows to access a node in a subtree from outside of it:
/// ```
/// fn custom_rendered_widget(&mut ui: Ui, key: NodeKey) {
///     subtree(|| {
///         #[node_key] pub const CUSTOM_RENDERED_NODE: NodeKey;    
///         // some complicated GUI code that uses CUSTOM_RENDERED_NODE
///     });
/// }
/// // Somewhere else in the code, we want to get the custom node's rectangle, so we can run our custom render code
/// // But we can't just do `ui.get_node(CUSTOM_RENDERED_NODE)?.render_rect();`:
/// // That node was defined inside a private subtree, and we are outside of it.
/// // So, we re-enter the _same_ named subtree, using the same key as before::
/// named_subtree(self.key, || {
///     let render_rect = ui.get_node(CUSTOM_RENDERED_NODE)?.render_rect();
///     // use render_rect as we please
/// });
/// ```
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
/// fn custom_container(&mut ui: Ui, content: impl FnOnce()) {
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