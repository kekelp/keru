pub mod shapes;
pub use shapes::*;
pub mod color;
pub use color::*;
pub mod gpu_vec;
pub mod gpu_slab;
pub mod images;

mod limited_hangout;
pub use limited_hangout::*;

use gpu_vec::GpuVec;
use gpu_slab::{GpuSlab, GpuSlabItem};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy)]
pub struct InstanceRange { pub start: usize, pub end: usize }

/// A range of instances in the deferred buffer.
/// Returned by `end_deferred_mode()` and used with `draw_deferred_elements()`.
#[derive(Debug, Clone, Copy)]
pub struct DeferredInstanceRange { start: usize, end: usize }

/// A handle to a transform inside the [`Renderer`]'s transforms slab.
///
/// The transform should eventually be removed with [`Renderer::remove_transform()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformHandle {
    index: usize,
    text_transform: keru_text::GroupTransformHandle,
}

impl TransformHandle {
    /// A handle to the always-present identity transform.
    pub const IDENTITY: Self = Self {
        index: 0,
        text_transform: keru_text::GroupTransformHandle::IDENTITY,
    };
}


/// A handle to a clip rect inside the [`Renderer`]'s clip rects slab.
/// 
/// The clip rect should eventually be removed with [`Renderer::remove_clip rect()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipRectHandle(usize);

impl ClipRectHandle {
    /// A handle to the always-present "no clip" rect.
    pub const NO_CLIP: Self = Self(1);
}

pub use keru_text;

pub use keru_text::{
    Text, TextBoxHandle, TextEditHandle,
    TextStyle2, StyleHandle, ColorBrush, with_clipboard, BoundingBox,
    parley,
};
// Re-export font properties from parley
pub use keru_text::parley::{FontWeight, FontStyle, LineHeight};
pub use images::{ImageRenderer, LoadedImage};

pub use euclid;

pub mod primitive {
    pub const BOX: u32 = 0;
    pub const CIRCLE: u32 = 1;
    pub const SEGMENT: u32 = 2;
    pub const TEXT: u32 = 3;
    pub const GRID: u32 = 4;
    pub const TRIANGLE: u32 = 5;
    pub const HEXAGON: u32 = 6;
    pub const QUADRATIC_BEZIER: u32 = 7;
}

bitflags::bitflags! {
    /// Bitflags specifying which corners of a rectangle should be rounded.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct RoundedCorners: u32 {
        const TOP_LEFT = 1 << 0;
        const TOP_RIGHT = 1 << 1;
        const BOTTOM_LEFT = 1 << 2;
        const BOTTOM_RIGHT = 1 << 3;
        const ALL = Self::TOP_LEFT.bits() | Self::TOP_RIGHT.bits() | Self::BOTTOM_LEFT.bits() | Self::BOTTOM_RIGHT.bits();
        const NONE = 0;
        const TOP = Self::TOP_LEFT.bits() | Self::TOP_RIGHT.bits();
        const BOTTOM = Self::BOTTOM_LEFT.bits() | Self::BOTTOM_RIGHT.bits();
        const LEFT = Self::TOP_LEFT.bits() | Self::BOTTOM_LEFT.bits();
        const RIGHT = Self::TOP_RIGHT.bits() | Self::BOTTOM_RIGHT.bits();
    }
}


/// Pixel insets from each edge of a source image that define the 9 slice regions.
/// Each value is a distance in source image pixels from the respective edge.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NineSliceMargins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl NineSliceMargins {
    pub const ZERO: Self = Self { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 };

    pub const fn uniform(v: f32) -> Self {
        Self { top: v, right: v, bottom: v, left: v }
    }

    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }
}

impl std::hash::Hash for NineSliceMargins {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.top.to_bits().hash(state);
        self.right.to_bits().hash(state);
        self.bottom.to_bits().hash(state);
        self.left.to_bits().hash(state);
    }
}

/// How a non-corner region of a bordered texture is filled along one axis.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TileMode {
    /// Stretch the region to fill the available space.
    #[default]
    Stretch = 0,
    /// Repeat the region at its natural size.
    Tile = 1,
    /// Repeat the region, scaled so a whole number of copies fit exactly.
    TileFit = 2,
}

/// Texture sampling options: border insets for 9-slice scaling and per-axis tiling modes.
///
/// Used alongside `texture: Option<LoadedImage>` on shape structs.
/// Nine-slice margins default to `None` (no slicing) and tiling defaults to `Stretch`.
#[derive(Debug, Clone, Copy, Default)]
pub struct TextureOptions {
    /// Nine-slice margins.
    pub nine_slice: Option<NineSliceMargins>,
    /// Horizontal tile/stretch mode.
    ///
    /// If `nine_slice` is not `None`, it will not apply to the corner regions
    pub tile_x: TileMode,
    /// Vertical tile/stretch mode.
    ///
    /// If `nine_slice` is not `None`, it will not apply to the corner regions
    pub tile_y: TileMode,
}
impl TextureOptions {
    pub const DEFAULT: TextureOptions = TextureOptions {
        nine_slice: Some(NineSliceMargins {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }),
        tile_x: TileMode::Stretch,
        tile_y: TileMode::Stretch,
    };
}

/// Parameters for drawing a box/rectangle
#[derive(Debug, Clone)]
pub struct Rectangle {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub corner_radius: f32,
    pub rounded_corners: RoundedCorners,
    pub border_thickness: f32,
    pub fill: ColorFill,
    pub texture: Option<LoadedImage>,
    pub texture_options: Option<TextureOptions>,
    pub blur: f32,
}

/// Parameters for drawing a circle
#[derive(Debug, Clone)]
pub struct Circle {
    pub center: [f32; 2],
    pub radius: f32,
    pub fill: ColorFill,
    pub texture: Option<LoadedImage>,
    pub texture_options: Option<TextureOptions>,
    pub blur: f32,
}

