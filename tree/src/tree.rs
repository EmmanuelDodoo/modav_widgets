use iced::{
    advanced::{
        self,
        layout::{Layout, Limits, Node},
        mouse, overlay,
        renderer::Quad,
        widget::{self, operation::Focusable, tree, Widget},
    },
    event::{self, Event},
    keyboard::{self, key::Named, Key},
    window, Element, Length, Padding, Point, Rectangle, Size,
};

use crate::style::*;
use lilt::{Animated, Easing};
use std::slice::IterMut;
use std::time::Instant;

/// A collapsible vertical tree widget
pub struct Tree<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: advanced::Renderer,
    Theme: Catalog,
{
    children: Vec<Element<'a, Message, Theme, Renderer>>,
    id: Option<widget::Id>,
    width: Length,
    height: Length,
    padding: Padding,
    gap: f32,
    easing: Easing,
    duration: f32,
    class: Theme::Class<'a>,
    collapsed: bool,
    collapse_on_click: bool,
    on_action: Option<Box<dyn Fn(Action) -> Message + 'a>>,
}

impl<'a, Message, Theme, Renderer> Tree<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + 'a,
    Message: 'a,
    Theme: Catalog + 'a,
{
    /// Creates a new [`Tree`] widget with the given root value.
    pub fn new(root: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self::with_children(root, std::iter::empty())
    }

    /// Creates a new [`Tree`] widget with the given root value and children subtrees.
    pub fn with_children(
        root: impl Into<Element<'a, Message, Theme, Renderer>>,
        children: impl Iterator<Item = Self>,
    ) -> Self {
        let children = std::iter::once(root.into())
            .chain(children.map(Element::from))
            .collect();

        Self {
            children,
            id: None,
            width: Length::Shrink,
            height: Length::Shrink,
            gap: 10.0,
            padding: [3, 3].into(),
            easing: Easing::EaseInOut,
            duration: 250.0,
            collapsed: false,
            on_action: None,
            class: Theme::default(),
            collapse_on_click: true,
        }
    }

    /// Adds a sub-tree to the [`Tree`].  
    pub fn push_child(mut self, child: Self) -> Self {
        self.children.push(child.into());
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

    /// If `true`, the [`Tree`] starts out as collapsed.
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// Sets the gap between subtrees in the [`Tree`].
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Sets the collapsing behavior of the [`Tree`].
    ///
    /// When set to true, the [`Tree`] collapses when clicked on regardless of
    /// its prior selection state.
    pub fn collapse_on_click(mut self, collapse: bool) -> Self {
        self.collapse_on_click = collapse;
        self
    }

    /// Sets the padding on the root of the [`Tree`].
    ///
    /// Increasing this gives more room for the [`Tree`] to respond directly to
    /// an [`Event`] without going through the root element first
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
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
    Renderer: advanced::Renderer,
    Theme: Catalog,
{
    fn size(&self) -> iced::Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.collapsed, self.easing, self.duration))
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.children.iter().map(tree::Tree::new).collect()
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(&self.children)
    }

    fn layout(&self, tree: &mut tree::Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let state = tree.state.downcast_mut::<State>();
        let factor = 1.0 - state.animation.animate(std::convert::identity, state.now);

        let spacing = self.gap * factor;

        let root = self.children[0]
            .as_widget()
            .layout(
                &mut tree.children[0],
                renderer,
                &limits
                    .width(self.width)
                    .height(self.height)
                    .shrink(self.padding),
            )
            .move_to(Point::new(self.padding.left, self.padding.top));

        let root = Node::with_children(root.size().expand(self.padding), vec![root]);
        let base_size = root.size();
        let offset_x = (base_size.width * 0.3).min(40.0);

        let mut subs = vec![];
        let mut offset_y = 0.0;
        let mut subs_width = 0.0f32;

        let mut centers = vec![];

        for (child, tree) in self.children[1..].iter().zip(tree.children[1..].iter_mut()) {
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

        let height = if self.children.len() == 1 {
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
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let Some(viewport) = bounds.intersection(viewport) else {
            return;
        };

        let mut children = layout.children();

        let root = children
            .next()
            .expect("Widget draw: Missing paddded root layout");

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

            let root = root
                .children()
                .next()
                .expect("Tree draw: Missing root layout");

            self.children[0].as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                root,
                cursor,
                &viewport,
            )
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
            self.children[1..]
                .iter()
                .zip(tree.children[1..].iter())
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
        let state = tree.state.downcast_mut::<State>();
        let mut children = layout.children();
        let root = children
            .next()
            .expect("Widget update: Missing padded root layout");
        let base = root
            .children()
            .next()
            .expect("Tree update: Missing root layout");

        let root_status = self.children[0].as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            base,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if root_status == event::Status::Captured {
            state.focused = true;
            state.is_selected = true;
            state.tab = 0;
            unfocus_subtrees(tree.children[1..].iter_mut());

            if let Some(on_action) = self.on_action.as_ref() {
                let msg = on_action(Action::Selected(state.is_selected));

                shell.publish(msg);
            }

            return root_status;
        }

        let mut propagate = |layout: Option<Layout<'_>>,
                             shell: &mut advanced::Shell<'_, Message>| {
            layout
                .expect("Widget update: Missing subtree layouts")
                .children()
                .zip(self.children[1..].iter_mut())
                .zip(tree.children[1..].iter_mut())
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

                let can_collapse = self.collapse_on_click || state.is_selected;

                if cursor.is_over(root.bounds()) {
                    state.is_dirty = true;
                    state.is_selected = true;
                    state.tab = 0;
                    if can_collapse {
                        state.collapsed = !state.collapsed;
                    }

                    if let Some(on_action) = self.on_action.as_ref() {
                        if can_collapse {
                            let msg = on_action(Action::Collapsed(state.collapsed));
                            shell.publish(msg);
                        }

                        let msg2 = on_action(Action::Selected(state.is_selected));

                        shell.publish(msg2);
                    }

                    shell.request_redraw(window::RedrawRequest::NextFrame);

                    return event::Status::Captured;
                }

                if cursor.is_over(stem.bounds()) {
                    state.is_dirty = true;
                    state.is_selected = true;
                    state.tab = 0;
                    if can_collapse {
                        state.collapsed = !state.collapsed;
                    }

                    if let Some(on_action) = self.on_action.as_ref() {
                        if can_collapse {
                            let msg = on_action(Action::Collapsed(state.collapsed));
                            shell.publish(msg);
                        }
                        let msg2 = on_action(Action::Selected(state.is_selected));

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
                let _links = children.next();

                let subtrees = children
                    .next()
                    .expect("Widget update: Missing subtree layouts");

                walk_up(
                    self,
                    state,
                    subtrees,
                    tree.children[1..].iter_mut(),
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
                let _links = children.next();

                let subtrees = children
                    .next()
                    .expect("Widget update: Missing subtree layouts");

                walk_down(
                    self,
                    state,
                    subtrees,
                    tree.children[1..].iter_mut(),
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
                let _links = children.next();

                let subtrees = children
                    .next()
                    .expect("Widget update: Missing subtree layouts");

                let status = walk_up(
                    self,
                    state,
                    subtrees,
                    tree.children[1..].iter_mut(),
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
                let _links = children.next();

                let subtrees = children
                    .next()
                    .expect("Widget update: Missing subtree layouts");

                walk_down(
                    self,
                    state,
                    subtrees,
                    tree.children[1..].iter_mut(),
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
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::Escape),
                ..
            }) => {
                let _links = children.next();

                if !state.collapsed {
                    let (_, _) = propagate(children.next(), shell);
                }

                if state.focused {
                    state.focused = false;
                    state.is_selected = false;
                    state.tab = -1;

                    if let Some(on_action) = self.on_action.as_ref() {
                        let msg = on_action(Action::Selected(state.is_selected));

                        shell.publish(msg);
                    }

                    event::Status::Ignored
                } else {
                    event::Status::Ignored
                }
            }
            _ => {
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
            .expect("Widget interaction: Missing padded root layout");

        if cursor.is_over(root.bounds()) {
            let root = root
                .children()
                .next()
                .expect("Tree interaction: Missing root layout");
            return self.children[0].as_widget().mouse_interaction(
                &tree.children[0],
                root,
                cursor,
                viewport,
                renderer,
            );
        }

        let _links = children.next();

        let subs = children
            .next()
            .expect("Widget Interaction: Missing subtree layout");

        subs.children()
            .zip(self.children[1..].iter())
            .zip(tree.children[1..].iter())
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

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        let mut group = overlay::Group::new();

        let mut children = layout.children();
        let root = children
            .next()
            .expect("Tree overlay: Missing padded root layout")
            .children()
            .next()
            .expect("Tree overlay: Missing root layout");
        let _links = children.next();

        let subs = children
            .next()
            .expect("Tree overlay: Missing subtree layout")
            .children();

        let children = std::iter::once(root).chain(subs);

        for ((subtree, tree), layout) in self
            .children
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(children)
        {
            if let Some(overlay) =
                subtree
                    .as_widget_mut()
                    .overlay(tree, layout, renderer, translation)
            {
                group = group.push(overlay)
            }
        }

        Some(group.overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<Tree<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: advanced::Renderer + 'a,
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
struct State {
    is_dirty: bool,
    collapsed: bool,
    animation: Animated<f32, Instant>,
    now: Instant,
    is_selected: bool,
    tab: i32,
    focused: bool,
}

impl State {
    fn new(collapsed: bool, easing: Easing, duration: f32) -> Self {
        Self {
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

impl Focusable for State {
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
fn walk_down<Message, Theme: Catalog, Renderer: advanced::Renderer>(
    tree: &mut Tree<'_, Message, Theme, Renderer>,
    state: &mut State,
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

    if walk_collapsed || state.tab >= tree.children.len() as i32 - 1 {
        state.tab = -1;
        state.focused = false;
        return event::Status::Ignored;
    }

    if !walk_collapsed {
        state.collapsed = false;
        state.is_dirty = true;
        shell.request_redraw(window::RedrawRequest::NextFrame);
    }

    let mut subs = layout
        .children()
        .zip(tree.children[1..].iter_mut())
        .zip(trees);

    for _ in 0..state.tab {
        subs.next();
    }

    for ((layout, sub), tree) in subs {
        {
            let state = tree.state.downcast_mut::<State>();
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
fn walk_up<Message, Theme: Catalog, Renderer: advanced::Renderer>(
    tree: &mut Tree<'_, Message, Theme, Renderer>,
    state: &mut State,
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
    let len = tree.children.len() as i32 - 1;

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
    let subs = tree.children[1..].iter_mut().rev();
    let trees = trees.rev();

    let mut subs = layouts.zip(subs).zip(trees).enumerate();

    for _ in 0..diff {
        subs.next();
    }

    for (idx, ((layout, sub), tree)) in subs {
        {
            let state = tree.state.downcast_mut::<State>();
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

fn unfocus_subtrees(subs: IterMut<'_, tree::Tree>) {
    for tree in subs {
        let state = tree.state.downcast_mut::<State>();
        state.focused = false;
        state.is_selected = false;
        state.tab = -1;

        unfocus_subtrees(tree.children[1..].iter_mut())
    }
}
