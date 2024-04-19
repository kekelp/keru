pub mod ui;
pub use ui::Id;

use bytemuck::{Pod, Zeroable};
use glyphon::{
    Resolution as GlyphonResolution,
};
use ui::{Ui, NodeKey, LayoutMode, Color, floating_window_1, Resolution, Rectangle};
use wgpu::{
    vertex_attr_array,
    CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, LoadOp, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions,
    Surface, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor,
    VertexAttribute,
};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

use std::{marker::PhantomData, mem, sync::Arc};

use crate::ui::NODE_ROOT_ID;



#[rustfmt::skip]
fn main() {
    let (event_loop, mut state) = init();

    event_loop.run(
        move |event, target| {
            state.handle_event(&event, target);
        }
    ).unwrap();
}

pub const WIDTH: u32 = 1200;
pub const HEIGHT: u32 = 800;
pub const SWAPCHAIN_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

fn init() -> (EventLoop<()>, State<'static>) {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
            .with_title("BLUE")
            .build(&event_loop)
            .unwrap(),
    );
    let size = window.inner_size();
    // let scale_factor = window.scale_factor();

    let instance = Instance::new(InstanceDescriptor::default());

    let adapter_options = &RequestAdapterOptions::default();
    let adapter = pollster::block_on(instance.request_adapter(adapter_options)).unwrap();

    let device_desc = &DeviceDescriptor {
        label: None,
        required_features: Features::empty(),
        required_limits: Limits::default(),
    };
    let (device, queue) = pollster::block_on(adapter.request_device(device_desc, None)).unwrap();

    let surface = instance.create_surface(window.clone()).unwrap();

    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: SWAPCHAIN_FORMAT,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    let ui = Ui::new(&device, &config, &queue);

    let state = State {
        window,
        surface,
        config,
        device,
        queue,

        ui,

        // app state
        count: 0,
        counter_mode: true,
    };

    return (event_loop, state);
}

pub struct State<'window> {
    pub window: Arc<Window>,

    pub surface: Surface<'window>,
    pub config: SurfaceConfiguration,
    pub device: Device,
    pub queue: Queue,

    pub ui: Ui,

    pub count: i32,
    pub counter_mode: bool,
}


pub const INCREASE_BUTTON: NodeKey = NodeKey {
    id: id!(),
    static_text: Some(&"Increase"),
    dyn_text: None,
    clickable: true,
    color: Color {
        r: 0.0,
        g: 0.1,
        b: 0.1,
        a: 0.9,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.1,
        end: 0.9,
    },
    layout_y: LayoutMode::Fixed {
        start: 100,
        len: 100,
    },
    is_update: false,
    is_layout_update: false,
};

pub const SHOW_COUNTER_BUTTON: NodeKey = NodeKey {
    id: id!(),
    static_text: Some(&"Show counter"),
    dyn_text: None,
    clickable: true,
    color: Color {
        r: 0.6,
        g: 0.3,
        b: 0.6,
        a: 0.6,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.1,
        end: 0.9,
    },
    layout_y: LayoutMode::Fixed {
        start: 400,
        len: 100,
    },
    is_update: false,
    is_layout_update: false,
};

pub const COUNT_LABEL: NodeKey = NodeKey {
    id: id!(),
    static_text: None,
    dyn_text: None,
    clickable: false,
    color: Color {
        r: 0.1,
        g: 0.3,
        b: 0.9,
        a: 0.6,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.2,
        end: 0.5,
    },
    layout_y: LayoutMode::PercentOfParent {
        start: 0.2,
        end: 0.8,
    },
    is_update: false,
    is_layout_update: false,
};

pub const COLUMN_1: NodeKey = NodeKey {
    id: id!(),
    static_text: None,
    dyn_text: None,
    clickable: true,
    color: Color {
        r: 0.0,
        g: 0.2,
        b: 0.7,
        a: 0.2,
    },
    layout_x: LayoutMode::PercentOfParent {
        start: 0.7,
        end: 0.9,
    },
    layout_y: LayoutMode::PercentOfParent {
        start: 0.0,
        end: 1.0,
    },
    is_update: false,
    is_layout_update: false,
};


pub const FLOATING_WINDOW_1: NodeKey = floating_window_1();