/// Parameters for drawing a ring (hollow circle)
#[derive(Debug, Clone)]
pub struct CircleRing {
    pub center: [f32; 2],
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub fill: ColorFill,
    pub texture: Option<LoadedImage>,
    pub texture_options: Option<TextureOptions>,
    pub dash_length: Option<f32>,
    pub dash_offset: f32,
    pub blur: f32,
}

/// Parameters for drawing an arc
#[derive(Debug, Clone)]
pub struct CircleArc {
    pub center: [f32; 2],
    pub radius: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub thickness: f32,
    pub fill: ColorFill,
    pub texture: Option<LoadedImage>,
    pub texture_options: Option<TextureOptions>,
    pub dash_length: Option<f32>,
    pub dash_offset: f32,
    pub blur: f32,
}

/// Parameters for drawing a pie slice
#[derive(Debug, Clone)]
pub struct CirclePie {
    pub center: [f32; 2],
    pub radius: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub fill: ColorFill,
    pub texture: Option<LoadedImage>,
    pub texture_options: Option<TextureOptions>,
    pub blur: f32,
    pub stroke_thickness: f32,  // 0 = filled, >0 = hollow wedge outline
    pub corner_radius: f32,     // rounds the center point and arc-edge corners
}

/// Parameters for drawing a line segment
#[derive(Debug, Clone)]
pub struct Segment {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub thickness: f32,
    pub stroke_thickness: f32,  // 0 = filled capsule, >0 = hollow capsule outline
    pub fill: ColorFill,
    pub dash_length: Option<f32>,
    pub dash_offset: f32,
    pub texture: Option<LoadedImage>,
    pub texture_options: Option<TextureOptions>,
    pub blur: f32,
}

/// Grid type for the grid primitive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridType {
    Square = 0,
    Hexagonal = 1,
}

/// Parameters for drawing a grid
#[derive(Debug, Clone)]
pub struct Grid {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    /// Cell spacing in pixels. Separate x and y for rectangular grids.
    /// Hexagonal grids only use the x component.
    pub lattice_size: [f32; 2],
    pub offset: [f32; 2],
    pub line_thickness: f32,
    pub fill: ColorFill,
    pub grid_type: GridType,
    pub texture: Option<LoadedImage>,
    pub texture_options: Option<TextureOptions>,
    pub blur: f32,
}

/// Parameters for drawing a triangle
#[derive(Debug, Clone)]
pub struct Triangle {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub fill: ColorFill,
    pub stroke_thickness: f32,  // 0 = filled, >0 = stroke only
    pub corner_radius: f32,
    pub texture: Option<LoadedImage>,
    pub texture_options: Option<TextureOptions>,
    pub blur: f32,
}

/// Parameters for drawing a hexagon
#[derive(Debug, Clone)]
pub struct Hexagon {
    pub center: [f32; 2],
    pub size: f32,              // distance from center to vertex
    pub rotation: f32,          // rotation in radians (0 = flat-top)
    pub fill: ColorFill,
    pub stroke_thickness: f32,  // 0 = filled, >0 = stroke only
    pub corner_radius: f32,
    pub texture: Option<LoadedImage>,
    pub texture_options: Option<TextureOptions>,
    pub blur: f32,
}

/// Parameters for drawing a quadratic bezier curve from `p0` to `p2`, with `p1` as a control point.
///
/// The curve is rendered analytically, solving the cubic distance equation in the fragment shader. This is not cheap, relatively speaking.
#[derive(Debug, Clone)]
pub struct QuadraticBezier {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub thickness: f32,
    pub stroke_thickness: f32,  // 0 = filled tube, >0 = hollow tube outline
    pub color: Color,
    pub blur: f32,
}

/// Parameters for drawing a dashed box outline (composed of segments and corner arcs)
#[derive(Debug, Clone)]
pub struct DashedBoxOutline {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub corner_radius: f32,
    pub thickness: f32,
    pub color: Color,
    pub dash_length: f32,
    pub blur: f32,
}

/// Parameters for drawing a dashed hexagon outline (composed of segments)
#[derive(Debug, Clone)]
pub struct DashedHexagonOutline {
    pub center: [f32; 2],
    pub size: f32,              // distance from center to vertex
    pub rotation: f32,          // rotation in radians (0 = flat-top)
    pub thickness: f32,
    pub color: Color,
    pub dash_length: f32,
    pub blur: f32,
}

fn push_gradient(resources: &mut GpuSlab<ResourceSlot>, gradient_indices: &mut Vec<usize>, gradient: shapes::GradientGpu) -> u32 {
    let index = resources.insert(gradient.into());
    gradient_indices.push(index);
    index as u32
}

fn gradient_index_for_fill(resources: &mut GpuSlab<ResourceSlot>, gradient_indices: &mut Vec<usize>, fill: ColorFill) -> u32 {
    match fill {
        ColorFill::Color(color) => push_gradient(resources, gradient_indices, shapes::GradientGpu::solid(color)),
        ColorFill::Gradient(gradient) => push_gradient(resources, gradient_indices, gradient.to_gpu()),
        ColorFill::SharedGradient(handle) => handle.0,
    }
}

