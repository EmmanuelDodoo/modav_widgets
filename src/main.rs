#![allow(unused_imports, dead_code)]
use iced::{
    application, font,
    widget::{center, container, container::bordered_box, text, tooltip::Position, Tooltip},
    Element, Font, Length, Renderer, Theme,
};

use context::*;
use menu::*;

fn main() -> iced::Result {
    application("Playground", App::update, App::view)
        .theme(|_| Theme::TokyoNightLight)
        .run_with(|| {
            let font = font::load(include_bytes!("../fontello.ttf")).map(|_| Message::None);

            (App, font)
        })
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Test,
    None,
}

struct App;

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Test => println!("Testing"),
            Message::None => {}
        }
    }

    fn main<'a>() -> Vec<Item<'a, Message>> {
        let font = Font::with_name("fontello");

        let one = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example Menu");
            Item::new(icon, text)
        };

        let two: Item<'_, Message> = {
            let tip = "Some test tooltip";
            //let icon = text("\u{F0F6}").font(font);
            let tip = Tooltip::new("H", tip, Position::Right);

            let text = text("Cringe Two");
            Item::with_tooltip(tip, text).message(Message::Test)
        };
        let three: Item<'_, Message> = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example 3");
            Item::new(icon, text)
        };
        let four: Item<'_, Message> = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example 3");
            Item::new(icon, text)
        };
        let five: Item<'_, Message> = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example 3");
            Item::new(icon, text)
        };
        let six: Item<'_, Message> = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example 3");
            Item::new(icon, text)
        };
        let seven: Item<'_, Message> = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example 3");
            Item::new(icon, text)
        };

        vec![one, two, three, four, five, six, seven]
    }

    fn footer<'a>() -> Vec<Item<'a, Message>> {
        let font = Font::with_name("fontello");

        let one = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Settings");
            Item::new(icon, text)
        };

        let two = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("About");
            Item::new(icon, text)
        };
        let three = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Help");
            Item::new(icon, text)
        };

        vec![one, two, three]
    }

    fn view(&self) -> Element<'_, Message> {
        let header: Item<'_, Message> = {
            let base = Font {
                //weight: font::Weight::Semibold,
                //style: font::Style::Italic,
                ..Default::default()
            };

            let font = Font::with_name("fontello");

            let header = {
                let icon = text("\u{F0F6}").font(font).size(24);
                let text = text("Header").font(base).size(24);
                Item::new(icon, text)
            };

            header
        };

        let main: Section<'_, Message> = Section::from_vec(Self::main()).spacing(20.0);
        let footer: Section<'_, Message> = Section::from_vec(Self::footer());

        let cols = Collapsible::new(header, main, footer);
        //let cols = Collapsible::no_header(main, footer);
        //let cols = Collapsible::no_footer(header, main);
        //let cols = Collapsible::only_main(main);

        let temp: Element<'_, Message, Theme, Renderer> =
            center(container(cols).style(bordered_box))
                .center(Length::Fill)
                .into();

        //temp.explain(color!(255, 0, 0))
        temp
    }
}

mod menu {
    use iced::{
        advanced::{self, layout, renderer::Quad, widget::tree, Clipboard, Shell, Widget},
        alignment::{Horizontal, Vertical},
        event, mouse,
        widget::{Space, Text},
        Background, Border, Color, Element, Event, Length, Padding, Point, Rectangle, Renderer,
        Size, Theme, Vector,
    };

    pub use items::Item;
    pub use sections::Section;

    use items::State as ItemState;
    use overlay::Overlay;

    mod items {
        use super::*;
        use iced::widget::Tooltip;

