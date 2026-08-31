use keru_text::*;
use std::{sync::Arc, time::Duration, error::Error};
use wgpu::*;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::ModifiersState,
    window::{Window, WindowId},
};
use accesskit::{Action, Node, NodeId, Role, Tree, TreeUpdate};
use accesskit_winit::{Adapter, Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};

const WINDOW_TITLE: &str = "Accessibility";

const WINDOW_ID: NodeId = NodeId(0);
const TEXT_EDIT_ID: NodeId = NodeId(1);
const INFO_TEXT_ID: NodeId = NodeId(2);
const MULTILINE_TEXT_ID: NodeId = NodeId(3);

struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: SurfaceConfiguration,
    
    text: Text,
    text_edit_handle: TextEditHandle,
    info_text_handle: TextBoxHandle,
    multiline_text_handle: TextEditHandle,
    
    adapter: Adapter,

    modifiers: ModifiersState,
    
    sent_initial_access_update: bool,
    
}

impl State {
    fn new(
        event_loop: &ActiveEventLoop, 
        event_loop_proxy: EventLoopProxy<AccessKitEvent>
    ) -> Result<Self, Box<dyn Error>> {
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(1200, 800))
            .with_visible(false);

        let window = Arc::new(event_loop.create_window(window_attributes)?);
        let adapter = Adapter::with_event_loop_proxy(event_loop, &window, event_loop_proxy);

        // wgpu boilerplate
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
        let wgpu_adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default())).unwrap();
        let (device, queue) = pollster::block_on(wgpu_adapter.request_device(&DeviceDescriptor::default())).unwrap();
        let surface = instance.create_surface(window.clone()).expect("Create surface");
        let surface_config = surface.get_default_config(&wgpu_adapter, window.inner_size().width, window.inner_size().height).unwrap();
        surface.configure(&device, &surface_config);

        let mut text = Text::new(&device, &queue, surface.format);
        let text_edit_handle = text.add_text_edit("".to_string(), Some((50.0, 100.0)), (300.0, 35.0), 0.0);
        text.set_text_edit_accesskit_id(&text_edit_handle, TEXT_EDIT_ID);
        text.get_text_edit_mut(&text_edit_handle).set_single_line(true);
        text.get_text_edit_mut(&text_edit_handle).set_placeholder_static("Type here");
        
        let info_text_handle = text.add_text_box(
            "This is an accessibility demo. To navigate, try using the platform screen reader's keyboard shortcuts (e.g. Caps Lock + arrow keys on Windows by default).".to_string(),
            Some((50.0, 200.0)), (400.0, 100.0), 0.0
        );
        text.set_text_box_accesskit_id(&info_text_handle, INFO_TEXT_ID);

        let multiline_text_handle = text.add_text_edit("".to_string(), Some((50.0, 450.0)), (500.0, 200.0), 0.0);
        text.set_text_edit_accesskit_id(&multiline_text_handle, MULTILINE_TEXT_ID);
        text.get_text_edit_mut(&multiline_text_handle).set_single_line(false);
        text.get_text_edit_mut(&multiline_text_handle).set_placeholder_static("Multiline text edit");
        
        window.set_visible(true);
        window.set_ime_allowed(true);

        Ok(Self {
            window,
            device,
            queue,
            surface,
            surface_config,
            text,
            text_edit_handle,
            info_text_handle,
            multiline_text_handle,
            adapter,
            modifiers: ModifiersState::default(),
            sent_initial_access_update: false,
        })
    }

    fn build_initial_tree(&mut self) -> TreeUpdate {
        let mut root = Node::new(Role::Window);
        root.set_children(vec![TEXT_EDIT_ID, INFO_TEXT_ID, MULTILINE_TEXT_ID]);
        root.set_label(WINDOW_TITLE);

        let tree = Tree::new(WINDOW_ID);

        // In this example, there's no "gui library", just the text boxes. If there was a gui library, we'd use its "focused" field as a source of truth instead of the one in `text`.
        let focus = self.text.focused_accesskit_id().unwrap_or(WINDOW_ID);
        let mut result = TreeUpdate {
            nodes: vec![
                (WINDOW_ID, root),
            ],
            tree: Some(tree),
            focus,
        };

        self.text.get_text_edit_mut(&self.text_edit_handle).push_accesskit_update(&mut result);
        self.text.get_text_box_mut(&self.info_text_handle).push_accesskit_update(&mut result);
        self.text.get_text_edit_mut(&self.multiline_text_handle).push_accesskit_update(&mut result);

        result
    }

    // If the code was better organized, we could avoid calculating the tree updates when the adapter is inactive.
    // This example doesn't do it for simplicity and because I ran into partial borrow errors when trying. 
    fn send_accesskit_update(&mut self, tree_update: TreeUpdate) {
        if ! self.sent_initial_access_update {
            let initial_tree = self.build_initial_tree();
            self.adapter.update_if_active(|| initial_tree);
            self.sent_initial_access_update = true;
        } else {
            self.adapter.update_if_active(|| tree_update);
        }
    }

    fn set_focus(&mut self, focus: NodeId) {
        self.text.set_focus_by_accesskit_id(focus);

        let tree_update = TreeUpdate {
            nodes: vec![],
            tree: None,
            focus,
        };
        self.send_accesskit_update(tree_update);
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        self.text.handle_event(event, &self.window);
        
        // Because of how accesskit updates work, we always have to specify a value for the current focus, even if we don't want to change it.
        // In this example, there's only text elements, so we ask Text for the current focused element, and then we pass it back to Text, which looks a bit weird.
        // But in general, other GUI elements could have focus. In that case, instead of this line we'd ask the GUI library for the currently focused accesskit NodeId. Then we pass it to Text, so that if it doesn't want to change the focus, it can use that.
        // I don't know why accesskit updates work this way.
        let current_focus = self.text.focused_accesskit_id();
        if let Some((tree_update, _focus_update)) = self.text.accesskit_update(current_focus, WINDOW_ID) {
            self.send_accesskit_update(tree_update);
        }
        

        self.adapter.process_event(&self.window, event);

        match event {
            WindowEvent::Resized(size) => {
                self.surface_config.width = size.width;
                self.surface_config.height = size.height;
                self.surface.configure(&self.device, &self.surface_config);
                self.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let frame = match self.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                    other => panic!("Failed to get current surface texture: {other:?}"),
                };
                let view = frame.texture.create_view(&TextureViewDescriptor::default());

                self.text.prepare_all();
                self.text_renderer.load_to_gpu(&self.device, &self.queue);

                let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });
                {
                    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        ..Default::default()
                    });

                    self.text_renderer.render(&mut pass);
                }

                self.queue.submit(Some(encoder.finish()));
                self.queue.present(frame);

                std::thread::sleep(Duration::from_millis(16));
                self.window.request_redraw();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            _ => {}
        }
    }
}