// Returns (uv_origin, uv_size, page, ns_l, ns_r, ns_t, ns_b, tiling_flags)
fn texture_options_gpu(texture: Option<LoadedImage>, opts: Option<TextureOptions>) -> ([f32; 2], [f32; 2], u32, f32, f32, f32, f32, u32) {
    match texture {
        None => ([0.0, 0.0], [0.0, 0.0], u32::MAX, 0.0, 0.0, 0.0, 0.0, 0),
        Some(image) => {
            let uv_origin = [image.alloc.rectangle.min.x as f32, image.alloc.rectangle.min.y as f32];
            let uv_size = [image.width as f32, image.height as f32];
            let page = image.page as u32;
            let opts = opts.unwrap_or_default();
            let has_insets = opts.nine_slice.is_some();
            let has_tiling = opts.tile_x != TileMode::Stretch || opts.tile_y != TileMode::Stretch;
            let enabled = (has_insets || has_tiling) as u32;
            let flags: u32 = enabled | ((opts.tile_x as u32) << 1) | ((opts.tile_y as u32) << 3);
            let i = opts.nine_slice.unwrap_or_default();
            (uv_origin, uv_size, page, i.left, i.right, i.top, i.bottom, flags)
        }
    }
}

/// A screen-space rectangle in pixel coordinates.
/// Screen space has (0, 0) at the top-left corner, with Y increasing downward.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    /// Minimum X coordinate in pixels (left edge)
    pub min_x: f32,
    /// Minimum Y coordinate in pixels (top edge)
    pub min_y: f32,
    /// Maximum X coordinate in pixels (right edge)
    pub max_x: f32,
    /// Maximum Y coordinate in pixels (bottom edge)
    pub max_y: f32,
}

/// A simple 2D transform with uniform scale and offset
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform {
    pub offset: [f32; 2],
    pub scale: f32,
    pub _padding: f32,  // For 16-byte alignment
}

/// A clip rect
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ClipRect {
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
}
impl ClipRect {
    pub const NO_CLIPPING: ClipRect = ClipRect {
        x_clip: [f32::MIN, f32::MAX],
        y_clip: [f32::MIN, f32::MAX],
    };
}


impl Transform {
    /// Create an identity transform (no translation, scale = 1.0)
    pub fn identity() -> Self {
        Self {
            offset: [0.0, 0.0],
            scale: 1.0,
            _padding: 0.0,
        }
    }

    /// Create a translation transform
    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            offset: [x, y],
            scale: 1.0,
            _padding: 0.0,
        }
    }

    /// Create a scale transform centered at origin
    pub fn scale(scale: f32) -> Self {
        Self {
            offset: [0.0, 0.0],
            scale,
            _padding: 0.0,
        }
    }
}

// 48-byte raw slot that can hold a Transform, ClipRect, or GradientGpu.
// All this crap is needed because of the absolutely insane limit `max_storage_buffers_per_shader_stage: 8`.
// No actual GPU has such a limit, but if we don't obey, keru_draw and keru would crash on everyone's wgpu loop until they request the higher limit manually.
// https://github.com/gpuweb/gpuweb/issues/4235
// https://vulkan.gpuinfo.org/displaydevicelimit.php?name=maxDescriptorSetStorageBuffers&platform=all 
pub type ResourceSlot = [f32; 16];

impl From<Transform> for ResourceSlot {
    fn from(t: Transform) -> Self {
        let mut s = [0f32; 16];
        s[0] = t.offset[0]; s[1] = t.offset[1]; s[2] = t.scale; s[3] = t._padding;
        s
    }
}
impl From<ResourceSlot> for Transform {
    fn from(s: ResourceSlot) -> Self {
        Self { offset: [s[0], s[1]], scale: s[2], _padding: s[3] }
    }
}
impl From<ClipRect> for ResourceSlot {
    fn from(c: ClipRect) -> Self {
        let mut s = [0f32; 16];
        s[0] = c.x_clip[0]; s[1] = c.x_clip[1]; s[2] = c.y_clip[0]; s[3] = c.y_clip[1];
        s
    }
}
impl From<ResourceSlot> for ClipRect {
    fn from(s: ResourceSlot) -> Self {
        Self { x_clip: [s[0], s[1]], y_clip: [s[2], s[3]] }
    }
}
impl From<shapes::GradientGpu> for ResourceSlot {
    fn from(g: shapes::GradientGpu) -> Self {
        bytemuck::cast(g)
    }
}

