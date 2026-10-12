use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextContent {
    None,
    Box,
    Edit,
}

/// The parts of a node that the layout reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LayoutParams {
    pub size: Xy<Size>,
    pub min_size: Xy<Option<Size>>,
    pub max_size: Xy<Option<Size>>,
    pub padding: Xy<f32>,
    pub children_layout: ChildrenLayout,
    pub free_placement: bool,
    pub grid_span: Xy<u16>,
    pub text: TextContent,
    /// The size of the node's raster image in pixels.
    pub image: Option<Xy<f32>>,
    /// Exiting nodes stay in their parent's child list for the exit animation, but the layout skips them.
    pub disabled: bool,
}

const EMPTY_PARAMS: LayoutParams = LayoutParams {
    size: Xy { x: Size::Pixels(0.0), y: Size::Pixels(0.0) },
    min_size: Xy { x: None, y: None },
    max_size: Xy { x: None, y: None },
    padding: Xy { x: 0.0, y: 0.0 },
    children_layout: ChildrenLayout::Free,
    free_placement: false,
    grid_span: Xy { x: 1, y: 1 },
    text: TextContent::None,
    image: None,
    disabled: true,
};

#[derive(Debug)]
pub(crate) struct LayoutTree {
    pub params: NodeVec<LayoutParams>,
}

impl LayoutTree {
    pub fn new() -> Self {
        LayoutTree { params: NodeVec(Vec::with_capacity(100)) }
    }

    pub fn init(&mut self, i: NodeI, params: LayoutParams) {
        if self.params.0.len() <= i.as_usize() {
            self.params.0.resize(i.as_usize() + 1, EMPTY_PARAMS);
        }
        self.params[i] = params;
    }
}

pub(crate) fn layout_params(node: &InnerNode) -> LayoutParams {
    let params = &node.params;
    let text = match node.text_i {
        Some(TextI::TextBox(_)) => TextContent::Box,
        Some(TextI::TextEdit(_)) => TextContent::Edit,
        None => TextContent::None,
    };
    LayoutParams {
        size: params.layout.size,
        min_size: params.layout.min_size,
        max_size: params.layout.max_size,
        padding: params.layout.padding,
        children_layout: params.children_layout,
        free_placement: params.free_placement,
        grid_span: Xy::new(params.grid_element.column_span, params.grid_element.row_span),
        text,
        image: image_size(node),
        disabled: node.exiting,
    }
}

pub(crate) fn image_size(node: &InnerNode) -> Option<Xy<f32>> {
    match &node.imageref {
        Some(ImageRef::Raster(loaded)) => Some(Xy::new(loaded.width as f32, loaded.height as f32)),
        _ => None,
    }
}