struct Application {
    event_loop_proxy: EventLoopProxy<AccessKitEvent>,
    state: Option<State>,
}

impl Application {
    fn new(event_loop_proxy: EventLoopProxy<AccessKitEvent>) -> Self {
        Self {
            event_loop_proxy,
            state: None,
        }
    }
}

impl ApplicationHandler<AccessKitEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.state = Some(State::new(event_loop, self.event_loop_proxy.clone()).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Some(state) = &mut self.state {
            state.adapter.process_event(&state.window, &event);
            state.handle_window_event(&event);
            
            if event == WindowEvent::CloseRequested {
                event_loop.exit();
            }
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, user_event: AccessKitEvent) {
        if let Some(state) = &mut self.state {
            match user_event.window_event {
                AccessKitWindowEvent::InitialTreeRequested => {
                    let initial_tree = state.build_initial_tree();
                    state.adapter.update_if_active(|| initial_tree);
                }
                AccessKitWindowEvent::ActionRequested(request) => {
                    let handled = state.text.handle_accessibility_action(&request);
                    
                    // Fallback for Focus action if not handled by the mapping
                    if !handled && request.action == Action::Focus {
                        state.set_focus(request.target);
                    }
                }
                AccessKitWindowEvent::AccessibilityDeactivated => {}
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut app = Application::new(event_loop.create_proxy());
    event_loop.run_app(&mut app).unwrap();
}