        pub struct Item<'a, Message>
        where
            Message: Clone,
        {
            children: [Element<'a, Message, Theme, Renderer>; 2],
            message: Option<Message>,
            padding: Padding,
            spacing: f32,
            align: Vertical,
            width: Length,
            height: Length,
        }

        impl<'a, Message> Item<'a, Message>
        where
            Message: Clone + 'a,
        {
            pub fn from_slice(children: [Element<'a, Message, Theme, Renderer>; 2]) -> Self {
                Self {
                    children,
                    message: None,
                    spacing: 10.0,
                    padding: [6.0, 10.0].into(),
                    align: Vertical::Center,
                    width: Length::Shrink,
                    height: Length::Shrink,
                }
            }

            pub fn new(icon: Text<'a>, label: Text<'a>) -> Self {
                Self::from_slice([icon.into(), label.into()])
            }

            pub fn with_tooltip(tooltip: Tooltip<'a, Message>, label: Text<'a>) -> Self {
                Self::from_slice([tooltip.into(), label.into()])
            }

            pub fn message(mut self, message: Message) -> Self {
                self.message = Some(message);
                self
            }

            pub fn width(mut self, width: impl Into<Length>) -> Self {
                self.width = width.into();
                self
            }

            pub fn height(mut self, height: impl Into<Length>) -> Self {
                self.height = height.into();
                self
            }

            pub fn spacing(mut self, spacing: f32) -> Self {
                self.spacing = spacing;
                self
            }

            pub fn align(mut self, align: impl Into<Vertical>) -> Self {
                self.align = align.into();
                self
            }

            pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
                self.padding = padding.into();
                self
            }
        }

        impl<'a, Message> Widget<Message, Theme, Renderer> for Item<'a, Message>
        where
            Message: Clone,
        {
            fn children(&self) -> Vec<tree::Tree> {
                self.children.iter().map(tree::Tree::new).collect()
            }

            fn diff(&self, tree: &mut tree::Tree) {
                tree.diff_children(&self.children);
            }

            fn tag(&self) -> tree::Tag {
                tree::Tag::of::<State>()
            }

            fn state(&self) -> tree::State {
                tree::State::new(State::new())
            }

            fn size(&self) -> Size<Length> {
                Size::new(self.width, self.height)
            }

            fn layout(
                &self,
                tree: &mut tree::Tree,
                renderer: &Renderer,
                limits: &layout::Limits,
            ) -> layout::Node {
                let state = tree.state.downcast_ref::<State>();

                if state.collapsed {
                    let limits = limits.shrink(Size::new(
                        self.padding.horizontal(),
                        self.padding.vertical(),
                    ));

                    let position = Point::new(self.padding.left, self.padding.top);
                    let child = self.children[0]
                        .as_widget()
                        .layout(&mut tree.children[0], renderer, &limits)
                        .move_to(position);

                    layout::Node::container(child, self.padding)
                } else {
                    layout::flex::resolve(
                        layout::flex::Axis::Horizontal,
                        renderer,
                        limits,
                        self.width,
                        self.height,
                        self.padding,
                        self.spacing,
                        self.align.into(),
                        &self.children,
                        &mut tree.children,
                    )
                }
            }

            fn on_event(
                &mut self,
                tree: &mut tree::Tree,
                event: Event,
                layout: layout::Layout<'_>,
                cursor: mouse::Cursor,
                renderer: &Renderer,
                clipboard: &mut dyn Clipboard,
                shell: &mut Shell<'_, Message>,
                viewport: &Rectangle,
            ) -> event::Status {
                let state = tree.state.downcast_mut::<State>();
                let bounds = layout.bounds();

                if event == Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    && cursor.is_over(bounds)
                {
                    if let Some(message) = self.message.clone() {
                        shell.publish(message);
                    }

                    state.collapsed = false;

                    event::Status::Captured
                } else if state.collapsed {
                    let tree = &mut tree.children[0];

                    let layout = layout.children().next().expect("Menu missing first child");

                    self.children[0].as_widget_mut().on_event(
                        tree, event, layout, cursor, renderer, clipboard, shell, viewport,
                    )
                } else {
                    self.children
                        .iter_mut()
                        .zip(&mut tree.children)
                        .zip(layout.children())
                        .map(|((child, state), layout)| {
                            child.as_widget_mut().on_event(
                                state,
                                event.clone(),
                                layout,
                                cursor,
                                renderer,
                                clipboard,
                                shell,
                                viewport,
                            )
                        })
                        .fold(event::Status::Ignored, event::Status::merge)
                }
            }

            fn mouse_interaction(
                &self,
                tree: &tree::Tree,
                layout: layout::Layout<'_>,
                cursor: mouse::Cursor,
                viewport: &Rectangle,
                renderer: &Renderer,
            ) -> mouse::Interaction {
                let bounds = layout.bounds();
                let state = tree.state.downcast_ref::<State>();

                if cursor.is_over(bounds) && self.message.is_some() {
                    mouse::Interaction::Pointer
                } else if state.collapsed {
                    let tree = &tree.children[0];
                    let layout = layout.children().next().expect("Menu missing first child");
                    let child = self.children[0].as_widget();

                    child.mouse_interaction(tree, layout, cursor, viewport, renderer)
                } else {
                    self.children
                        .iter()
                        .zip(&tree.children)
                        .zip(layout.children())
                        .map(|((child, state), layout)| {
                            child
                                .as_widget()
                                .mouse_interaction(state, layout, cursor, viewport, renderer)
                        })
                        .max()
                        .unwrap_or_default()
                }
            }

            fn draw(
                &self,
                tree: &tree::Tree,
                renderer: &mut Renderer,
                theme: &Theme,
                style: &iced::advanced::renderer::Style,
                layout: layout::Layout<'_>,
                cursor: mouse::Cursor,
                viewport: &Rectangle,
            ) {
                if layout.bounds().intersection(viewport).is_none() {
                    return;
                };

                let bounds = layout.bounds();
                let state = tree.state.downcast_ref::<State>();

                if cursor.is_over(bounds) && self.message.is_some() {
                    let border = Border::default().rounded(8.0);
                    let background = theme.extended_palette().background.strong.color;

                    <Renderer as advanced::Renderer>::fill_quad(
                        renderer,
                        Quad {
                            bounds,
                            border,
                            ..Default::default()
                        },
                        Background::Color(background),
                    );
                }

                if state.collapsed {
                    let layout = layout.children().next().unwrap();
                    self.children[0].as_widget().draw(
                        &tree.children[0],
                        renderer,
                        theme,
                        style,
                        layout,
                        cursor,
                        viewport,
                    );
                } else {
                    for ((child, state), layout) in self
                        .children
                        .iter()
                        .zip(&tree.children)
                        .zip(layout.children())
                    {
                        child
                            .as_widget()
                            .draw(state, renderer, theme, style, layout, cursor, viewport);
                    }
                }
            }

            fn overlay<'b>(
                &'b mut self,
                tree: &'b mut tree::Tree,
                layout: layout::Layout<'_>,
                renderer: &Renderer,
                translation: Vector,
            ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
                advanced::overlay::from_children(
                    &mut self.children,
                    tree,
                    layout,
                    renderer,
                    translation,
                )
            }

            //fn operate(
            //    &self,
            //    tree: &mut tree::Tree,
            //    layout: layout::Layout<'_>,
            //    renderer: &Renderer,
            //    operation: &mut dyn Operation,
            //) {
            //    operation.container(None, layout.bounds(), &mut |operation| {
            //        self.children
            //            .iter()
            //            .zip(&mut tree.children)
            //            .zip(layout.children())
            //            .for_each(|((child, state), layout)| {
            //                child
            //                    .as_widget()
            //                    .operate(state, layout, renderer, operation);
            //            });
            //    });
            //}
        }