// GpuSlabItem for ResourceSlot: store the free-list pointer in the first f32's bits.
// u32::MAX means None.
impl GpuSlabItem for ResourceSlot {
    fn next_free(&self) -> Option<usize> {
        let bits = self[0].to_bits();
        if bits == u32::MAX { None } else { Some(bits as usize) }
    }
    fn set_next_free(&mut self, i: Option<usize>) {
        self[0] = f32::from_bits(match i { Some(idx) => idx as u32, None => u32::MAX });
    }
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    pub image_renderer: ImageRenderer,
    pub text: Text,
    shapes: Shapes,
    resources: GpuSlab<ResourceSlot>,
    shapes_bind_group: wgpu::BindGroup,
    instances: GpuVec<Instance>,
    current_transform: TransformHandle,
    current_clip_rect: usize,
    // Deferred mode
    deferred_mode: bool,
    deferred_mode_start: usize,
    deferred_instances: Vec<Instance>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    p_type: u32,
    p_index: u32,
    transform_index: u32,
    clip_rect_index: u32,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        #[cfg(debug_assertions)] {
            assert_imported_keru_text_shader_matches();
        }

        let vs_spirv = include_bytes!("../slangc_output/shader.vert.spv");
        let vs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::util::make_spirv(vs_spirv),
        });

        let fs_spirv = include_bytes!("../slangc_output/shader.frag.spv");
        let fs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::util::make_spirv(fs_spirv),
        });

        let text = Text::new(&device, &queue, surface_format);

        let shapes = Shapes::new(&device);
        let image_renderer = ImageRenderer::new(&device, &queue);

        let mut resources: GpuSlab<ResourceSlot> = GpuSlab::new(&device, 64, "keru_draw clip_rects and transforms");
        let identity_index = resources.insert(Transform::identity().into()); // index 0: identity transform
        let no_clip_index = resources.insert(ClipRect::NO_CLIPPING.into()); // index 1: no clip
        debug_assert_eq!(identity_index, TransformHandle::IDENTITY.index);
        debug_assert_eq!(no_clip_index, ClipRectHandle::NO_CLIP.0);

        // Create merged bind group layout for shapes + images
        let shapes_bind_group_layout = Self::create_shapes_bind_group_layout(&device);

        // Create merged bind group
        let shapes_bind_group = Self::create_shapes_bind_group(
            &device,
            &shapes_bind_group_layout,
            &resources,
            &shapes,
            &image_renderer,
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                // The binding indices has to match the order in which the parameter blocks appear in the shader!
                // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
                // Binding order: shapes+images(0), keru_text(1)
                bind_group_layouts: &[
                    Some(&shapes_bind_group_layout),
                    Some(&text.bind_group_layout()),
                ],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: Some("main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Uint32,
                        },
                        wgpu::VertexAttribute {
                            offset: 4,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Uint32,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Uint32,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Uint32,
                        },
                    ],
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: Some("main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let instances = GpuVec::with_usage(
            &device,
            256,
            "keru_draw instances",
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        Self {
            deferred_mode: false,
            deferred_mode_start: 0,
            deferred_instances: Vec::with_capacity(5),
            device: device.clone(),
            queue: queue.clone(),
            current_transform: TransformHandle::IDENTITY,
            current_clip_rect: ClipRectHandle::NO_CLIP.0,
            render_pipeline, shapes, resources, image_renderer, text, shapes_bind_group, instances
        }
    }

    fn create_shapes_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let entries = &[
            GpuSlab::<ResourceSlot>::bind_group_layout_entry(0),
            GpuVec::<PrimitiveGpu>::bind_group_layout_entry(1),
            // Texture atlas
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX.union(wgpu::ShaderStages::FRAGMENT),
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            // Sampler
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("keru_draw shapes+images bind group layout"),
            entries: entries,
        })
    }

    fn create_shapes_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        resources: &GpuSlab<ResourceSlot>,
        shapes: &Shapes,
        image_renderer: &ImageRenderer,
    ) -> wgpu::BindGroup {
        let texture_view = image_renderer.texture_array.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shapes+Images Bind Group"),
            layout,
            entries: &[
                resources.bind_group_entry(0),
                shapes.primitives.bind_group_entry(1),
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&image_renderer.sampler),
                },
            ],
        })
    }

    // Shape drawing methods
    pub fn draw_box(&mut self, params: Rectangle) {
        let gradient_index = gradient_index_for_fill(&mut self.resources, &mut self.shapes.gradient_indices, params.fill);
        let (texture_uv_origin, texture_uv_size, texture_page, nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling) = texture_options_gpu(params.texture, params.texture_options);

        let index = self.shapes.push_primitive(shapes::RectangleGpu {
            top_left: params.top_left,
            size: params.size,
            corner_radius: params.corner_radius,
            border_thickness: params.border_thickness,
            gradient_direction: [0.0; 2],
            color_start: Color::default(),
            color_end: Color::default(),
            gradient_index,
            rounded_corners: params.rounded_corners.bits(),
            texture_uv_origin,
            texture_uv_size,
            texture_page,
            blur_radius: params.blur,
            nine_slice_l,
            nine_slice_r,
            nine_slice_t,
            nine_slice_b,
            nine_slice_tiling,
            ..Default::default()
        });
        self.push_instance(Instance {
            p_type: primitive::BOX,
            p_index: index,
            transform_index: self.current_transform.index as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    /// Draw an image.
    /// This is a convenience method equivalent to draw_box with white fill and the image as texture.
    pub fn draw_image(
        &mut self,
        image: LoadedImage,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.draw_box(Rectangle {
            top_left: [x, y],
            size: [width, height],
            corner_radius: 0.0,
            rounded_corners: RoundedCorners::NONE,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color::WHITE),
            texture: Some(image),
            texture_options: None,
            blur: 0.0,
        });
    }

    pub fn draw_circle(&mut self, params: Circle) {
        let gradient_index = gradient_index_for_fill(&mut self.resources, &mut self.shapes.gradient_indices, params.fill);
        let (texture_uv_origin, texture_uv_size, texture_page, nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling) = texture_options_gpu(params.texture, params.texture_options);

        let index = self.shapes.push_primitive(shapes::CircleGpu {
            center: params.center,
            radii: [0.0, params.radius],
            angles: [0.0, std::f32::consts::TAU],
            gradient_direction: [0.0; 2],
            color_start: Color::default(),
            color_end: Color::default(),
            gradient_index,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: 0.0,
            dash_offset: 0.0,
            blur_radius: params.blur,
            nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling,
            _ns_pad: [0.0; 2],
        });
        self.push_instance(Instance {
            p_type: primitive::CIRCLE,
            p_index: index,
            transform_index: self.current_transform.index as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    pub fn draw_ring(&mut self, params: CircleRing) {
        let gradient_index = gradient_index_for_fill(&mut self.resources, &mut self.shapes.gradient_indices, params.fill);
        let (texture_uv_origin, texture_uv_size, texture_page, nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling) = texture_options_gpu(params.texture, params.texture_options);

        let index = self.shapes.push_primitive(shapes::CircleGpu {
            center: params.center,
            radii: [params.inner_radius, params.outer_radius],
            angles: [0.0, std::f32::consts::TAU],
            gradient_direction: [0.0; 2],
            color_start: Color::default(),
            color_end: Color::default(),
            gradient_index,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: params.dash_length.unwrap_or(0.0),
            dash_offset: params.dash_offset,
            blur_radius: params.blur,
            nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling,
            _ns_pad: [0.0; 2],
        });
        self.push_instance(Instance {
            p_type: primitive::CIRCLE,
            p_index: index,
            transform_index: self.current_transform.index as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    pub fn draw_arc(&mut self, params: CircleArc) {
        let gradient_index = gradient_index_for_fill(&mut self.resources, &mut self.shapes.gradient_indices, params.fill);

        let (texture_uv_origin, texture_uv_size, texture_page, nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling) = texture_options_gpu(params.texture, params.texture_options);

        let index = self.shapes.push_primitive(shapes::CircleGpu {
            center: params.center,
            radii: [params.radius - params.thickness * 0.5, params.radius + params.thickness * 0.5],
            angles: [params.start_angle, params.end_angle],
            gradient_direction: [0.0; 2],
            color_start: Color::default(),
            color_end: Color::default(),
            gradient_index,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: params.dash_length.unwrap_or(0.0),
            dash_offset: params.dash_offset,
            blur_radius: params.blur,
            nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling,
            _ns_pad: [0.0; 2],
        });
        self.push_instance(Instance {
            p_type: primitive::CIRCLE,
            p_index: index,
            transform_index: self.current_transform.index as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    pub fn draw_pie(&mut self, params: CirclePie) {
        let gradient_index = gradient_index_for_fill(&mut self.resources, &mut self.shapes.gradient_indices, params.fill);
        let (texture_uv_origin, texture_uv_size, texture_page, nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling) = texture_options_gpu(params.texture, params.texture_options);

        let index = self.shapes.push_primitive(shapes::CircleGpu {
            center: params.center,
            radii: [0.0, params.radius],
            angles: [params.start_angle, params.end_angle],
            gradient_direction: [0.0; 2],
            color_start: Color::default(),
            color_end: Color::default(),
            gradient_index,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            dash_length: 0.0,
            dash_offset: 0.0,
            blur_radius: params.blur,
            nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling,
            _ns_pad: [params.stroke_thickness, params.corner_radius],
        });
        self.push_instance(Instance {
            p_type: primitive::CIRCLE,
            p_index: index,
            transform_index: self.current_transform.index as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    pub fn draw_segment(&mut self, params: Segment) {
        let gradient_index = gradient_index_for_fill(&mut self.resources, &mut self.shapes.gradient_indices, params.fill);
        let (texture_uv_origin, texture_uv_size, texture_page, nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling) = texture_options_gpu(params.texture, params.texture_options);

        let index = self.shapes.push_primitive(shapes::SegmentGpu {
            start: params.start,
            end: params.end,
            color_start: Color::default(),
            color_end: Color::default(),
            thickness_dash: [params.thickness, params.dash_length.unwrap_or(0.0), params.dash_offset, params.stroke_thickness],
            gradient_index,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            blur_radius: params.blur,
            nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling,
        });
        self.push_instance(Instance {
            p_type: primitive::SEGMENT,
            p_index: index,
            transform_index: self.current_transform.index as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    pub fn draw_grid(&mut self, params: Grid) {
        let gradient_index = gradient_index_for_fill(&mut self.resources, &mut self.shapes.gradient_indices, params.fill);
        let (texture_uv_origin, texture_uv_size, texture_page, nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling) = texture_options_gpu(params.texture, params.texture_options);

        let index = self.shapes.push_primitive(shapes::GridGpu {
            top_left: params.top_left,
            size: params.size,
            offset: params.offset,
            lattice_size: params.lattice_size,
            line_thickness: params.line_thickness,
            gradient_direction: [0.0; 2],
            color_start: Color::default(),
            color_end: Color::default(),
            gradient_index,
            grid_type: params.grid_type as u32,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            blur_radius: params.blur,
            nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling,
        });
        self.push_instance(Instance {
            p_type: primitive::GRID,
            p_index: index,
            transform_index: self.current_transform.index as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    pub fn draw_triangle(&mut self, params: Triangle) {
        let gradient_index = gradient_index_for_fill(&mut self.resources, &mut self.shapes.gradient_indices, params.fill);
        let (texture_uv_origin, texture_uv_size, texture_page, nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling) = texture_options_gpu(params.texture, params.texture_options);

        let index = self.shapes.push_primitive(shapes::TriangleGpu {
            p0: params.p0,
            p1: params.p1,
            p2: params.p2,
            gradient_direction: [0.0; 2],
            color_start: Color::default(),
            color_end: Color::default(),
            gradient_index,
            texture_page,
            texture_uv_origin,
            texture_uv_size,
            blur_radius: params.blur,
            nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling,
            stroke_thickness: params.stroke_thickness,
            _tri_pad: [params.corner_radius, 0.0, 0.0],
        });
        self.push_instance(Instance {
            p_type: primitive::TRIANGLE,
            p_index: index,
            transform_index: self.current_transform.index as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    pub fn draw_hexagon(&mut self, params: Hexagon) {
        let gradient_index = gradient_index_for_fill(&mut self.resources, &mut self.shapes.gradient_indices, params.fill);
        let (texture_uv_origin, texture_uv_size, texture_page, nine_slice_l, nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling) = texture_options_gpu(params.texture, params.texture_options);

        let index = self.shapes.push_primitive(shapes::HexagonGpu {
            center: params.center,
            size: params.size,
            rotation: params.rotation,
            gradient_direction: [0.0; 2],
            stroke_thickness: params.stroke_thickness,
            texture_page,
            color_start: Color::default(),
            color_end: Color::default(),
            gradient_index,
            nine_slice_l,
            texture_uv_origin,
            texture_uv_size,
            blur_radius: params.blur,
            nine_slice_r, nine_slice_t, nine_slice_b, nine_slice_tiling,
            _ns_pad: params.corner_radius,
        });
        self.push_instance(Instance {
            p_type: primitive::HEXAGON,
            p_index: index,
            transform_index: self.current_transform.index as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    pub fn draw_quadratic_bezier(&mut self, params: QuadraticBezier) {
        let gradient_index = push_gradient(&mut self.resources, &mut self.shapes.gradient_indices, shapes::GradientGpu::solid(params.color));
        let index = self.shapes.push_primitive(shapes::QuadraticBezierGpu {
            p0: params.p0,
            p1: params.p1,
            p2: params.p2,
            thickness: params.thickness,
            blur_radius: params.blur,
            gradient_index,
            stroke_thickness: params.stroke_thickness,
            _cu_pad: [0.0; 2],
        });
        self.push_instance(Instance {
            p_type: primitive::QUADRATIC_BEZIER,
            p_index: index,
            transform_index: self.current_transform.index as u32,
            clip_rect_index: self.current_clip_rect as u32,
        });
    }

    /// Draw a dashed box outline using segments and corner arcs.
    pub fn draw_dashed_box_outline(&mut self, params: DashedBoxOutline) {
        let [x, y] = params.top_left;
        let [w, h] = params.size;
        let r = params.corner_radius;
        let fill = ColorFill::Color(params.color);
        let mut offset = 0.0f32;

        if r < 0.001 {
            // No rounded corners - just 4 segments
            // Top edge
            self.draw_segment(Segment {
                start: [x, y],
                end: [x + w, y],
                thickness: params.thickness,
                stroke_thickness: 0.0,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
                texture_options: None,
                blur: params.blur,
            });
            offset += w;
            // Right edge
            self.draw_segment(Segment {
                start: [x + w, y],
                end: [x + w, y + h],
                thickness: params.thickness,
                stroke_thickness: 0.0,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
                texture_options: None,
                blur: params.blur,
            });
            offset += h;
            // Bottom edge
            self.draw_segment(Segment {
                start: [x + w, y + h],
                end: [x, y + h],
                thickness: params.thickness,
                stroke_thickness: 0.0,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
                texture_options: None,
                blur: params.blur,
            });
            offset += w;
            // Left edge
            self.draw_segment(Segment {
                start: [x, y + h],
                end: [x, y],
                thickness: params.thickness,
                stroke_thickness: 0.0,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
                texture_options: None,
                blur: params.blur,
            });
        } else {
            // Rounded corners - 4 segments + 4 quarter arcs
            let pi = std::f32::consts::PI;
            let quarter_arc = r * pi * 0.5; // length of quarter circle arc

            // Top edge (between top-left and top-right corners)
            let top_len = w - 2.0 * r;
            self.draw_segment(Segment {
                start: [x + r, y],
                end: [x + w - r, y],
                thickness: params.thickness,
                stroke_thickness: 0.0,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
                texture_options: None,
                blur: params.blur,
            });
            offset += top_len;
            // Top-right corner arc
            self.draw_arc(CircleArc {
                center: [x + w - r, y + r],
                radius: r,
                start_angle: -pi * 0.5,
                end_angle: 0.0,
                thickness: params.thickness,
                fill,
                texture: None,
                texture_options: None,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                blur: params.blur,
            });
            offset += quarter_arc;
            // Right edge
            let right_len = h - 2.0 * r;
            self.draw_segment(Segment {
                start: [x + w, y + r],
                end: [x + w, y + h - r],
                thickness: params.thickness,
                stroke_thickness: 0.0,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
                texture_options: None,
                blur: params.blur,
            });
            offset += right_len;
            // Bottom-right corner arc
            self.draw_arc(CircleArc {
                center: [x + w - r, y + h - r],
                radius: r,
                start_angle: 0.0,
                end_angle: pi * 0.5,
                thickness: params.thickness,
                fill,
                texture: None,
                texture_options: None,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                blur: params.blur,
            });
            offset += quarter_arc;
            // Bottom edge
            self.draw_segment(Segment {
                start: [x + w - r, y + h],
                end: [x + r, y + h],
                thickness: params.thickness,
                stroke_thickness: 0.0,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
                texture_options: None,
                blur: params.blur,
            });
            offset += top_len;
            // Bottom-left corner arc
            self.draw_arc(CircleArc {
                center: [x + r, y + h - r],
                radius: r,
                start_angle: pi * 0.5,
                end_angle: pi,
                thickness: params.thickness,
                fill,
                texture: None,
                texture_options: None,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                blur: params.blur,
            });
            offset += quarter_arc;
            // Left edge
            self.draw_segment(Segment {
                start: [x, y + h - r],
                end: [x, y + r],
                thickness: params.thickness,
                stroke_thickness: 0.0,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
                texture_options: None,
                blur: params.blur,
            });
            offset += right_len;
            // Top-left corner arc
            self.draw_arc(CircleArc {
                center: [x + r, y + r],
                radius: r,
                start_angle: pi,
                end_angle: pi * 1.5,
                thickness: params.thickness,
                fill,
                texture: None,
                texture_options: None,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                blur: params.blur,
            });
        }
    }

    /// Draw a dashed hexagon outline using 6 segments.
    pub fn draw_dashed_hexagon_outline(&mut self, params: DashedHexagonOutline) {
        let fill = ColorFill::Color(params.color);
        let pi = std::f32::consts::PI;

        // Calculate the 6 vertices of the hexagon
        let mut vertices = [[0.0f32; 2]; 6];
        for i in 0..6 {
            let angle = params.rotation + (i as f32) * pi / 3.0;
            vertices[i] = [
                params.center[0] + params.size * angle.cos(),
                params.center[1] + params.size * angle.sin(),
            ];
        }

        // Edge length of regular hexagon = size (distance from center to vertex)
        let edge_len = params.size;
        let mut offset = 0.0f32;

        // Draw 6 segments connecting the vertices
        for i in 0..6 {
            let next = (i + 1) % 6;
            self.draw_segment(Segment {
                start: vertices[i],
                end: vertices[next],
                thickness: params.thickness,
                stroke_thickness: 0.0,
                fill,
                dash_length: Some(params.dash_length),
                dash_offset: offset,
                texture: None,
                texture_options: None,
                blur: params.blur,
            });
            offset += edge_len;

        }
    }

    /// Draw a text box.
    pub fn draw_text_box(&mut self, text_box: &TextBoxHandle) {
        let transform = self.current_transform.text_transform;
        self.text.get_text_box_mut(text_box).set_group_transform(transform);

        let glyph_range = self.text.get_text_box(text_box).glyph_quad_range();

        for q in (glyph_range.0)..(glyph_range.1) {
            self.push_instance(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
                transform_index: self.current_transform.index as u32,
                clip_rect_index: self.current_clip_rect as u32,
            });
        }
    }

    /// Draw a text edit widget.
    pub fn draw_text_edit(&mut self, text_edit: &TextEditHandle) {
        let transform = self.current_transform.text_transform;
        self.text.get_text_edit_mut(text_edit).set_group_transform(transform);

        let glyph_range = self.text.get_text_edit(text_edit).glyph_quad_range();

        for q in (glyph_range.0)..(glyph_range.1) {
            self.push_instance(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
                transform_index: self.current_transform.index as u32,
                clip_rect_index: self.current_clip_rect as u32,
            });
        }
    }

    /// Draw decoration quads (selection rects + cursor blink rect) for all text edits.
    ///
    /// Decoration quads are shared across all text boxes, so this should be called once per frame
    /// after all `draw_text_edit` calls.
    pub fn draw_text_decorations(&mut self) {
        let decoration_range = self.text.decoration_quad_range();

        for q in (decoration_range.0)..(decoration_range.1) {
            self.push_instance(Instance {
                p_type: primitive::TEXT,
                p_index: q as u32,
                transform_index: self.current_transform.index as u32,
                clip_rect_index: self.current_clip_rect as u32,
            });
        }
    }

    /// Begin recording a new frame.
    ///
    /// This only clears the main instance buffer, not the shape data, transforms, clip_rects, or deferred instances, so they can be reused.
    ///
    /// To clear everything, call [`Renderer::clear_for_new_frame()`].
    pub fn begin_frame(&mut self) {
        self.instances.clear();
        self.current_transform = TransformHandle::IDENTITY;
        self.current_clip_rect = ClipRectHandle::NO_CLIP.0;
        self.deferred_mode = false;
        self.deferred_mode_start = 0;
    }

    /// Clear all the render data, including shapes, deferred instances, transforms, and clip_rects, and begin a new frame from scratch.
    pub fn clear_for_new_frame(&mut self) {
        self.instances.clear();
        for idx in self.shapes.gradient_indices.drain(..) {
            self.resources.remove(idx);
        }
        self.shapes.clear();
        self.deferred_instances.clear();
        self.current_transform = TransformHandle::IDENTITY;
        self.current_clip_rect = ClipRectHandle::NO_CLIP.0;
    }

    pub fn prepare_text(&mut self) {
        self.text.prepare_all();
    }

    /// Internal helper to push an instance to the correct buffer based on deferred mode.
    fn push_instance(&mut self, instance: Instance) {
        if self.deferred_mode {
            self.deferred_instances.push(instance);
        } else {
            self.instances.push(instance);
        }
    }

    /// Start deferred mode. While in deferred mode, draw calls will be recorded
    /// but not added to the main instance buffer. Call `end_deferred_mode()` to
    /// get a handle to the recorded instances, which can later be drawn with
    /// `draw_deferred_elements()`.
    pub fn start_deferred_mode(&mut self) {
        self.deferred_mode = true;
        self.deferred_mode_start = self.deferred_instances.len();
    }

    /// End deferred mode and return a handle to the recorded instances.
    /// The returned [`DeferredInstanceRange`] can be used with `draw_deferred_elements()`to copy the recorded instances to the main buffer at any time.
    pub fn end_deferred_mode(&mut self) -> DeferredInstanceRange {
        self.deferred_mode = false;
        DeferredInstanceRange {
            start: self.deferred_mode_start,
            end: self.deferred_instances.len(),
        }
    }

    /// Copy deferred instances to the main instance buffer.
    /// 
    /// This allows drawing primitives in a different order than they were created.
    pub fn draw_deferred_elements(&mut self, range: DeferredInstanceRange) {
        self.instances.vec_mut().extend_from_slice(&self.deferred_instances[range.start..range.end]);                                                         
    }

    /// Set the current transform to an existing transform handle.
    /// All subsequent draw calls will use this transform until `pop_current_transform` is called.
    pub fn set_current_transform(&mut self, handle: TransformHandle) {
        self.current_transform = handle;
    }

    /// Reset the current transform back to identity.
    pub fn clear_current_transform(&mut self) {
        self.current_transform = TransformHandle::IDENTITY
    }

    /// Create a retained transform.
    /// The returned `TransformHandle` is valid until [`Renderer::remove_transform()`] is called on it.
    pub fn insert_transform(&mut self, transform: Transform) -> TransformHandle {
        let draw_index = self.resources.insert(transform.into());
        // Also create a keru_text GroupTransform
        let text_transform = keru_text::GroupTransform {
            offset: transform.offset,
            scale: transform.scale,
            _padding: 0.0,
        };
        let text_handle = self.text.insert_group_transform(text_transform);
        TransformHandle { index: draw_index, text_transform: text_handle }
    }

    /// Remove a retained transform.
    pub fn remove_transform(&mut self, handle: TransformHandle) {
        self.resources.remove(handle.index);
        // Also remove from keru_text group transforms
        self.text.remove_group_transform(handle.text_transform);
    }

    /// Modify a transform.
    /// All instances using this transform will be affected.
    pub fn update_transform(&mut self, handle: TransformHandle, transform: Transform) {
        self.resources[handle.index] = transform.into();
        // Also update keru_text group transform
        let text_transform = keru_text::GroupTransform {
            offset: transform.offset,
            scale: transform.scale,
            _padding: 0.0,
        };
        self.text.update_group_transform(handle.text_transform, text_transform);
    }

    /// Get the value of a transform.
    pub fn get_transform(&self, handle: TransformHandle) -> Transform {
        self.resources[handle.index].into()
    }

    /// Set the current clip rect to an existing clip rect handle.
    /// All subsequent draw calls will use this clip rect until `clear_current_clip_rect` is called.
    pub fn set_current_clip_rect(&mut self, handle: ClipRectHandle) {
        self.current_clip_rect = handle.0;
    }

    /// Reset the current clip rect back to "no clip".
    pub fn clear_current_clip_rect(&mut self) {
        self.current_clip_rect = ClipRectHandle::NO_CLIP.0;
    }

    /// Store a gradient in the resource buffer and return a handle for reuse within the frame.
    /// The handle can be used with [`ColorFill::SharedGradient`] to apply the same gradient
    /// to multiple shapes.
    pub fn create_gradient(&mut self, gradient: Gradient) -> SharedGradient {
        let index = self.resources.insert(gradient.to_gpu().into());
        self.shapes.gradient_indices.push(index);
        SharedGradient(index as u32)
    }

    /// Create a retained clip rect.
    /// The returned `ClipRectHandle` is valid until [`Renderer::remove_clip_rect()`] is called on it.
    pub fn insert_clip_rect(&mut self, clip_rect: ClipRect) -> ClipRectHandle {
        let index = self.resources.insert(clip_rect.into());
        ClipRectHandle(index)
    }

    /// Remove a retained clip rect.
    pub fn remove_clip_rect(&mut self, handle: ClipRectHandle) {
        self.resources.remove(handle.0);
    }

    /// Modify a clip rect.
    /// All instances using this clip rect will be affected.
    pub fn update_clip_rect(&mut self, handle: ClipRectHandle, clip_rect: ClipRect) {
        self.resources[handle.0] = clip_rect.into();
    }

    /// Get the value of a clip rect.
    pub fn get_clip_rect(&self, handle: ClipRectHandle) -> ClipRect {
        self.resources[handle.0].into()
    }

    /// Render into a render pass.
    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        // Upload resources to GPU
        let slots_realloc = self.resources.load_to_gpu(&self.device, &self.queue);
        let shapes_realloc = self.shapes.load_to_gpu(&self.device, &self.queue);
        let images_realloc = self.image_renderer.load_to_gpu(&self.device, &self.queue);

        // Recreate bind group if slots, shapes or images realloc
        if slots_realloc || shapes_realloc || images_realloc {
            let layout = Self::create_shapes_bind_group_layout(&self.device);
            self.shapes_bind_group = Self::create_shapes_bind_group(
                &self.device,
                &layout,
                &self.resources,
                &self.shapes,
                &self.image_renderer,
            );
        }

        self.text.load_to_gpu();
        self.instances.load_to_gpu(&self.device, &self.queue);

        self.set_pipeline_state(render_pass);

        render_pass.draw(0..4, 0..self.instances.len() as u32);
    }

    pub fn load_to_gpu(&mut self) {
        // Upload resources to GPU
        let slots_changed = self.resources.load_to_gpu(&self.device, &self.queue);
        let shapes_changed = self.shapes.load_to_gpu(&self.device, &self.queue);
        let images_changed = self.image_renderer.load_to_gpu(&self.device, &self.queue);

        // Recreate bind group if slots, shapes or images changed
        if slots_changed || shapes_changed || images_changed {
            let layout = Self::create_shapes_bind_group_layout(&self.device);
            self.shapes_bind_group = Self::create_shapes_bind_group(
                &self.device,
                &layout,
                &self.resources,
                &self.shapes,
                &self.image_renderer,
            );
        }

        self.text.load_to_gpu();
        self.instances.load_to_gpu(&self.device, &self.queue);
    }

    pub fn set_pipeline_state(&mut self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        // The binding indices has to match the order in which the parameter blocks appear in the shader!
        // If there are issues, compile the shaders with the -reflection-json flag and see the parameterBlock fields.
        // Binding order: shapes+images(0), keru_text(1)
        render_pass.set_bind_group(0, &self.shapes_bind_group, &[]);
        render_pass.set_bind_group(1, &self.text.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.instances.buffer().slice(..));
    }

    /// Render a specific range of instances into a render pass.
    ///
    /// This is useful for custom rendering where you want to interleave
    /// Keru's rendering with your own custom drawing code.
    ///
    /// Note: You must call `setup_render_pass()` before calling this method.
    pub fn render_range(&mut self, render_pass: &mut wgpu::RenderPass, range: InstanceRange) {
        if range.start >= range.end || range.start >= self.instances.len() {
            return;
        }

        self.set_pipeline_state(render_pass);

        let real_end = range.end.min(self.instances.len());
        render_pass.draw(0..4, range.start as u32..real_end as u32);
    }

    /// Convenience function that creates a render pass, renders into it, and presents to the screen.
    /// 
    /// Panics if the current surface texture can't be obtained from `surface`.  
    pub fn autorender(&mut self, surface: &wgpu::Surface, background_color: wgpu::Color) {
        let output = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            other => panic!("Failed to get current surface texture: {other:?}"),
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("keru_draw autorender render encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("keru_draw autorender render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(background_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            self.render(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        self.queue.present(output);
    }

    // todo: remove this and make nicer start/end methods 
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }
}

#[cfg(debug_assertions)] 
fn assert_imported_keru_text_shader_matches() {
    let imported_shader = include_str!("shaders/keru_text.slang");
    let original_shader = keru_text::Text::composable_shader_source();
    assert!(imported_shader == original_shader);
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn test_imported_shaders() {
        assert_imported_keru_text_shader_matches();
    }
}