impl<'window> State<'window> {
    pub fn handle_event(&mut self, event: &Event<()>, target: &EventLoopWindowTarget<()>) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(size) => self.resize(size),
                WindowEvent::RedrawRequested => {
                    self.update();

                    self.window.request_redraw();
                }
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::CursorMoved { position, .. } => {
                    self.ui.mouse_pos.x = position.x as f32;
                    self.ui.mouse_pos.y = position.y as f32;
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if *button == MouseButton::Left {
                        if *state == ElementState::Pressed {
                            self.ui.mouse_left_clicked = true;
                            if !self.ui.mouse_left_just_clicked {
                                self.ui.mouse_left_just_clicked = true;
                            }
                        } else {
                            self.ui.mouse_left_clicked = false;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn update(&mut self) {
        self.ui
            .nodes
            .get_mut(&NODE_ROOT_ID)
            .unwrap()
            .children_ids
            .clear();

        let ui = &mut self.ui;

        div!((ui, FLOATING_WINDOW_1) {

            div!(ui, COUNT_LABEL.with_text(self.count));

            div!((ui, COLUMN_1) {

                let text = match self.counter_mode {
                    true => &"Hide counter",
                    false => &"Show counter",
                };
                div!(ui, SHOW_COUNTER_BUTTON.with_text(text));

                if self.counter_mode {
                    let color = Color { r: 0.1 * (self.count as f32), g: 0.0, b: 0.0, a: 1.0 };
                    div!(ui, INCREASE_BUTTON.with_color(color));
                }

            });

        });

        self.layout();
        // self.resolve_input();

        if self.is_clicked(INCREASE_BUTTON) {
            self.count += 1;
        }

        if self.is_clicked(SHOW_COUNTER_BUTTON) {
            self.counter_mode = !self.counter_mode;
        }

        self.build_buffers();
        self.render();

        self.ui.current_frame += 1;
        self.ui.mouse_left_just_clicked = false;
    }

    pub fn render(&mut self) {
        self.ui
            .gpu_vertex_buffer
            .queue_write(&self.ui.rects[..], &self.queue);

        self.ui
            .text_renderer
            .prepare(
                &self.device,
                &self.queue,
                &mut self.ui.font_system,
                &mut self.ui.atlas,
                GlyphonResolution {
                    width: self.config.width,
                    height: self.config.height,
                },
                &mut self.ui.text_areas,
                &mut self.ui.cache,
                self.ui.current_frame,
            )
            .unwrap();

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let n = self.ui.rects.len() as u32;
            if n > 0 {
                pass.set_pipeline(&self.ui.render_pipeline);
                pass.set_bind_group(0, &self.ui.bind_group, &[]);
                pass.set_vertex_buffer(0, self.ui.gpu_vertex_buffer.slice(n));
                pass.draw(0..6, 0..n);
            }

            self.ui
                .text_renderer
                .render(&self.ui.atlas, &mut pass)
                .unwrap();
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        self.ui.atlas.trim();
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);

        let resolution = Resolution {
            width: size.width as f32,
            height: size.height as f32,
        };
        self.queue.write_buffer(
            &self.ui.resolution_buffer,
            0,
            bytemuck::bytes_of(&resolution),
        );

        self.window.request_redraw();
    }

    pub fn build_buffers(&mut self) {
        self.ui.rects.clear();
        self.ui.stack.clear();

        // push the ui.direct children of the root without processing the root
        if let Some(root) = self.ui.nodes.get(&NODE_ROOT_ID) {
            for &child_id in root.children_ids.iter().rev() {
                self.ui.stack.push(child_id);
            }
        }

        while let Some(current_node_id) = self.ui.stack.pop() {
            let current_node = self.ui.nodes.get_mut(&current_node_id).unwrap();

            if current_node.last_frame_touched == self.ui.current_frame {
                self.ui.rects.push(Rectangle {
                    x0: current_node.x0 * 2. - 1.,
                    x1: current_node.x1 * 2. - 1.,
                    y0: current_node.y0 * 2. - 1.,
                    y1: current_node.y1 * 2. - 1.,
                    r: current_node.key.color.r,
                    g: current_node.key.color.g,
                    b: current_node.key.color.b,
                    a: current_node.key.color.a,
                });
            }

            for &child_id in current_node.children_ids.iter() {
                self.ui.stack.push(child_id);
            }
        }
    }

    // todo: deduplicate the traversal with build_buffers, or just merge build_buffers inside here.
    // either way should wait to see how a real layout pass would look like
    pub fn layout(&mut self) {
        self.ui.stack.clear();

        let mut last_rect_xs = (0.0, 1.0);
        let mut last_rect_ys = (0.0, 1.0);

        // push the direct children of the root without processing the root
        if let Some(root) = self.ui.nodes.get(&NODE_ROOT_ID) {
            for &child_id in root.children_ids.iter() {
                self.ui.stack.push(child_id);
            }
        }

        while let Some(current_node_id) = self.ui.stack.pop() {
            let current_node = self.ui.nodes.get_mut(&current_node_id).unwrap();

            let mut new_rect_xs = last_rect_xs;
            let mut new_rect_ys = last_rect_ys;

            match current_node.key.layout_x {
                LayoutMode::PercentOfParent { start, end } => {
                    let len = new_rect_xs.1 - new_rect_xs.0;
                    let x0 = new_rect_xs.0;
                    new_rect_xs = (x0 + len * start, x0 + len * end)
                }
                LayoutMode::ChildrenSum {} => todo!(),
                LayoutMode::Fixed { start, len } => {
                    let x0 = new_rect_xs.0;
                    new_rect_xs = (
                        x0 + (start as f32) / (self.config.width as f32),
                        x0 + ((start + len) as f32) / (self.config.width as f32),
                    )
                }
            }
            match current_node.key.layout_y {
                LayoutMode::PercentOfParent { start, end } => {
                    let len = new_rect_ys.1 - new_rect_ys.0;
                    let y0 = new_rect_ys.0;
                    new_rect_ys = (y0 + len * start, y0 + len * end)
                }
                LayoutMode::Fixed { start, len } => {
                    let y0 = new_rect_ys.0;
                    new_rect_ys = (
                        y0 + (start as f32) / (self.config.height as f32),
                        y0 + ((start + len) as f32) / (self.config.height as f32),
                    )
                }
                LayoutMode::ChildrenSum {} => todo!(),
            }

            current_node.x0 = new_rect_xs.0;
            current_node.x1 = new_rect_xs.1;
            current_node.y0 = new_rect_ys.0;
            current_node.y1 = new_rect_ys.1;

            if let Some(id) = current_node.text_id {
                self.ui.text_areas[id as usize].left = current_node.x0 * (self.config.width as f32);
                self.ui.text_areas[id as usize].top =
                    (1.0 - current_node.y1) * (self.config.height as f32);
                self.ui.text_areas[id as usize].buffer.set_size(
                    &mut self.ui.font_system,
                    100000.,
                    100000.,
                );
                self.ui.text_areas[id as usize]
                    .buffer
                    .shape_until_scroll(&mut self.ui.font_system);
            }

            // do I really need iter.rev() here? why?
            for &child_id in current_node.children_ids.iter().rev() {
                self.ui.stack.push(child_id);

                last_rect_xs.0 = new_rect_xs.0;
                last_rect_xs.1 = new_rect_xs.1;
                last_rect_ys.0 = new_rect_ys.0;
                last_rect_ys.1 = new_rect_ys.1;
            }
        }

        // println!(" {:?}", "  ");

        // print_whole_tree
        // for (k, v) in &self.ui.nodes {
        //     println!(" {:?}: {:#?}", k, v);
        // }

        // println!("self.text_areas.len() {:?}", self.text_areas.len());
        // println!("self.ui.rects.len() {:?}", self.ui.rects.len());
    }

    // in the future, do the full tree pass (for covered stuff etc)
    pub fn is_clicked(&self, button: NodeKey) -> bool {
        if !self.ui.mouse_left_just_clicked {
            return false;
        }

        let node = self.ui.nodes.get(&button.id);
        if let Some(node) = node {
            if node.last_frame_touched != self.ui.current_frame {
                return false;
            }

            let mouse_pos = (
                self.ui.mouse_pos.x / (self.config.width as f32),
                1.0 - (self.ui.mouse_pos.y / (self.config.height as f32)),
            );
            if node.x0 < mouse_pos.0 && mouse_pos.0 < node.x1 {
                if node.y0 < mouse_pos.1 && mouse_pos.1 < node.y1 {
                    return true;
                }
            }
        }

        return false;
    }
}

// these have to be macros only because of the deferred pop().
// todo: pass "ui" or something instead of self.

#[macro_export]
macro_rules! div {
    // non-leaf, has to manage the stack and pop() after the code
    (($ui:expr, $node_key:expr) $code:block) => {
        $ui.div($node_key);

        $ui.parent_stack.push($node_key.id);
        $code;
        $ui.parent_stack.pop();
    };
    // leaf. doesn't need to touch the stack. doesn't actually need to be a macro except for symmetry.
    ($ui:expr, $node_key:expr) => {
        $ui.div($node_key);
    };
}

