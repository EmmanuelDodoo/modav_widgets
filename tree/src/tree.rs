use iced::{
    advanced::{
        self,
        layout::{Layout, Limits, Node},
        mouse,
        renderer::Quad,
        text::{self, paragraph::Plain, LineHeight, Paragraph, Shaping, Wrapping},
        widget::{self, operation::Focusable, tree, Widget},
    },
    alignment::{self, Horizontal, Vertical},
    event::{self, Event},
    keyboard::{self, key::Named, Key},
    window, Color, Element, Length, Padding, Pixels, Point, Rectangle, Size,
};

use crate::style::*;
use lilt::{Animated, Easing};
use std::slice::IterMut;
use std::time::Instant;

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

/// A collapsible vertical tree widget
pub struct Tree<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: text::Renderer,
    Theme: Catalog,
{
    children: Vec<Element<'a, Message, Theme, Renderer>>,
    id: Option<widget::Id>,
    root: String,
    size: Option<Pixels>,
    icon: Option<Icon<Renderer::Font>>,
    font: Option<Renderer::Font>,
    padding: Padding,
    width: Length,
    height: Length,
    line_height: LineHeight,
    gap: f32,
    easing: Easing,
    duration: f32,
    class: Theme::Class<'a>,
    on_action: Option<Box<dyn Fn(Action) -> Message + 'a>>,
}

