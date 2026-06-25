//! A queue-based wrapper around [`accesskit_winit::Adapter`].

use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Weak,
};

use accesskit::{
    ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, NodeId, TreeUpdate,
};
use accesskit_winit::{Adapter};
use winit::{
    event::WindowEvent as WinitWindowEvent,
    event_loop::ActiveEventLoop,
    window::Window,
};

pub use accesskit_winit::{Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};

/// One of the three AccessKit handlers. Each call pushes an event onto the
/// shared queue, then wakes the event loop by requesting a redraw of the window.
/// A single type implements all three handler traits; one instance is created
/// per role, each holding its own clone of the sender.
struct QueueingHandler {
    window: Weak<Window>,
    sender: Sender<AccessKitEvent>,
}

impl QueueingHandler {
    fn push(&self, window_event: AccessKitWindowEvent) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let event = AccessKitEvent {
            window_id: window.id(),
            window_event,
        };
        self.sender.send(event).ok();
        window.request_redraw();
    }
}

impl ActivationHandler for QueueingHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        self.push(AccessKitWindowEvent::InitialTreeRequested);
        None
    }
}

impl ActionHandler for QueueingHandler {
    fn do_action(&mut self, request: ActionRequest) {
        self.push(AccessKitWindowEvent::ActionRequested(request));
    }
}

impl DeactivationHandler for QueueingHandler {
    fn deactivate_accessibility(&mut self) {
        self.push(AccessKitWindowEvent::AccessibilityDeactivated);
    }
}

/// A wrapper around [`accesskit_winit::Adapter`] that delivers AccessKit events
/// as plain data through an internal queue instead of through a winit event
/// loop proxy or direct callbacks.
pub struct AccessKitAdapter {
    adapter: Adapter,
    event_receiver: Receiver<AccessKitEvent>,
    // The adapter keeps its own weak handle to the window so callers don't have
    // to pass it back in on every event.
    window: Weak<Window>,
}

impl AccessKitAdapter {
    /// Creates a new adapter for a winit window. As with the underlying
    /// `accesskit_winit` adapter, this must be done before the window is shown
    /// for the first time, so create the window with
    /// [`winit::window::WindowAttributes::with_visible`] set to `false`, then
    /// create the adapter, then show the window.
    ///
    /// Because events are queued and handled later on the main thread, the
    /// adapter cannot return an initial tree synchronously; some platform
    /// adapters will use a placeholder tree until you send the first update via
    /// [`AccessKitAdapter::update_if_active`].
    pub fn new(event_loop: &ActiveEventLoop, window: Arc<Window>) -> Self {
        let (sender, event_receiver) = channel();
        let make_handler = || QueueingHandler {
            window: Arc::downgrade(&window),
            sender: sender.clone(),
        };
        let adapter = Adapter::with_direct_handlers(
            event_loop,
            &window,
            make_handler(),
            make_handler(),
            make_handler(),
        );
        let window = Arc::downgrade(&window);
        Self {
            adapter,
            event_receiver,
            window,
        }
    }

    /// Returns an iterator over the AccessKit events queued since the last call.
    /// Call this whenever the window is woken for a redraw.
    pub fn poll_events(&mut self) -> impl Iterator<Item = AccessKitEvent> + '_ {
        self.event_receiver.try_iter()
    }

    /// Forwards a winit window event to the underlying adapter. Call this for
    /// every window event, before your application handles it. Does nothing if
    /// the window has been dropped.
    pub fn process_event(&mut self, event: &WinitWindowEvent) {
        if let Some(window) = self.window.upgrade() {
            self.adapter.process_event(&window, event);
        }
    }

    /// If and only if the tree has been initialized, calls the provided function
    /// and applies the resulting update. Because this adapter never returns an
    /// initial tree synchronously, the first [`TreeUpdate`] you supply must
    /// contain a full tree.
    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
    }
}

/// The AccessKit id of the window root node, which mirrors the Keru root node
/// (whose [`Id`] inner value is `0`). Every other AccessKit node reuses its
/// Keru [`Id`] directly. The tree itself is built in [`crate::accessibility`].
pub const WINDOW_NODE_ID: NodeId = NodeId(0);