        impl<'a, Message> From<Item<'a, Message>> for Element<'a, Message, Theme, Renderer>
        where
            Message: Clone + 'a,
        {
            fn from(value: Item<'a, Message>) -> Self {
                Self::new(value)
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub(super) struct State {
            pub collapsed: bool,
        }

        impl State {
            fn new() -> Self {
                Self { collapsed: false }
            }
        }
    }

    mod sections {
        use super::*;

        pub struct Section<'a, Message>
        where
            Message: Clone,
        {
            spacing: f32,
            align: Horizontal,
            width: Length,
            height: Length,
            children: Vec<Element<'a, Message, Theme, Renderer>>,
        }

        impl<'a, Message> Section<'a, Message>
        where
            Message: Clone + 'a,
        {
            pub fn from_vec(children: Vec<Item<'a, Message>>) -> Self {
                Self {
                    spacing: 10.0,
                    align: Horizontal::Left,
                    width: Length::Shrink,
                    height: Length::Shrink,
                    children: children.into_iter().map(Into::into).collect(),
                }
            }

            pub fn width(mut self, width: impl Into<Length>) -> Self {
                self.width = width.into();
                self
            }

            pub fn height(mut self, height: impl Into<Length>) -> Self {
                self.height = height.into();
                self
            }

            pub fn align(mut self, align: impl Into<Horizontal>) -> Self {
                self.align = align.into();
                self
            }

            pub fn spacing(mut self, spacing: f32) -> Self {
                self.spacing = spacing;
                self
            }

            /// Requires `tree` to be a valid [`Section`] widget tree.
            pub(super) fn collapse(tree: &mut tree::Tree, collapse: bool) {
                tree.children.iter_mut().for_each(|child| {
                    let state = child.state.downcast_mut::<ItemState>();
                    state.collapsed = collapse
                });
            }
        }

        impl<'a, Message> Widget<Message, Theme, Renderer> for Section<'a, Message>
        where
            Message: Clone + 'a,
        {
            fn children(&self) -> Vec<tree::Tree> {
                self.children.iter().map(tree::Tree::new).collect()
            }

            fn diff(&self, tree: &mut tree::Tree) {
                tree.diff_children(&self.children)
            }

            fn size(&self) -> Size<Length> {
                Size {
                    width: self.width,
                    height: self.height,
                }
            }

            fn layout(
                &self,
                tree: &mut tree::Tree,
                renderer: &Renderer,
                limits: &layout::Limits,
            ) -> layout::Node {
                layout::flex::resolve(
                    layout::flex::Axis::Vertical,
                    renderer,
                    limits,
                    self.width,
                    self.height,
                    Padding::ZERO,
                    self.spacing,
                    self.align.into(),
                    &self.children,
                    &mut tree.children,
                )
            }

            fn on_event(
                &mut self,
                state: &mut tree::Tree,
                event: Event,
                layout: layout::Layout<'_>,
                cursor: iced::advanced::mouse::Cursor,
                renderer: &Renderer,
                clipboard: &mut dyn Clipboard,
                shell: &mut Shell<'_, Message>,
                viewport: &Rectangle,
            ) -> event::Status {
                self.children
                    .iter_mut()
                    .zip(&mut state.children)
                    .zip(layout.children())
                    .map(|((child, state), layout)| {
                        child.as_widget_mut().on_event(
                            state,
                            event.clone(),
                            layout,
                            cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        )
                    })
                    .fold(event::Status::Ignored, event::Status::merge)
            }

            fn mouse_interaction(
                &self,
                state: &tree::Tree,
                layout: layout::Layout<'_>,
                cursor: iced::advanced::mouse::Cursor,
                viewport: &Rectangle,
                renderer: &Renderer,
            ) -> mouse::Interaction {
                self.children
                    .iter()
                    .zip(&state.children)
                    .zip(layout.children())
                    .map(|((child, state), layout)| {
                        child
                            .as_widget()
                            .mouse_interaction(state, layout, cursor, viewport, renderer)
                    })
                    .max()
                    .unwrap_or_default()
            }

            fn draw(
                &self,
                tree: &tree::Tree,
                renderer: &mut Renderer,
                theme: &Theme,
                style: &iced::advanced::renderer::Style,
                layout: layout::Layout<'_>,
                cursor: iced::advanced::mouse::Cursor,
                viewport: &Rectangle,
            ) {
                if layout.bounds().intersection(viewport).is_none() {
                    return;
                }

                for ((child, state), layout) in self
                    .children
                    .iter()
                    .zip(&tree.children)
                    .zip(layout.children())
                {
                    child
                        .as_widget()
                        .draw(state, renderer, theme, style, layout, cursor, viewport);
                }
            }

            fn overlay<'b>(
                &'b mut self,
                tree: &'b mut tree::Tree,
                layout: layout::Layout<'_>,
                renderer: &Renderer,
                translation: Vector,
            ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
                advanced::overlay::from_children(
                    &mut self.children,
                    tree,
                    layout,
                    renderer,
                    translation,
                )
            }
        }

        impl<'a, Message> From<Section<'a, Message>> for Element<'a, Message, Theme, Renderer>
        where
            Message: Clone + 'a,
        {
            fn from(value: Section<'a, Message>) -> Self {
                Self::new(value)
            }
        }
    }

