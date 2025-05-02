use iced::{
    advanced::{
        self,
        layout::{self, Node},
        mouse, overlay,
        renderer::Quad,
        text,
    },
    Padding, Point, Rectangle, Size,
};

use super::{draw, Catalog, Cell, CELL_GAP};

const SCALING: f32 = 0.75;

pub struct Overlay<'a, 'b, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
    'b: 'a,
{
    position: Point,
    cells: Vec<(Rectangle, &'a Cell<Renderer>, usize)>,
    is_row: bool,
    padding: Padding,
    class: &'a <Theme as Catalog>::Class<'b>,
}

impl<'a, 'b, Theme, Renderer> Overlay<'a, 'b, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
    'b: 'a,
{
    pub fn new(
        bounds: impl Iterator<Item = (Rectangle, &'a Cell<Renderer>, usize)>,
        position: Point,
        is_row: bool,
        padding: Padding,
        class: &'a <Theme as Catalog>::Class<'b>,
    ) -> Self {
        let cells = bounds
            .map(|(rect, cell, row)| (rect * SCALING, cell, row))
            .collect();

        Self {
            cells,
            position,
            is_row,
            padding,
            class,
        }
    }
}

impl<'a, 'b, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'a, 'b, Theme, Renderer>
where
    Renderer: advanced::Renderer + text::Renderer,
    Theme: Catalog,
    'b: 'a,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        let mut nodes = vec![];
        let mut height = 0.0;
        let mut width = 0.0;

        for (bounds, _, _) in self.cells.iter() {
            let size = bounds.size();
            let y = if self.is_row { 0.0 } else { height };
            let x = if self.is_row { width } else { 0.0 };
            let position = Point::new(x, y);

            height = if self.is_row {
                size.height
            } else {
                height + size.height
            };
            width = if !self.is_row {
                size.width
            } else {
                width + size.width
            };

            let node = Node::new(size).move_to(position);
            nodes.push(node)
        }

        let node = Node::with_children(Size::new(width, height), nodes);

        node.move_to(self.position)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let style = theme.style(self.class);
        let alpha = 0.85;
        let gap = CELL_GAP / 2.0;

        for (layout, (_, cell, row)) in layout.children().zip(self.cells.iter()) {
            let bounds = layout.bounds();

            renderer.fill_quad(
                Quad {
                    bounds,
                    ..Default::default()
                },
                style.cell_border.scale_alpha(alpha),
            );

            let (cell_background, text_color) = if row % 2 == 0 {
                (
                    style.alternating_backgrounds.1,
                    style.alternating_text_color.1,
                )
            } else {
                (
                    style.alternating_backgrounds.0,
                    style.alternating_text_color.0,
                )
            };

            renderer.fill_quad(
                Quad {
                    bounds: layout.bounds().shrink([gap, gap]),
                    ..Default::default()
                },
                cell_background.scale_alpha(alpha),
            );

            draw(
                renderer,
                text_color.scale_alpha(alpha),
                layout,
                cell.raw(),
                self.padding,
                &layout.bounds(),
            )
        }
    }
}