impl<'a, Message, Theme, Renderer> Tree<'a, Message, Theme, Renderer>
where
    Renderer: text::Renderer + 'a,
    Message: 'a,
    Theme: Catalog + 'a,
{
    /// Creates a new [`Tree`] widget with the given root value.
    pub fn new(root: impl Into<String>) -> Self {
        Self::with_children(root, std::iter::empty())
    }

    /// Creates a new [`Tree`] widget with the given root value and children subtrees.
    pub fn with_children(root: impl Into<String>, children: impl Iterator<Item = Self>) -> Self {
        let children = children.map(Element::from).collect();

        Self {
            children,
            root: root.into(),
            size: None,
            icon: None,
            font: None,
            id: None,
            padding: [5, 7].into(),
            width: Length::Shrink,
            height: Length::Shrink,
            gap: 10.0,
            line_height: LineHeight::default(),
            easing: Easing::EaseInOut,
            duration: 250.0,
            on_action: None,
            class: Theme::default(),
        }
    }

    /// Sets the [`Icon`] of the [`Tree`].
    pub fn icon(mut self, icon: Icon<Renderer::Font>) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the text size of the [`Tree`]'s root value.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the width of the [`Tree`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Tree`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the font of the [`Tree`].
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the [`Padding`] of the [`Tree`].
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the gap between subtrees in the [`Tree`].
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Sets the [`Easing`] function for animations on the [`Tree`].
    pub fn animation_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the duration for animations on the [`Tree`] in milliseconds.
    pub fn animation_duration(mut self, duration_ms: f32) -> Self {
        self.duration = duration_ms;
        self
    }

    /// Sets the message that should be produced when some action is performed
    /// in the [`Tree`].
    pub fn on_action(mut self, on_action: impl Fn(Action) -> Message + 'a) -> Self {
        self.on_action = Some(Box::new(on_action));
        self
    }

    /// Sets the [widget::Id] of the [Tree].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the style class of the [`Tree`].
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Sets the style of the [`Tree`].
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Tree<'_, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
    Theme: Catalog,
{
    fn size(&self) -> iced::Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new(
            self.easing,
            self.duration,
        ))
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.children.iter().map(tree::Tree::new).collect()
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(&self.children)
    }

    fn layout(&self, tree: &mut tree::Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let factor = 1.0 - state.animation.animate(std::convert::identity, state.now);

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let text_size = self.size.unwrap_or_else(|| renderer.default_size());

        let padding = self.padding;
        let height = self.line_height.to_absolute(text_size);
        let spacing = self.gap * factor;

        state.root.update(text::<Renderer>(
            &self.root,
            Size::new(f32::INFINITY, height.0),
            font,
            Horizontal::Left,
            self.line_height,
            text_size,
        ));

        let root = if let Some(icon) = &self.icon {
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
            let text_size = state.root.min_bounds();

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
            let text_size = state.root.min_bounds();
            let size = limits
                .resolve(self.width, self.height, text_size)
                .expand(padding);
            let text = Node::new(text_size).move_to(Point::new(padding.left, padding.top));

            Node::with_children(size, vec![text])
        };

        let base_size = root.size();
        let offset_x = (base_size.width * 0.3).min(40.0);

        let mut subs = vec![];
        let mut offset_y = 0.0;
        let mut subs_width = 0.0f32;

        let mut centers = vec![];

        for (child, tree) in self.children.iter().zip(tree.children.iter_mut()) {
            let node = child
                .as_widget()
                .layout(tree, renderer, limits)
                .move_to(Point::new(0.0, offset_y));

            let height = node.children()[0].size().height;

            centers.push(offset_y + (height * 0.5));

            let size = node.size();

            offset_y += size.height + spacing;

            subs_width = subs_width.max(size.width);

            subs.push(node)
        }

        let subs_height = (offset_y - spacing).max(0.0);
        let subs_size = Size::new(subs_width, subs_height);
        let subs = Node::with_children(subs_size, subs)
            .move_to(Point::new(offset_x, base_size.height + spacing));
        let subs_size = subs.size();
        let f_height = (spacing + subs_size.height) * factor;

        let links = {
            let thickness = 1.0;
            let stem_height = f_height;

            let size = Size::new(thickness, stem_height);

            let x = (base_size.width * 0.125).min(15.0);
            let stem = Node::new(size);

            let width = offset_x - x;
            let size = Size::new(width, thickness);

            let links = centers
                .into_iter()
                .map(|center| center + spacing)
                .map(|y| Node::new(size).move_to(Point::new(0.0, y - (thickness * 0.5))));

            let mut children = vec![stem];
            children.extend(links);

            Node::with_children(Size::new(width, stem_height), children)
                .move_to(Point::new(x, base_size.height))
        };

        let height = if self.children.is_empty() {
            base_size.height
        } else {
            base_size.height + f_height
        };
        let width = base_size.width.max(subs_size.width + offset_x);

        let intrinsic = Size::new(width, height);

        let size = limits.resolve(self.width, self.height, intrinsic);

        Node::with_children(size, vec![root, links, subs])
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let bounds = layout.bounds();

        let Some(viewport) = bounds.intersection(viewport) else {
            return;
        };

        let mut children = layout.children();

        let root = children.next().expect("Widget draw: Missing root layout");

        let status = if state.is_selected {
            Status::Active
        } else if cursor.is_over(root.bounds()) {
            Status::Hovered
        } else {
            Status::Idle
        };

        let own_style = theme.style(&self.class, status);

        if let Some(viewport) = root.bounds().intersection(&viewport) {
            renderer.fill_quad(
                Quad {
                    bounds: viewport,
                    border: own_style.border,
                    shadow: own_style.shadow,
                },
                own_style.background,
            );

            let mut base_children = root.children();

            let text = base_children
                .next()
                .expect("Widget draw: Missing root text layout");

            if let Some(clipped) = viewport.intersection(&text.bounds()) {
                draw(
                    renderer,
                    own_style.text_color,
                    text,
                    state.root.raw(),
                    &clipped,
                );
            }

            if self.icon.is_some() {
                let icon = base_children
                    .next()
                    .expect("Widget draw: Missing icon layout");

                if let Some(clipped) = viewport.intersection(&icon.bounds()) {
                    draw(
                        renderer,
                        own_style.text_color,
                        icon,
                        state.icon.raw(),
                        &clipped,
                    );
                }
            }
        }

        let links = children.next().expect("Widget draw: Missing links layout");
        if let Some(viewport) = links.bounds().intersection(&viewport) {
            for link in links.children() {
                if let Some(viewport) = link.bounds().intersection(&viewport) {
                    renderer.fill_quad(
                        Quad {
                            bounds: viewport,
                            ..Default::default()
                        },
                        own_style.text_color,
                    );
                }
            }
        }

        let subs = children
            .next()
            .expect("Widget draw: Missing subtrees layout");

        if let Some(viewport) = subs.bounds().intersection(&viewport) {
            self.children
                .iter()
                .zip(tree.children.iter())
                .zip(subs.children())
                .for_each(|((child, tree), layout)| {
                    child
                        .as_widget()
                        .draw(tree, renderer, theme, style, layout, cursor, &viewport);
                });
        }
    }

    fn on_event(
        &mut self,
        tree: &mut tree::Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let mut children = layout.children();

        let mut propagate = |layout: Option<Layout<'_>>,
                             shell: &mut advanced::Shell<'_, Message>| {
            layout
                .expect("Widget update: Missing subtree layouts")
                .children()
                .zip(self.children.iter_mut())
                .zip(tree.children.iter_mut())
                .enumerate()
                .fold(
                    (-1, event::Status::Ignored),
                    |(tab, acc), (idx, ((layout, sub), tree))| {
                        let status = sub.as_widget_mut().on_event(
                            tree,
                            event.clone(),
                            layout,
                            cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );

                        if acc == event::Status::Captured {
                            (tab, acc)
                        } else if status == event::Status::Captured {
                            (idx as i32, status)
                        } else {
                            (tab, acc)
                        }
                    },
                )
        };

        match &event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.focused = cursor.is_over(layout.bounds());

                let root = children.next().expect("Widget update: Missing root layout");
                let stem = children
                    .next()
                    .expect("Widget update: Missing links layout")
                    .children()
                    .next()
                    .expect("Widget update: Missing stem layout");

                if !state.collapsed {
                    match propagate(children.next(), shell) {
                        (tab, event::Status::Captured) => {
                            state.tab = tab;
                            if state.is_selected {
                                state.is_selected = false;
                                if let Some(on_action) = self.on_action.as_ref() {
                                    let msg = on_action(Action::Selected(state.is_selected));
                                    shell.publish(msg)
                                }
                            }
                            return event::Status::Captured;
                        }
                        (tab, event::Status::Ignored) => state.tab = tab,
                    };
                }

                if cursor.is_over(root.bounds()) {
                    state.collapsed = !state.collapsed;
                    state.is_dirty = true;
                    state.is_selected = true;
                    state.tab = 0;

                    if let Some(on_action) = self.on_action.as_ref() {
                        let msg = on_action(Action::Collapsed(state.collapsed));
                        let msg2 = on_action(Action::Selected(state.is_selected));

                        shell.publish(msg);
                        shell.publish(msg2);
                    }

                    shell.request_redraw(window::RedrawRequest::NextFrame);

                    return event::Status::Captured;
                }

                if cursor.is_over(stem.bounds()) {
                    state.collapsed = !state.collapsed;
                    state.is_dirty = true;
                    state.is_selected = true;
                    state.tab = 0;

                    if let Some(on_action) = self.on_action.as_ref() {
                        let msg = on_action(Action::Collapsed(state.collapsed));
                        let msg2 = on_action(Action::Selected(state.is_selected));

                        shell.publish(msg);
                        shell.publish(msg2);
                    }
                    shell.request_redraw(window::RedrawRequest::NextFrame);

                    return event::Status::Captured;
                }

                state.tab = -1;
                if state.is_selected {
                    state.is_selected = false;
                    if let Some(on_action) = self.on_action.as_ref() {
                        let msg = on_action(Action::Selected(state.is_selected));

                        shell.publish(msg);
                    }
                }

                event::Status::Ignored
            }
            Event::Window(window::Event::RedrawRequested(now)) if state.is_dirty => {
                state.now = *now;

                state
                    .animation
                    .transition(f32::from(state.collapsed), Instant::now());

                shell.invalidate_layout();

                if state.animation.in_progress(*now) {
                    shell.request_redraw(window::RedrawRequest::NextFrame);
                } else {
                    state.is_dirty = false;
                }

                let _base = children.next();
                let _links = children.next();

                if !state.collapsed {
                    let (_, status) = propagate(children.next(), shell);
                    status
                } else {
                    event::Status::Ignored
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::Tab),
                modifiers,
                ..
            }) if modifiers.shift() && state.focused => {
                let _base = children.next();
                let _links = children.next();

                let subtrees = children
                    .next()
                    .expect("Widget update: Missing subtree layouts");

                walk_up(
                    self,
                    state,
                    subtrees,
                    tree.children.iter_mut(),
                    event,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    true,
                    viewport,
                )
            }

            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Tab),
                ..
            }) if state.focused => {
                let _base = children.next();
                let _links = children.next();

                let subtrees = children
                    .next()
                    .expect("Widget update: Missing subtree layouts");

                walk_down(
                    self,
                    state,
                    subtrees,
                    tree.children.iter_mut(),
                    event,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    true,
                    viewport,
                )
            }

            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::ArrowUp),
                ..
            }) => {
                let _base = children.next();
                let _links = children.next();

                let subtrees = children
                    .next()
                    .expect("Widget update: Missing subtree layouts");

                let status = walk_up(
                    self,
                    state,
                    subtrees,
                    tree.children.iter_mut(),
                    event,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    false,
                    viewport,
                );

                status
            }

            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::ArrowDown),
                ..
            }) if state.focused => {
                let _base = children.next();
                let _links = children.next();

                let subtrees = children
                    .next()
                    .expect("Widget update: Missing subtree layouts");

                walk_down(
                    self,
                    state,
                    subtrees,
                    tree.children.iter_mut(),
                    event,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    false,
                    viewport,
                )
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::Enter),
                ..
            }) => {
                let _base = children.next();
                let _links = children.next();
                if !state.collapsed {
                    if let (tab, event::Status::Captured) = propagate(children.next(), shell) {
                        state.tab = tab;
                        return event::Status::Captured;
                    }
                }

                if state.is_selected {
                    state.collapsed = !state.collapsed;
                    state.is_dirty = true;

                    if let Some(on_action) = self.on_action.as_ref() {
                        let msg = on_action(Action::Collapsed(state.collapsed));
                        shell.publish(msg);
                    }

                    shell.request_redraw(window::RedrawRequest::NextFrame);

                    event::Status::Captured
                } else {
                    event::Status::Ignored
                }
            }
            _ => {
                let _base = children.next();
                let _links = children.next();

                if !state.collapsed {
                    let (_, status) = propagate(children.next(), shell);

                    status
                } else {
                    event::Status::Ignored
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &tree::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::default();
        }

        let mut children = layout.children();

        let root = children
            .next()
            .expect("Widget interaction: Missing root layout");

        if cursor.is_over(root.bounds()) {
            return mouse::Interaction::Pointer;
        }

        let _links = children.next();

        let subs = children
            .next()
            .expect("Widget Interaction: Missing subtree layout");

        subs.children()
            .zip(self.children.iter())
            .zip(tree.children.iter())
            .map(|((layout, sub), tree)| {
                sub.as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .fold(mouse::Interaction::default(), |acc, curr| {
                if acc == mouse::Interaction::default() {
                    curr
                } else {
                    acc
                }
            })
    }

    //fn operate(
    //    &self,
    //    tree: &mut tree::Tree,
    //    layout: Layout<'_>,
    //    renderer: &Renderer,
    //    operation: &mut dyn widget::Operation,
    //) {
    //    let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
    //
    //    operation.focusable(state, self.id.as_ref());
    //
    //    let mut children = layout.children();
    //    let _base = children.next();
    //    let _links = children.next();
    //
    //    let subs = children
    //        .next()
    //        .expect("Widget Operate: Missing subtree layouts");
    //
    //    let _ = subs
    //        .children()
    //        .zip(tree.children.iter_mut())
    //        .zip(self.children.iter())
    //        .for_each(|((layout, tree), sub)| {
    //            sub.as_widget().operate(tree, layout, renderer, operation)
    //        });
    //}
}

impl<'a, Message, Theme, Renderer> From<Tree<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
    Message: 'a,
{
    fn from(value: Tree<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// An interaction with a [`Tree`] widget.
pub enum Action {
    /// If true, the [`Tree`] is collapsed.
    Collapsed(bool),
    /// If true, the [`Tree`]'s root is selected.
    Selected(bool),
}

#[derive(Debug)]
struct State<P: text::Paragraph> {
    root: Plain<P>,
    icon: Plain<P>,
    is_dirty: bool,
    collapsed: bool,
    animation: Animated<f32, Instant>,
    now: Instant,
    is_selected: bool,
    tab: i32,
    focused: bool,
}

impl<P: text::Paragraph> State<P> {
    fn new(easing: Easing, duration: f32) -> Self {
        let collapsed = false;
        Self {
            root: Plain::default(),
            icon: Plain::default(),
            collapsed,
            is_dirty: false,
            focused: false,
            now: Instant::now(),
            tab: -1,
            is_selected: false,
            animation: Animated::new(f32::from(collapsed))
                .duration(duration)
                .easing(easing),
        }
    }
}

impl<P: text::Paragraph> Focusable for State<P> {
    fn focus(&mut self) {
        self.is_selected = true;
    }

    fn unfocus(&mut self) {
        self.is_selected = false;
    }

    fn is_focused(&self) -> bool {
        self.is_selected
    }
}

#[allow(clippy::too_many_arguments)]
fn walk_down<Message, Theme: Catalog, Renderer: text::Renderer>(
    tree: &mut Tree<'_, Message, Theme, Renderer>,
    state: &mut State<Renderer::Paragraph>,
    layout: Layout<'_>,
    trees: IterMut<'_, tree::Tree>,
    event: Event,
    cursor: mouse::Cursor,
    renderer: &Renderer,
    clipboard: &mut dyn advanced::Clipboard,
    shell: &mut advanced::Shell<'_, Message>,
    tab: bool,
    viewport: &Rectangle,
) -> event::Status {
    if state.tab <= -1 && !state.is_selected {
        state.is_selected = true;
        if let Some(on_action) = tree.on_action.as_ref() {
            let msg = on_action(Action::Selected(state.is_selected));

            shell.publish(msg);
        }
        state.tab = 0;
        return event::Status::Captured;
    }

    state.tab = state.tab.max(0);

    if state.is_selected {
        state.is_selected = false;
        if let Some(on_action) = tree.on_action.as_ref() {
            let msg = on_action(Action::Selected(state.is_selected));
            shell.publish(msg)
        }
    }

    let walk_collapsed = if tab { state.collapsed } else { false };

    if walk_collapsed || state.tab >= tree.children.len() as i32 {
        state.tab = -1;
        state.focused = false;
        return event::Status::Ignored;
    }

    if !walk_collapsed {
        state.collapsed = false;
        state.is_dirty = true;
        shell.request_redraw(window::RedrawRequest::NextFrame);
    }

    let mut subs = layout.children().zip(tree.children.iter_mut()).zip(trees);

    for _ in 0..state.tab {
        subs.next();
    }

    for ((layout, sub), tree) in subs {
        {
            let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
            state.focused = true;
        }

        let event::Status::Ignored = sub.as_widget_mut().on_event(
            tree,
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) else {
            return event::Status::Captured;
        };

        state.tab += 1;
    }

    state.tab = -1;
    state.focused = false;

    event::Status::Ignored
}

#[allow(clippy::too_many_arguments)]
fn walk_up<Message, Theme: Catalog, Renderer: text::Renderer>(
    tree: &mut Tree<'_, Message, Theme, Renderer>,
    state: &mut State<Renderer::Paragraph>,
    layout: Layout<'_>,
    trees: IterMut<'_, tree::Tree>,
    event: Event,
    cursor: mouse::Cursor,
    renderer: &Renderer,
    clipboard: &mut dyn advanced::Clipboard,
    shell: &mut advanced::Shell<'_, Message>,
    tab: bool,
    viewport: &Rectangle,
) -> event::Status {
    let len = tree.children.len() as i32;

    if state.tab == -2 || state.tab >= len || state.is_selected {
        state.tab = -1;
        state.focused = false;
        state.is_selected = false;
        if let Some(on_action) = tree.on_action.as_ref() {
            let msg = on_action(Action::Selected(state.is_selected));

            shell.publish(msg);
        }
        return event::Status::Ignored;
    }

    let length = if tab || (state.tab < len - 1 && state.tab != -1) {
        len
    } else {
        0
    };
    let diff = length - state.tab - 1;

    if !tab {
        state.collapsed = false;
        state.is_dirty = true;
        shell.request_redraw(window::RedrawRequest::NextFrame);
    }

    let layouts = layout.children().rev();
    let subs = tree.children.iter_mut().rev();
    let trees = trees.rev();

    let mut subs = layouts.zip(subs).zip(trees).enumerate();

    for _ in 0..diff {
        subs.next();
    }

    for (idx, ((layout, sub), tree)) in subs {
        {
            let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
            state.focused = true;
        }

        let event::Status::Ignored = sub.as_widget_mut().on_event(
            tree,
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) else {
            state.tab = len - (idx as i32) - 1;
            return event::Status::Captured;
        };
    }

    state.is_selected = true;
    if let Some(on_action) = tree.on_action.as_ref() {
        let msg = on_action(Action::Selected(state.is_selected));

        shell.publish(msg);
    }
    state.tab = -2;

    event::Status::Captured
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