    mod overlay {
        use super::*;
        use iced::{
            advanced::{overlay, text::Text},
            alignment::{Horizontal, Vertical},
            widget::text::{LineHeight, Shaping, Wrapping},
        };

        pub struct Overlay<'a> {
            position: Point,
            height: f32,
            width: f32,
            state: &'a mut State,
        }

        impl<'a> Overlay<'a> {
            /// Creates a new overlay using the widget tree of a
            /// [`Collapsible`].
            pub fn new(state: &'a mut State, position: Point, width: f32) -> Self {
                Self {
                    width,
                    height: width,
                    position,
                    state,
                }
            }
        }

        impl<'a, Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'a>
        where
            Message: Clone + 'a,
        {
            fn on_event(
                &mut self,
                event: Event,
                layout: layout::Layout<'_>,
                cursor: advanced::mouse::Cursor,
                _renderer: &Renderer,
                _clipboard: &mut dyn Clipboard,
                _shell: &mut Shell<'_, Message>,
            ) -> event::Status {
                let bounds = layout.bounds();

                if !cursor.is_over(bounds) {
                    self.state.overlay_hovered = false;
                    return event::Status::Ignored;
                }

                self.state.overlay_hovered = true;

                if event == Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) {
                    let collapsed = !self.state.collapsed;
                    self.state.collapsed = collapsed;
                }

                event::Status::Ignored
            }

