use iced::{
    advanced::{
        self,
        layout::{Layout, Limits, Node},
        text::{self, paragraph::Plain, LineHeight, Paragraph, Shaping, Wrapping},
        widget::{tree, Widget},
    },
    alignment::{self, Horizontal, Vertical},
    mouse, Color, Element, Length, Padding, Pixels, Point, Rectangle, Size,
};

#[allow(unused_imports)]
use iced::widget::Text;

/// Alternative to [`Text`] with optional [`Icon`] support.
pub struct Base<Renderer: text::Renderer> {
    value: String,
    icon: Option<Icon<Renderer::Font>>,
    font: Option<Renderer::Font>,
    size: Option<Pixels>,
    width: Length,
    height: Length,
    padding: Padding,
    horizontal: Horizontal,
    line_height: LineHeight,
}

impl<Renderer: text::Renderer> Base<Renderer> {
    /// Creates a new [`Base`] widget with the provided value.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            icon: None,
            size: None,
            font: None,
            width: Length::Shrink,
            height: Length::Shrink,
            line_height: LineHeight::default(),
            padding: [2, 4].into(),
            horizontal: Horizontal::Left,
        }
    }

    /// Sets the width of the [`Base`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Base`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Icon`] of the [`Base`].
    pub fn icon(mut self, icon: impl Into<Icon<Renderer::Font>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the [`Padding`] of the [`Base`].
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the `Font` of the [`Base`].
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the text size of the [`Base`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the [`Horizontal`] alignment of the [`Base`].
    pub fn align_x(mut self, alignment: impl Into<Horizontal>) -> Self {
        self.horizontal = alignment.into();
        self
    }

    /// Sets the [`LineHeight`] of the [`Base`].
    pub fn line_height(mut self, height: impl Into<LineHeight>) -> Self {
        self.line_height = height.into();
        self
    }
}

impl<Message, Renderer> Widget<Message, iced::Theme, Renderer> for Base<Renderer>
where
    Renderer: text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<BaseState<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(BaseState::<Renderer::Paragraph>::default())
    }

    fn layout(&self, tree: &mut tree::Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let state = tree.state.downcast_mut::<BaseState<Renderer::Paragraph>>();

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let text_size = self.size.unwrap_or_else(|| renderer.default_size());
        let padding = self.padding;
        let height = self.line_height.to_absolute(text_size);

        state.value.update(text::<Renderer>(
            &self.value,
            Size::new(f32::INFINITY, height.0),
            font,
            self.horizontal,
            self.line_height,
            text_size,
        ));

        if let Some(icon) = &self.icon {
            let mut content = [0; 8];

            let icon_text = text::<Renderer>(
                icon.code_point.encode_utf8(&mut content) as &_,
                Size::new(f32::INFINITY, height.0),
                icon.font,
                Horizontal::Left,
                self.line_height,
                icon.size.unwrap_or_else(|| renderer.default_size()),
            );

            state.icon.update(icon_text);

            let icon_width = state.icon.min_width();

            let text_position = Point::new(padding.left + icon_width + icon.spacing, padding.top);

            let icon_position = Point::new(padding.left, padding.top);

            let icon_size = state.icon.min_bounds();
            let text_size = state.value.min_bounds();

            let total_size = Size::new(
                icon_size.width + icon.spacing + text_size.width,
                icon_size.height.max(text_size.height),
            );

            let size = limits
                .resolve(self.width, self.height, total_size)
                .expand(padding);

            let text_node = Node::new(text_size).move_to(text_position);

            let icon_node = Node::new(icon_size).move_to(icon_position);

            Node::with_children(size, vec![text_node, icon_node])
        } else {
            let text_size = state.value.min_bounds();
            let size = limits
                .resolve(self.width, self.height, text_size)
                .expand(padding);
            let text = Node::new(text_size).move_to(Point::new(padding.left, padding.top));

            Node::with_children(size, vec![text])
        }
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        _theme: &iced::Theme,
        style: &advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<BaseState<Renderer::Paragraph>>();

        let bounds = layout.bounds();

        let Some(viewport) = bounds.intersection(viewport) else {
            return;
        };

        let mut children = layout.children();

        let value = children.next().expect("Base draw: Missing value layout");

        if let Some(viewport) = value.bounds().intersection(&viewport) {
            draw(
                renderer,
                style.text_color,
                value,
                state.value.raw(),
                &viewport,
            );
        }

        if self.icon.is_some() {
            let icon = children.next().expect("Widget draw: Missing icon layout");

            if let Some(viewport) = icon.bounds().intersection(&viewport) {
                draw(
                    renderer,
                    style.text_color,
                    icon,
                    state.icon.raw(),
                    &viewport,
                );
            }
        }
    }

    fn mouse_interaction(
        &self,
        _state: &tree::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message, Renderer> From<Base<Renderer>> for Element<'a, Message, iced::Theme, Renderer>
where
    Renderer: text::Renderer + 'a,
    Message: 'a,
{
    fn from(value: Base<Renderer>) -> Self {
        Self::new(value)
    }
}

#[derive(Default)]
struct BaseState<P: text::Paragraph> {
    value: Plain<P>,
    icon: Plain<P>,
}

#[derive(Debug, Clone, Copy, Default)]
/// The icon content.
pub struct Icon<Font = iced::Font> {
    /// The font used for the `code_point`.
    pub font: Font,
    /// The unicode code point used as the icon.
    pub code_point: char,
    /// The font size of the content.
    pub size: Option<Pixels>,
    /// The spacing between the [`Icon`] and the text.
    pub spacing: f32,
}

fn text<Renderer: text::Renderer>(
    content: &str,
    bounds: Size,
    font: Renderer::Font,
    horizontal: Horizontal,
    line_height: LineHeight,
    size: Pixels,
) -> text::Text<&str, Renderer::Font> {
    text::Text {
        content,
        bounds,
        size,
        line_height,
        horizontal_alignment: horizontal,
        vertical_alignment: Vertical::Center,
        font,
        shaping: Shaping::Advanced,
        wrapping: Wrapping::Word,
    }
}

fn draw<Renderer>(
    renderer: &mut Renderer,
    text_color: Color,
    layout: Layout<'_>,
    paragraph: &Renderer::Paragraph,
    viewport: &Rectangle,
) where
    Renderer: text::Renderer,
{
    let bounds = layout.bounds();

    let x = match paragraph.horizontal_alignment() {
        alignment::Horizontal::Left => bounds.x,
        alignment::Horizontal::Center => bounds.center_x(),
        alignment::Horizontal::Right => bounds.x + bounds.width,
    };

    let y = match paragraph.vertical_alignment() {
        alignment::Vertical::Top => bounds.y,
        alignment::Vertical::Center => bounds.center_y(),
        alignment::Vertical::Bottom => bounds.y + bounds.height,
    };

    renderer.fill_paragraph(paragraph, Point::new(x, y), text_color, *viewport);
}
