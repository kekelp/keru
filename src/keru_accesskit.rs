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

pub(crate) use accesskit_winit::{Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};

/// Pushes each AccessKit event onto the shared queue, then requests a redraw to wake the event loop. One instance implements all three handler traits, per role, each with its own sender clone.
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

pub(crate) struct AccessKitAdapter {
    adapter: Adapter,
    event_receiver: Receiver<AccessKitEvent>,
    window: Weak<Window>,
}

impl AccessKitAdapter {
    pub(crate) fn new(event_loop: &ActiveEventLoop, window: Arc<Window>) -> Self {
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

    pub(crate) fn poll_events(&mut self) -> impl Iterator<Item = AccessKitEvent> + '_ {
        self.event_receiver.try_iter()
    }

    pub(crate) fn process_event(&mut self, event: &WinitWindowEvent) {
        if let Some(window) = self.window.upgrade() {
            self.adapter.process_event(&window, event);
        }
    }

    pub(crate) fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        self.adapter.update_if_active(updater);
    }
}

pub(crate) const WINDOW_NODE_ID: NodeId = NodeId(0);