            fn mouse_interaction(
                &self,
                _layout: layout::Layout<'_>,
                _cursor: advanced::mouse::Cursor,
                _viewport: &Rectangle,
                _renderer: &Renderer,
            ) -> mouse::Interaction {
                mouse::Interaction::Pointer
            }

            fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
                let size = Size::new(self.width, self.height);

                let node = layout::Node::new(size);

                node.translate(Vector::new(self.position.x, self.position.y))
            }

            fn draw(
                &self,
                renderer: &mut Renderer,
                theme: &Theme,
                _style: &advanced::renderer::Style,
                layout: layout::Layout<'_>,
                _cursor: advanced::mouse::Cursor,
            ) {
                let bounds = layout.bounds();
                let palette = theme.extended_palette();

                let pair = if self.state.overlay_hovered {
                    palette.primary.weak
                } else {
                    palette.primary.strong
                };
                let alpha = 0.85;

                let background = Color {
                    a: alpha,
                    ..pair.color
                };

                let border = Border::default().rounded(self.width).width(0.0);

                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds,
                        border,
                        ..Default::default()
                    },
                    Background::Color(background),
                );

                let collapsed = self.state.collapsed;
                let icon = if collapsed { "c" } else { "e" };

                let color = Color {
                    a: alpha,
                    ..pair.text
                };

                let font = <Renderer as advanced::text::Renderer>::default_font(renderer);

                let icon = Text {
                    content: icon.to_string(),
                    size: (self.width * 0.8).into(),
                    bounds: bounds.size(),
                    font,
                    horizontal_alignment: Horizontal::Center,
                    vertical_alignment: Vertical::Center,
                    line_height: LineHeight::default(),
                    shaping: Shaping::Basic,
                    wrapping: Wrapping::None,
                };

                <Renderer as advanced::text::Renderer>::fill_text(
                    renderer,
                    icon,
                    bounds.center(),
                    color,
                    bounds,
                )
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum Kind {
        NoHeader,
        NoFooter,
        OnlyMain,
        All,
    }

    pub struct Collapsible<'a, Message>
    where
        Message: Clone,
    {
        kind: Kind,
        align: Horizontal,
        width: Length,
        height: Length,
        padding: Padding,
        children: Vec<Element<'a, Message, Theme, Renderer>>,
    }

    impl<'a, Message> Collapsible<'a, Message>
    where
        Message: Clone + 'a,
    {
        fn from_children(children: Vec<Element<'a, Message>>, kind: Kind) -> Self {
            Self {
                height: Length::Fill,
                width: Length::Shrink,
                padding: [20.0, 16.0].into(),
                align: Horizontal::Left,
                kind,
                children,
            }
        }

        fn empty_spaces() -> (Element<'a, Message>, Element<'a, Message>) {
            (
                Space::with_height(Length::FillPortion(1)).into(),
                Space::with_height(Length::FillPortion(6)).into(),
            )
        }

        pub fn new(
            header: Item<'a, Message>,
            main: Section<'a, Message>,
            footer: Section<'a, Message>,
        ) -> Self {
            let (one, two) = Self::empty_spaces();

            let children = vec![header.into(), one, main.into(), two, footer.into()];

            Self::from_children(children, Kind::All)
        }

        pub fn no_header(main: Section<'a, Message>, footer: Section<'a, Message>) -> Self {
            let (one, two) = Self::empty_spaces();
            let children = vec![one, main.into(), two, footer.into()];

            Self::from_children(children, Kind::NoHeader)
        }

        pub fn no_footer(header: Item<'a, Message>, main: Section<'a, Message>) -> Self {
            let (one, two) = Self::empty_spaces();
            let children = vec![header.into(), one, main.into(), two];

            Self::from_children(children, Kind::NoFooter)
        }

        pub fn only_main(main: Section<'a, Message>) -> Self {
            let (one, two) = Self::empty_spaces();

            Self::from_children(vec![one, main.into(), two], Kind::OnlyMain)
        }

        pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
            self.padding = padding.into();
            self
        }

        pub fn width(mut self, width: impl Into<Length>) -> Self {
            self.width = width.into();
            self
        }

        pub fn height(mut self, height: impl Into<Length>) -> Self {
            self.height = height.into();
            self
        }

        pub fn align(mut self, align: impl Into<Horizontal>) -> Self {
            self.align = align.into();
            self
        }

        /// Assumes `tree` is a valid [`Collapsible`] widget tree
        fn collapse(children: &mut [tree::Tree], kind: Kind, collapse: bool) {
            match kind {
                Kind::All => {
                    // Header
                    {
                        let header = children[0].state.downcast_mut::<ItemState>();
                        header.collapsed = collapse;
                    }

                    // main
                    {
                        let main = &mut children[2];
                        Section::<Message>::collapse(main, collapse);
                    }

                    // footer
                    {
                        let footer = &mut children[4];
                        Section::<Message>::collapse(footer, collapse);
                    }
                }
                Kind::NoHeader => {
                    {
                        let main = &mut children[1];
                        Section::<Message>::collapse(main, collapse);
                    }

                    {
                        let footer = &mut children[3];
                        Section::<Message>::collapse(footer, collapse);
                    }
                }
                Kind::NoFooter => {
                    {
                        let header = children[0].state.downcast_mut::<ItemState>();
                        header.collapsed = collapse;
                    }

                    {
                        let main = &mut children[2];
                        Section::<Message>::collapse(main, collapse);
                    }
                }
                Kind::OnlyMain => {
                    let main = &mut children[1];
                    Section::<Message>::collapse(main, collapse);
                }
            }
        }
    }

    impl<'a, Message> Widget<Message, Theme, Renderer> for Collapsible<'a, Message>
    where
        Message: Clone + 'a,
    {
        fn children(&self) -> Vec<tree::Tree> {
            self.children.iter().map(tree::Tree::new).collect()
        }

        fn diff(&self, tree: &mut tree::Tree) {
            tree.diff_children(&self.children)
        }

        fn size(&self) -> Size<Length> {
            Size::new(self.width, self.height)
        }

        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<State>()
        }

        fn state(&self) -> tree::State {
            tree::State::new(State::new())
        }

        fn mouse_interaction(
            &self,
            state: &tree::Tree,
            layout: layout::Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.children
                .iter()
                .zip(&state.children)
                .zip(layout.children())
                .map(|((child, state), layout)| {
                    child
                        .as_widget()
                        .mouse_interaction(state, layout, cursor, viewport, renderer)
                })
                .max()
                .unwrap_or_default()
        }

        fn on_event(
            &mut self,
            state: &mut tree::Tree,
            event: Event,
            layout: layout::Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) -> event::Status {
            let bounds = layout.bounds();
            let tree = state;
            let state = tree.state.downcast_mut::<State>();

            state.hovered = cursor.is_over(bounds);

            let status = self
                .children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .map(|((child, state), layout)| {
                    child.as_widget_mut().on_event(
                        state,
                        event.clone(),
                        layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    )
                })
                .fold(event::Status::Ignored, event::Status::merge);

            if status == event::Status::Captured {
                state.collapsed = false;
            }

            if state.collapsed {
                Self::collapse(&mut tree.children, self.kind, true);

                shell.invalidate_layout()
            } else {
                Self::collapse(&mut tree.children, self.kind, false);

                shell.invalidate_layout()
            }

            status
        }

        fn overlay<'b>(
            &'b mut self,
            state: &'b mut tree::Tree,
            layout: layout::Layout<'_>,
            renderer: &Renderer,
            translation: Vector,
        ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
            let tree = state;
            let state = tree.state.downcast_mut::<State>();

            let trees = tree.children.iter_mut();

            let children = self
                .children
                .iter_mut()
                .zip(layout.children())
                .zip(trees)
                .filter_map(|((child, layout), tree)| {
                    child
                        .as_widget_mut()
                        .overlay(tree, layout, renderer, translation)
                })
                .collect();

            let children = advanced::overlay::Group::with_children(children);

            let bounds = layout.bounds();
            //let width = (bounds.width * 0.20).clamp(25.0, 40.0);
            let width = 30.;
            let translation = {
                let other = Vector::new(bounds.width * 0.9, (bounds.height * 0.5) - (width * 0.5));
                other + translation
            };
            let position = bounds.position() + translation;

            let hovered = state.hovered;
            let overlay_hovered = state.overlay_hovered;
            let overlay = Overlay::new(state, position, width);

            let own = advanced::overlay::Element::new(Box::new(overlay));

            if !hovered && !overlay_hovered {
                return Some(children.overlay());
            }

            Some(children.push(own).overlay())
        }

        fn layout(
            &self,
            tree: &mut tree::Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            layout::flex::resolve(
                layout::flex::Axis::Vertical,
                renderer,
                limits,
                self.width,
                self.height,
                self.padding,
                0.0,
                self.align.into(),
                &self.children,
                &mut tree.children,
            )
        }

        fn draw(
            &self,
            tree: &tree::Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &iced::advanced::renderer::Style,
            layout: layout::Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            viewport: &Rectangle,
        ) {
            if layout.bounds().intersection(viewport).is_none() {
                return;
            }

            for ((child, state), layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
            {
                child
                    .as_widget()
                    .draw(state, renderer, theme, style, layout, cursor, viewport);
            }
        }
    }

    impl<'a, Message> From<Collapsible<'a, Message>> for Element<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
    {
        fn from(value: Collapsible<'a, Message>) -> Self {
            Self::new(value)
        }
    }

    #[derive(Debug)]
    struct State {
        collapsed: bool,
        hovered: bool,
        overlay_hovered: bool,
    }

    impl State {
        fn new() -> Self {
            Self {
                collapsed: false,
                hovered: false,
                overlay_hovered: false,
            }
        }
    }
}

mod context {}
