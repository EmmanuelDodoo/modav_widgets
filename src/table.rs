#![allow(unused_imports, dead_code)]
use iced::{
    advanced::{
        self,
        layout::{self, Limits, Node},
        mouse::{self, click},
        renderer::Quad,
        text::{self, paragraph::Plain, LineHeight, Paragraph, Shaping, Wrapping},
        widget::tree::{self, Tag, Tree},
        Shell, Widget,
    },
    alignment::{self, Horizontal, Vertical},
    color, event, font, keyboard,
    time::{Duration, Instant},
    touch,
    widget::{Scrollable, TextInput},
    window, Background, Border, Color, Element, Event, Font, Length, Padding, Pixels, Point,
    Rectangle, Renderer, Size, Theme, Vector,
};

use modav_core::repr::{
    col_sheet::{CellRef, Column, ColumnSheet, DataType, Error},
    ColumnHeader, ColumnType, Config,
};

mod utils;
use utils::{Editor, Resizing, Selection};

const BACK: &str = "‹ Back";
const NEXT: &str = "Next ›";
const GOTO_PAGE: &str = "Page";
const GOTO_GO: &str = "Go";
const PAGINATION_ELLIPSIS: &str = "•••";
const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;

type Cell = Plain<iced_graphics::text::Paragraph>;

pub struct Table {
    config: Config<String>,
    width: Length,
    height: Length,
    text_size: Pixels,
    font: Font,
    spacing: f32,
    padding: Padding,
    cell_padding: Padding,
}

impl Table {
    pub fn new(config: Config<String>) -> Self {
        Self {
            config,
            width: Length::Shrink,
            height: Length::Shrink,
            text_size: 16.0.into(),
            padding: [10, 15].into(),
            cell_padding: [2, 5].into(),
            font: Font::default(),
            spacing: 20.0,
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

    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = size.into();
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn cell_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.cell_padding = padding.into();
        self
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Table {
    fn tag(&self) -> Tag {
        Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(&self.config, self.font, self.text_size))
    }

    fn size(&self) -> iced::Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&self, tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        let state = tree.state.downcast_mut::<State>();

        state.layout(
            *limits,
            self.width,
            self.height,
            self.font,
            self.padding,
            self.cell_padding,
            self.text_size,
            self.spacing,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        //let style = theme.style(&self.class);

        let Some(clipped_viewport) = bounds.intersection(viewport) else {
            return;
        };

        <Renderer as advanced::Renderer>::fill_quad(
            renderer,
            Quad {
                bounds,
                ..Default::default()
            },
            Background::Color(color!(105, 135, 141)),
        );

        match state {
            State::Invalid { message, .. } => {
                draw(
                    renderer,
                    style,
                    layout,
                    message.raw(),
                    self.padding,
                    &clipped_viewport,
                );
            }
            State::Valid(internal) => internal.draw(
                renderer,
                layout,
                style,
                self.padding,
                self.cell_padding,
                self.spacing,
                &clipped_viewport.shrink(self.padding),
            ),
        }
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> advanced::mouse::Interaction {
        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::None;
        }

        let state = state.state.downcast_ref::<State>();

        match state {
            State::Invalid { .. } => mouse::Interaction::None,
            State::Valid(internal) => internal.mouse_interaction(layout, cursor),
        }
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: iced::Event,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let state = state.state.downcast_mut::<State>();

        match state {
            State::Invalid { .. } => event::Status::Ignored,
            State::Valid(internal) => internal.on_update(
                event,
                layout,
                cursor,
                self.padding,
                self.cell_padding,
                self.font,
                self.text_size,
                self.spacing,
                shell,
            ),
        }
    }
}

impl<'a, Message> From<Table> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
{
    fn from(value: Table) -> Self {
        Element::new(value)
    }
}

struct Internal {
    raw: ColumnSheet,
    cells: Vec<Cell>,
    numbering: Vec<Cell>,
    headers: Vec<(Cell, Cell)>,
    paginations: Vec<(Cell, String)>,
    page_next: Cell,
    page_back: Cell,
    goto_input: (Cell, String),
    goto_page: Cell,
    goto_go: Cell,
    status: (Cell, String),
    pages_padding: Padding,
    rows: usize,
    cols: usize,
    page: usize,
    page_limit: usize,
    page_size: Pixels,
    pages_gap: f32,
    cursor: utils::Cursor,
    is_focused: Option<Focus>,
    last_click: Option<mouse::Click>,
    keyboard_modifiers: keyboard::Modifiers,
    is_text_dragging: bool,
    editing: Option<Editing>,
    scroll_offset: Vector,
    cells_dim: Size,
    min_widths: Vec<f32>,
    min_heights: Vec<f32>,
    resizing: Option<Resizing>,
    selection: Option<Selection>,
}

impl Internal {
    /// The maximum number of items on a page. Includes the header row
    const PAGE_LIMIT: usize = 25;
    /// The maximum number of page numbers displayed
    const PAGINATION_LIMIT: usize = 11;
    /// The maximum size of a cell
    const MAX_CELL: Size = Size::new(f32::INFINITY, 45.0);
    /// Multiplier for each scroll step
    const SCROLL_MULT: f32 = 5.0;
    /// Spacing between cells
    const CELL_GAP: f32 = 3.0;
    /// Multiplier for column kind text size.
    const KIND_MULT: f32 = 0.9;

    fn new(raw: ColumnSheet, font: Font, size: Pixels) -> Self {
        let pages_padding = Padding::from([2, 6]);
        let size = size * 7.0 / 8.0;

        let dimensions = (raw.height(), raw.width());

        let headers = vec![(Cell::default(), Cell::default()); raw.headers().len()];

        let limit = Self::PAGE_LIMIT.min(dimensions.0);
        let min_widths = vec![0.0f32; dimensions.1 + 1];
        let min_heights = vec![0.0f32; limit + 1];

        let numbering = vec![Cell::default(); limit + 1];
        let pages_end = if limit == 0 {
            0
        } else {
            (raw.height() / limit) + 1
        };
        let paginations =
            vec![(Cell::default(), String::default()); Self::PAGINATION_LIMIT.min(pages_end)];

        let status = {
            let value = format!("{} rows × {} columns", dimensions.0, dimensions.1);
            let text = text(&value, Self::MAX_CELL, font, Horizontal::Left, size);
            (Cell::new(text), value)
        };

        let cells = vec![Cell::default(); limit * dimensions.1];

        let back = {
            let text = text(BACK, Self::MAX_CELL, font, Horizontal::Center, size);
            Cell::new(text)
        };

        let next = {
            let text = text(NEXT, Self::MAX_CELL, font, Horizontal::Center, size);
            Cell::new(text)
        };

        let goto_page = {
            let text = text(GOTO_PAGE, Self::MAX_CELL, font, Horizontal::Center, size);
            Cell::new(text)
        };

        let goto_go = {
            let text = text(GOTO_GO, Self::MAX_CELL, font, Horizontal::Center, size);
            Cell::new(text)
        };

        let goto_input = {
            let value = String::from("1");
            let text = text(&value, Self::MAX_CELL, font, Horizontal::Center, size);
            (Cell::new(text), value)
        };

        Self {
            raw,
            rows: dimensions.0,
            cols: dimensions.1,
            page: 0,
            cells,
            paginations,
            pages_padding,
            page_size: size,
            page_back: back,
            page_next: next,
            status,
            page_limit: limit,
            pages_gap: 5.0,
            goto_input,
            goto_go,
            goto_page,
            cursor: utils::Cursor::default(),
            is_focused: None,
            last_click: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_text_dragging: false,
            editing: None,
            scroll_offset: Vector::ZERO,
            cells_dim: Size::ZERO,
            numbering,
            headers,
            min_widths,
            min_heights,
            resizing: None,
            selection: None,
        }
    }

    /// Ending page
    fn pages_end(&self) -> usize {
        if self.page_limit == 0 {
            return 0;
        }
        self.raw.height() / self.page_limit
    }

    fn is_focused(&self) -> bool {
        self.is_focused.is_some()
    }

    /// Resets both editing and resizing
    fn reset(&mut self) {
        self.reset_resizing();
        self.reset_editing();
    }

    fn reset_editing(&mut self) {
        self.is_focused = None;
        self.is_text_dragging = false;
        self.last_click = None;
        self.editing = None;
        self.cursor = utils::Cursor::default();
        self.keyboard_modifiers = keyboard::Modifiers::default()
    }

    fn reset_resizing(&mut self) {
        self.resizing = None;
    }

    fn scroll_cells(&mut self, viewport: Size, offset: Vector) {
        let offset = offset * Self::SCROLL_MULT;
        let new = self.scroll_offset + offset;

        let width_diff = (viewport.width - self.cells_dim.width).min(0.0);
        let height_diff = (viewport.height - self.cells_dim.height).min(0.0);

        self.scroll_offset =
            Vector::new(new.x.clamp(width_diff, 0.0), new.y.clamp(height_diff, 0.0));
    }

    fn multiple_pages(&self) -> bool {
        self.rows > self.page_limit
    }

    fn layout_cells(&mut self, font: Font, padding: Padding, size: Pixels) -> Node {
        let gap = Self::CELL_GAP;
        // Adds numbering column
        let dimensions = (self.rows, self.cols + 1);
        // Adds headers row
        let page_limit = self.page_limit + 1;

        let numbering_max = dimensions.0;
        let numbering_max = Cell::new(text(
            &numbering_max.to_string(),
            Self::MAX_CELL,
            font,
            Horizontal::Right,
            size,
        ))
        .min_bounds()
        .expand(padding);

        let total = dimensions.1 * page_limit;
        let mut knds_height = vec![];
        let mut curr = 0;

        // Prep stage. Fill the paragraphs, register the dimensions
        while curr < total {
            let row = curr % page_limit;
            let column = curr / page_limit;

            let size = if column != 0 {
                let column = column - 1;
                let col = self.raw.get_col(column).expect("Missing column in sheet");
                let kind = col.kind();
                let horizontal = type_alignment(kind);

                if row == 0 {
                    let (header, knd) = &mut self.headers[column];
                    let label = match self.editing.as_ref() {
                        Some(Editing::Cell {
                            index,
                            value,
                            is_header: true,
                            ..
                        }) if *index == column => value,
                        _ => &col.label().map(ToOwned::to_owned).unwrap_or_default(),
                    };
                    let kind = kind.to_string();

                    let font = Font {
                        style: font::Style::Normal,
                        ..font
                    };
                    let text = text(&label, Self::MAX_CELL, font, Horizontal::Center, size);
                    header.update(text);
                    let font = Font {
                        style: font::Style::Italic,
                        ..font
                    };
                    let text = self::text(
                        &kind,
                        Self::MAX_CELL,
                        font,
                        Horizontal::Center,
                        size * Self::KIND_MULT,
                    );
                    knd.update(text);

                    let header = header.min_bounds();
                    let knd = knd.min_bounds();

                    knds_height.push(knd.height);
                    Size::new(header.width.max(knd.width), header.height + knd.height)
                } else {
                    let row = row - 1;
                    let idx = (column * self.page_limit) + (row % self.page_limit);
                    let paragraph = &mut self.cells[idx];
                    let row = row + (self.page * (page_limit - 1));

                    let value = match self.editing.as_ref() {
                        Some(Editing::Cell {
                            index,
                            value,
                            is_header: false,
                            ..
                        }) if *index == idx => value,
                        _ => &col
                            .data_ref(row)
                            .map(|cell| cell_to_string(cell))
                            .unwrap_or_default(),
                    };

                    let text = text(&value, Self::MAX_CELL, font, horizontal, size);
                    paragraph.update(text);

                    paragraph.min_bounds()
                }
            } else if row != 0 {
                let paragraph = &mut self.numbering[row];
                let row = (row - 1) + (self.page_limit * self.page);
                let font = Font {
                    style: font::Style::Italic,
                    ..font
                };

                paragraph.update(text(
                    &row.to_string(),
                    Self::MAX_CELL,
                    font,
                    Horizontal::Right,
                    size,
                ));

                paragraph.min_bounds()
            } else {
                Size::ZERO
            }
            .expand(padding);

            let height = self.min_heights[row].max(size.height);
            self.min_heights[row] = height;

            let width = if column == 0 {
                numbering_max.width
            } else {
                self.min_widths[column].max(size.width)
            };
            self.min_widths[column] = width;

            curr += 1;
        }

        curr = 0;

        let mut offset_width = 0.0;
        let mut offset_height = 0.0;
        let mut headers_x = 0.0;
        let mut numbering_y = 0.0;
        let mut children = vec![];
        let mut headers = vec![];
        let mut numbering = vec![];

        // Create the layout nodes
        while curr < total {
            let row = curr % page_limit;
            let column = curr / page_limit;

            if column != 0 {
                if row == 0 {
                    let height = self.min_heights[row];
                    let width = self.min_widths[column];
                    let knd_height = knds_height[column - 1];
                    let label = Size::new(
                        width - padding.horizontal(),
                        height - padding.vertical() - knd_height,
                    );
                    let knd = Size::new(width - padding.horizontal(), knd_height);
                    let knd = Node::new(knd).translate([padding.left, label.height + padding.top]);
                    let label = Node::new(label).translate([padding.left, padding.top]);

                    let size = Size::new(width, height);
                    let node = Node::with_children(size, vec![label, knd]);

                    let size = size + Size::from([gap, gap]);
                    let node = Node::with_children(size, vec![node]).translate([headers_x, 0.0]);

                    headers_x += size.width;
                    headers.push(node);
                } else {
                    let size = Size::new(self.min_widths[column], self.min_heights[row]);
                    let node = Node::new(size);

                    let size = size + Size::from([gap, gap]);
                    let node = Node::with_children(size, vec![node])
                        .translate([offset_width, offset_height]);

                    if (curr + 1) / page_limit == column {
                        offset_height += size.height;
                    } else {
                        offset_height = 0.0;
                        offset_width += size.width;
                    }

                    children.push(node);
                }
            } else {
                let size = Size::new(self.min_widths[column], self.min_heights[row]);
                let node = Node::new(size);

                let size = size + Size::from([gap, gap]);
                let node = Node::with_children(size, vec![node]).translate([0.0, numbering_y]);
                numbering_y += size.height;
                numbering.push(node);
            }

            curr += 1;
        }

        let numbering = {
            let width = numbering
                .first()
                .map(|node| node.size().width)
                .unwrap_or_default();
            let height = numbering_y;

            let size = Size::new(width, height);

            Node::with_children(size, numbering).translate([0.0, self.scroll_offset.y])
        };

        let headers = {
            let width = headers_x;
            let height = headers
                .first()
                .map(|node| node.size().height)
                .unwrap_or_default();

            let size = Size::new(width, height);
            Node::with_children(size, headers)
                .translate([numbering.size().width + self.scroll_offset.x, 0.])
        };

        let total_height = self
            .min_heights
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx != 0)
            .fold(0.0, |acc, (_, curr)| acc + curr + gap);

        let total_width = self
            .min_widths
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx != 0)
            .fold(0.0, |acc, (_, curr)| acc + curr + gap);

        let size = Size::new(total_width, total_height);
        self.cells_dim = size;

        let cells = Node::with_children(size, children).translate(Vector::new(
            self.scroll_offset.x + numbering.size().width,
            self.scroll_offset.y + headers.size().height,
        ));

        let size = {
            let width = numbering.size().width + headers.size().width.max(cells.size().width);
            let height = (headers.size().height + cells.size().height).max(numbering.size().height);

            Size::new(width, height)
        };
        Node::with_children(size, vec![numbering, headers, cells])
    }

    fn layout_pagination(&mut self, font: Font) -> Node {
        if self.raw.is_empty() {
            return Node::with_children(Size::ZERO, vec![Node::default(); 3]);
        }

        let gap = self.pages_gap;
        let pages_end = self.pages_end() + 1;
        let current_page = self.page + 1;

        let pages = if pages_end <= Self::PAGINATION_LIMIT {
            (1..=pages_end)
                .map(|num| num.to_string())
                .collect::<Vec<String>>()
        } else {
            gen_pagination(1, pages_end as isize, current_page as isize)
        };

        let mut min_bounds = Size::INFINITY;

        // Update paragraphs, register min width
        for (page, (cell, content)) in pages.into_iter().zip(self.paginations.iter_mut()) {
            let text = text(
                &page,
                Self::MAX_CELL,
                font,
                Horizontal::Center,
                self.page_size,
            );
            cell.update(text);
            *content = page;

            let size = cell.min_bounds().expand(self.pages_padding);

            min_bounds = min_bounds.min(size);
        }

        let back = self.page_back.min_bounds().expand(self.pages_padding);
        let next = self.page_next.min_bounds().expand(self.pages_padding);

        let mut pages = vec![];
        let mut offset = 0.0;

        // Create layout nodes
        for _ in 0..self.paginations.len() {
            let node = Node::new(min_bounds).translate(Vector::new(offset, 0.0));

            pages.push(node);
            offset += min_bounds.width + gap;
        }

        let mut total_width = min_bounds.width * pages.len() as f32;

        if !pages.is_empty() {
            total_width += gap * (pages.len() - 1) as f32;
        }

        let mut offset_x = 0.0;
        let offset_y = 0.0;

        let back = Node::new(back).translate(Vector::new(offset_x, offset_y));

        offset_x += back.size().width + (gap * 1.5);

        let pages = Node::with_children(Size::new(total_width, min_bounds.height), pages)
            .translate(Vector::new(offset_x, offset_y));

        offset_x += pages.size().width + (gap * 1.5);

        let next = Node::new(next).translate(Vector::new(offset_x, offset_y));

        offset_x += next.size().width;

        let total_size = Size::new(
            offset_x,
            back.size()
                .height
                .max(pages.size().height.max(next.size().height)),
        );

        Node::with_children(total_size, vec![back, pages, next])
    }

    fn layout_goto(&mut self, font: Font) -> Node {
        if self.raw.is_empty() {
            return Node::with_children(Size::ZERO, vec![Node::default(); 3]);
        }
        let max = Cell::new(text(
            &(self.pages_end() + 1).to_string(),
            Self::MAX_CELL,
            font,
            Horizontal::Right,
            self.page_size,
        ));

        let page = self.goto_page.min_bounds().expand(self.pages_padding);
        let go = self.goto_go.min_bounds().expand(self.pages_padding);

        let (input, value) = &mut self.goto_input;

        input.update(text(
            &value,
            Self::MAX_CELL,
            font,
            Horizontal::Right,
            self.page_size,
        ));

        let min_bounds = max.min_bounds();
        let input =
            Size::new(min_bounds.width.max(35.0), min_bounds.height).expand(self.pages_padding);

        let mut offset = 0.0;

        let page = Node::new(page).translate(Vector::new(offset, 0.0));

        offset += page.size().width + (self.pages_gap * 1.5);

        let input = Node::new(input).translate(Vector::new(offset, 0.0));

        offset += input.size().width + (self.pages_gap * 1.5);

        let go = Node::new(go).translate(Vector::new(offset, 0.0));

        offset += go.size().width;

        let total_size = Size::new(offset, page.size().max(input.size().max(go.size())).height);

        Node::with_children(total_size, vec![page, input, go])
    }

    fn layout_status(&mut self, font: Font, max_width: f32) -> Node {
        if self.raw.is_empty() {
            return Node::default();
        }

        let bounds = Size::new(max_width, f32::INFINITY);
        let (cell, value) = &mut self.status;

        cell.update(text(
            &value,
            bounds,
            font,
            Horizontal::Center,
            self.page_size,
        ));

        let size = cell.min_bounds().expand(self.pages_padding);

        Node::new(size)
    }

    fn layout(
        &mut self,
        limits: Limits,
        width: Length,
        height: Length,
        font: Font,
        padding: Padding,
        cell_padding: Padding,
        size: Pixels,
        spacing: f32,
    ) -> Node {
        let spacing = if self.raw.is_empty() { 0.0 } else { spacing };

        let content_limits = limits.width(width).height(height).shrink(padding);

        let mut pagination = if self.multiple_pages() {
            self.layout_pagination(font)
        } else {
            Node::default()
        };
        let pagination_size = pagination.size();

        let mut goto = if self.multiple_pages() {
            self.layout_goto(font)
        } else {
            Node::default()
        };
        let goto_size = goto.size();

        let actions = Size::new(
            pagination_size.width + spacing + goto_size.width,
            pagination_size.height.max(goto_size.height),
        );

        let actions_spacing = if self.multiple_pages() { spacing } else { 0.0 };

        let mut status = self.layout_status(font, content_limits.max().width);
        let status_size = status.size();
        status.translate_mut(Vector::new(
            padding.left,
            padding.top + actions.height + actions_spacing,
        ));

        let cells = self
            .layout_cells(font, cell_padding, size)
            .translate(Vector::new(
                padding.left,
                padding.top + actions.height + actions_spacing + status_size.height + spacing,
            ));
        let cells_size = cells.size();

        let total_size = Size::new(
            actions.width.max(cells_size.width),
            actions.height + actions_spacing + status_size.height + spacing + cells_size.height,
        )
        .expand(padding);

        let size = limits.resolve(width, height, total_size);

        let sum = pagination_size.width + spacing + goto_size.width;
        let diff = (size.width - sum) * 0.5;

        let mut offset_x = diff;
        let offset_y = padding.top;

        pagination.translate_mut([offset_x, offset_y]);
        offset_x += pagination_size.width + spacing;
        goto.translate_mut([offset_x, offset_y]);

        let children = vec![cells, status, pagination, goto];

        Node::with_children(size, children)
    }

    fn draw_pages(
        &self,
        renderer: &mut Renderer,
        layout: layout::Layout<'_>,
        style: &iced::advanced::renderer::Style,
        viewport: &Rectangle,
    ) {
        for ((cell, content), layout) in self.paginations.iter().zip(layout.children()) {
            let bounds = layout.bounds();
            let color = if (self.page + 1).to_string() != *content {
                color!(25, 155, 175)
            } else {
                color!(195, 205, 205)
            };
            if let Some(clipped_viewport) = bounds.intersection(viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds: clipped_viewport,
                        ..Default::default()
                    },
                    Background::Color(color),
                );

                draw(
                    renderer,
                    style,
                    layout,
                    cell.raw(),
                    self.pages_padding,
                    &clipped_viewport,
                )
            }
        }
    }

    fn draw_pagination(
        &self,
        renderer: &mut Renderer,
        layout: layout::Layout<'_>,
        style: &iced::advanced::renderer::Style,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        {
            let back = children.next().expect("Missing paginations: Back");
            if let Some(bounds) = back.bounds().intersection(viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds,
                        ..Default::default()
                    },
                    Background::Color(color!(25, 155, 175)),
                );
                draw(
                    renderer,
                    style,
                    back,
                    self.page_back.raw(),
                    self.pages_padding,
                    viewport,
                );
            }
        };

        let pages = children.next().expect("Missing paginations: Pages");

        self.draw_pages(renderer, pages, style, viewport);

        {
            let next = children.next().expect("Missing paginations: Next");
            if let Some(bounds) = next.bounds().intersection(viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds,
                        ..Default::default()
                    },
                    Background::Color(color!(25, 155, 175)),
                );
                draw(
                    renderer,
                    style,
                    next,
                    self.page_next.raw(),
                    self.pages_padding,
                    viewport,
                );
            }
        }
    }

    fn draw_goto(
        &self,
        renderer: &mut Renderer,
        layout: layout::Layout<'_>,
        style: &iced::advanced::renderer::Style,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        {
            let page = children.next().expect("Widget draw: Missing Goto Page");

            if let Some(bounds) = page.bounds().intersection(viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds,
                        ..Default::default()
                    },
                    Background::Color(color!(25, 155, 175)),
                );
                draw(
                    renderer,
                    style,
                    page,
                    self.goto_page.raw(),
                    self.pages_padding,
                    viewport,
                );
            }
        }

        {
            let input = children.next().expect("Widget draw: Missing Goto Input");

            if let Some(bounds) = input.bounds().intersection(viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds,
                        ..Default::default()
                    },
                    Background::Color(color!(25, 155, 175)),
                );
                draw(
                    renderer,
                    style,
                    input,
                    self.goto_input.0.raw(),
                    self.pages_padding,
                    viewport,
                );
            }
        }

        {
            let go = children.next().expect("Widget draw: Missing Goto Go");

            if let Some(bounds) = go.bounds().intersection(viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds,
                        ..Default::default()
                    },
                    Background::Color(color!(25, 155, 175)),
                );
                draw(
                    renderer,
                    style,
                    go,
                    self.goto_go.raw(),
                    self.pages_padding,
                    viewport,
                );
            }
        }
    }

    fn draw_status(
        &self,
        renderer: &mut Renderer,
        layout: layout::Layout<'_>,
        style: &iced::advanced::renderer::Style,
        viewport: &Rectangle,
    ) {
        if let Some(bounds) = layout.bounds().intersection(viewport) {
            <Renderer as advanced::Renderer>::fill_quad(
                renderer,
                Quad {
                    bounds,
                    ..Default::default()
                },
                Background::Color(color!(25, 155, 175)),
            );

            draw(
                renderer,
                style,
                layout,
                self.status.0.raw(),
                self.pages_padding,
                viewport,
            )
        }
    }

    fn draw_cells(
        &self,
        renderer: &mut Renderer,
        layout: layout::Layout<'_>,
        style: &iced::advanced::renderer::Style,
        viewport: Rectangle,
        padding: Padding,
    ) {
        if let Some(clipped) = layout.bounds().intersection(&viewport) {
            <Renderer as advanced::Renderer>::fill_quad(
                renderer,
                Quad {
                    bounds: clipped,
                    ..Default::default()
                },
                Background::Color(color!(125, 185, 105)),
            );
        }

        let mut editing: Option<Rectangle> = None;

        let mut children = layout.children();
        let numbering = children
            .next()
            .expect("Widget draw: Missing numbering cells");
        let headers = children.next().expect("Widget draw: Missing header cells");
        let cells = children.next().expect("Widget draw: Missing cells layout");

        let numbering_viewport = {
            let moved = viewport + Vector::new(0.0, headers.bounds().height);

            let size = Size::new(moved.width, moved.height - headers.bounds().height);

            Rectangle::new(moved.position(), size)
        };

        for (number, layout) in self.numbering.iter().zip(numbering.children()) {
            let bounds = layout.bounds();

            if let Some(clipped_viewport) = bounds.intersection(&numbering_viewport) {
                let child = layout
                    .children()
                    .next()
                    .expect("Table draw: Resize node missing child layout");

                if let Some(clipped_viewport) = child.bounds().intersection(&clipped_viewport) {
                    <Renderer as advanced::Renderer>::fill_quad(
                        renderer,
                        Quad {
                            bounds: clipped_viewport,
                            ..Default::default()
                        },
                        Background::Color(color!(25, 155, 175)),
                    );

                    draw(
                        renderer,
                        style,
                        child,
                        number.raw(),
                        padding,
                        &clipped_viewport,
                    )
                }
            }
        }

        let viewport = {
            let moved = viewport + Vector::new(numbering.bounds().width, 0.0);

            let size = Size::new(moved.width - numbering.bounds().width, moved.height);

            Rectangle::new(moved.position(), size)
        };

        let header_viewport = viewport;

        for (idx, ((header, kind), layout)) in
            self.headers.iter().zip(headers.children()).enumerate()
        {
            let pair = layout
                .children()
                .next()
                .expect("Table draw: Resize node missing pair layout");

            let mut children = pair.children();
            let label = children
                .next()
                .expect("Table draw: Pair node missing label layout");
            let knd = children
                .next()
                .expect("Table draw: Pair node missing kind layout");

            if let Some(clipped_viewport) = pair.bounds().intersection(&viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds: clipped_viewport,
                        ..Default::default()
                    },
                    Background::Color(color!(25, 155, 175)),
                );
            }

            if let Some(label_viewport) = label.bounds().intersection(&viewport) {
                draw(
                    renderer,
                    style,
                    label,
                    header.raw(),
                    padding,
                    &label_viewport,
                )
            }

            if let Some(kind_viewport) = knd.bounds().intersection(&viewport) {
                draw(renderer, style, knd, kind.raw(), padding, &kind_viewport)
            }

            if let Some(Editing::Cell {
                index,
                is_header: true,
                ..
            }) = &self.editing
            {
                if idx == *index {
                    editing.replace(label.bounds().shrink(padding));
                }
            }
        }

        let viewport = {
            let moved = viewport + Vector::new(0.0, headers.bounds().height);

            let size = Size::new(moved.width, moved.height - headers.bounds().height);

            Rectangle::new(moved.position(), size)
        };
        let cell_viewport = viewport;

        for (idx, (cell, layout)) in self.cells.iter().zip(cells.children()).enumerate() {
            let bounds = layout.bounds();
            let child = layout
                .children()
                .next()
                .expect("Table draw: Resize node missing child layout");

            if let Some(clipped_viewport) = bounds.intersection(&viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds: clipped_viewport,
                        ..Default::default()
                    },
                    Background::Color(color!(205, 185, 75)),
                );

                if let Some(clipped_viewport) = child.bounds().intersection(&clipped_viewport) {
                    <Renderer as advanced::Renderer>::fill_quad(
                        renderer,
                        Quad {
                            bounds: clipped_viewport,
                            ..Default::default()
                        },
                        Background::Color(color!(25, 155, 175)),
                    );

                    draw(
                        renderer,
                        style,
                        child,
                        cell.raw(),
                        padding,
                        &clipped_viewport,
                    )
                }
            }

            if let Some(Editing::Cell {
                index,
                is_header: false,
                ..
            }) = &self.editing
            {
                if idx == *index {
                    editing.replace(child.bounds().shrink(padding));
                }
            }
        }

        match (editing, &self.editing) {
            (
                Some(bounds),
                Some(Editing::Cell {
                    index,
                    value,
                    is_header: true,
                }),
            ) => {
                let (cell, _) = &self.headers[*index];
                if let Some(clipped_bounds) = header_viewport.intersection(&bounds) {
                    self.draw_edit(
                        renderer,
                        cell,
                        clipped_bounds,
                        bounds,
                        &value,
                        cell.horizontal_alignment(),
                    )
                }
            }
            (
                Some(bounds),
                Some(Editing::Cell {
                    index,
                    value,
                    is_header: false,
                }),
            ) => {
                let cell = &self.cells[*index];
                if let Some(clipped_bounds) = cell_viewport.intersection(&bounds) {
                    self.draw_edit(
                        renderer,
                        cell,
                        clipped_bounds,
                        bounds,
                        &value,
                        cell.horizontal_alignment(),
                    )
                }
            }
            _ => {}
        };
    }

    fn draw_edit(
        &self,
        renderer: &mut Renderer,
        cell: &Cell,
        clipped_bounds: Rectangle,
        full_bounds: Rectangle,
        value: &str,
        alignment: Horizontal,
    ) {
        let (cursor, offset, is_selecting) = if let Some(focus) = self
            .is_focused
            .as_ref()
            .filter(|focus| focus.is_window_focused)
        {
            let min_bounds = cell.min_bounds();
            let y = full_bounds.y + ((full_bounds.height - min_bounds.height).max(0.0) * 0.5);
            let y2 = y + min_bounds.height;
            let y = y.max(clipped_bounds.y);
            let height = (y2 - y).max(0.0);

            match self.cursor.state(value) {
                utils::State::Index(position) => {
                    let (text_value_width, offset) =
                        measure_cursor_and_scroll_offset(cell.raw(), clipped_bounds, position);

                    let is_cursor_visible = ((focus.now - focus.updated_at).as_millis()
                        / CURSOR_BLINK_INTERVAL_MILLIS)
                        % 2
                        == 0;

                    let cursor = if is_cursor_visible {
                        Some((
                            Quad {
                                bounds: Rectangle {
                                    x: (clipped_bounds.x + text_value_width).floor(),
                                    y,
                                    width: 1.5,
                                    height,
                                },
                                ..Quad::default()
                            },
                            color!(255, 0, 255),
                        ))
                    } else {
                        None
                    };

                    (cursor, offset, false)
                }
                utils::State::Selection { start, end } => {
                    let left = start.min(end);
                    let right = end.max(start);

                    let (left_position, left_offset) =
                        measure_cursor_and_scroll_offset(cell.raw(), clipped_bounds, left);

                    let (right_position, right_offset) =
                        measure_cursor_and_scroll_offset(cell.raw(), clipped_bounds, right);

                    let width = right_position - left_position;

                    (
                        Some((
                            Quad {
                                bounds: Rectangle {
                                    x: clipped_bounds.x + left_position,
                                    y,
                                    width,
                                    height,
                                },
                                ..Quad::default()
                            },
                            color!(225, 42, 251),
                        )),
                        if end == right {
                            right_offset
                        } else {
                            left_offset
                        },
                        true,
                    )
                }
            }
        } else {
            (None, 0.0, false)
        };

        let draw = |renderer: &mut Renderer| {
            let paragraph = cell.raw();

            let alignment_offset =
                alignment_offset(clipped_bounds.width, paragraph.min_width(), alignment);

            if let Some((cursor, color)) = cursor {
                <Renderer as advanced::Renderer>::with_translation(
                    renderer,
                    Vector::new(alignment_offset - offset, 0.0),
                    |renderer| {
                        <Renderer as advanced::Renderer>::fill_quad(renderer, cursor, color);
                    },
                );
            } else {
                <Renderer as advanced::Renderer>::with_translation(renderer, Vector::ZERO, |_| {});
            }
        };

        if is_selecting {
            <Renderer as advanced::Renderer>::with_layer(renderer, clipped_bounds, |renderer| {
                draw(renderer)
            });
        } else {
            draw(renderer);
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: layout::Layout<'_>,
        style: &iced::advanced::renderer::Style,
        padding: Padding,
        cell_padding: Padding,
        spacing: f32,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let mut children = layout.children();
        let cells = children.next().expect("Widget draw: Missing cells layout");
        let status = children.next().expect("Widget draw: Missing status layout");
        let pagination = children
            .next()
            .expect("Widget draw: Missing pagination layout");
        let goto = children.next().expect("Widget draw: Missing goto layout");

        let cells_bounds = {
            let width = bounds.width - padding.horizontal() + Self::CELL_GAP;
            let diff = padding.vertical()
                + pagination.bounds().height.max(goto.bounds().height)
                + if self.multiple_pages() { spacing } else { 0.0 }
                + status.bounds().height
                + spacing;

            let height = bounds.height - diff;

            let size = Size::new(width, height);

            let y = bounds.y + diff - padding.bottom;
            let x = bounds.x + padding.left;

            Rectangle::new(Point::new(x, y), size)
        };

        if let Some(clipped_viewport) = cells_bounds.intersection(viewport) {
            self.draw_cells(renderer, cells, style, clipped_viewport, cell_padding)
        };

        self.draw_status(renderer, status, style, viewport);

        if self.multiple_pages() {
            self.draw_pagination(renderer, pagination, style, viewport);

            self.draw_goto(renderer, goto, style, viewport);
        }

        if let Some(Editing::Goto(bounds)) = &self.editing {
            self.draw_edit(
                renderer,
                &self.goto_input.0,
                *bounds,
                *bounds,
                &self.goto_input.1,
                self.goto_input.0.horizontal_alignment(),
            )
        };
    }

    fn interaction_cells(
        &self,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let mut children = layout.children();
        let _numbering = children
            .next()
            .expect("Widget Interaction: Missing numbering cells");
        let headers = children
            .next()
            .expect("Widget Interaction: Missing header cells");

        for resize in headers.children() {
            let pair = resize
                .children()
                .next()
                .expect("Table Interaction: Resize node missing pair layout");

            let resize = resize.bounds();

            let label = pair
                .children()
                .next()
                .expect("Table Interaction: Pair node missing label layout")
                .bounds();

            let pair = pair.bounds();

            if cursor.is_over(label) {
                return mouse::Interaction::Text;
            }

            if cursor.is_over(resize) {
                let horizontal = {
                    let position = resize.position() + Vector::new(pair.width, 0.0);
                    let height = resize.height;
                    let width = resize.width - pair.width;

                    let horizontal = Rectangle::new(position, Size::new(width, height));

                    cursor.is_over(horizontal)
                };
                let vertical = {
                    let position = resize.position() + Vector::new(0.0, pair.height);
                    let width = resize.width;
                    let height = resize.height - pair.height;

                    let vertical = Rectangle::new(position, Size::new(width, height));
                    cursor.is_over(vertical)
                };

                if vertical && horizontal {
                    return mouse::Interaction::ResizingDiagonallyDown;
                }

                if vertical {
                    return mouse::Interaction::ResizingVertically;
                }

                if horizontal {
                    return mouse::Interaction::ResizingHorizontally;
                }
            }
        }

        let cells = children.next().expect("Widget Interaction: Missing cells");

        for cell in cells.children() {
            let resize = cell.bounds();
            let child = cell
                .children()
                .next()
                .expect("Table Interaction: Resize node missing child layout")
                .bounds();

            if cursor.is_over(child) {
                return mouse::Interaction::Text;
            }

            if cursor.is_over(resize) {
                let horizontal = {
                    let position = resize.position() + Vector::new(child.width, 0.0);
                    let height = resize.height;
                    let width = resize.width - child.width;

                    let horizontal = Rectangle::new(position, Size::new(width, height));

                    cursor.is_over(horizontal)
                };
                let vertical = {
                    let position = resize.position() + Vector::new(0.0, child.height);
                    let width = resize.width;
                    let height = resize.height - child.height;

                    let vertical = Rectangle::new(position, Size::new(width, height));
                    cursor.is_over(vertical)
                };

                if vertical && horizontal {
                    return mouse::Interaction::ResizingDiagonallyDown;
                }

                if vertical {
                    return mouse::Interaction::ResizingVertically;
                }

                if horizontal {
                    return mouse::Interaction::ResizingHorizontally;
                }
            }
        }

        mouse::Interaction::None
    }

    fn interaction_pagination(
        &self,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let mut children = layout.children();

        let back = children
            .next()
            .expect("Widget Interaction: missing paginations: Back");

        if cursor.is_over(back.bounds()) {
            return mouse::Interaction::Pointer;
        }

        let pages = children
            .next()
            .expect("Widget Interaction: missing paginations: Pages");

        if pages.children().any(|page| cursor.is_over(page.bounds())) {
            return mouse::Interaction::Pointer;
        }

        let next = children
            .next()
            .expect("Widget Interaction: missing paginations: Next");

        if cursor.is_over(next.bounds()) {
            return mouse::Interaction::Pointer;
        }

        mouse::Interaction::None
    }

    fn interaction_goto(
        &self,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let mut children = layout.children();
        let _ = children.next();

        let input = children
            .next()
            .expect("Widget interaction: Missing goto input layout");

        if cursor.is_over(input.bounds()) {
            return mouse::Interaction::Text;
        }

        let go = children
            .next()
            .expect("Widget Interaction: Missing goto go layout");
        if cursor.is_over(go.bounds()) {
            return mouse::Interaction::Pointer;
        }

        mouse::Interaction::None
    }

    fn mouse_interaction(
        &self,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if let Some(interaction) = self.resizing.map(|resize| resize.interaction()) {
            return interaction;
        }

        let mut children = layout.children();

        let cells = children
            .next()
            .expect("Widget Interaction: Missing cells layout");
        if cursor.is_over(cells.bounds()) {
            return self.interaction_cells(cells, cursor);
        }

        let _status = children.next();

        if self.multiple_pages() {
            let pagination = children
                .next()
                .expect("Widget Interaction: Missing pagination layout");
            if cursor.is_over(pagination.bounds()) {
                return self.interaction_pagination(pagination, cursor);
            }

            let goto = children
                .next()
                .expect("Widget Interaction: Missing goto layout");
            if cursor.is_over(goto.bounds()) {
                return self.interaction_goto(goto, cursor);
            }
        }

        mouse::Interaction::None
    }

    fn update_cells<Message>(
        &mut self,
        event: event::Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        font: Font,
        size: Pixels,
        padding: Padding,
        shell: &mut Shell<'_, Message>,
        scroll_bounds: Size,
    ) -> event::Status {
        if self.raw.is_empty() {
            return event::Status::Ignored;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let mut children = layout.children();
                let _numbering = children
                    .next()
                    .expect("Widget Update: Missing numbering cells");
                let headers = children
                    .next()
                    .expect("Widget Update: Missing header cells")
                    .children()
                    .map(|child| (true, child));
                let cells = children
                    .next()
                    .expect("Widget Update: Missing cells")
                    .children()
                    .map(|child| (false, child));
                let children = headers.chain(cells);

                match children
                    .enumerate()
                    .find(|(_, (_, child))| cursor.is_over(child.bounds()))
                {
                    Some((idx, (is_header, cell))) => {
                        let cell_bounds = cell.bounds();
                        let cell = if is_header {
                            cell.children()
                                .next()
                                .expect("Table Update: Resize node missing pair layout")
                                .children()
                                .next()
                                .expect("Table Update: Pair node missing label layout")
                        } else {
                            cell.children()
                                .next()
                                .expect("Table Update: Resize node missing child layout")
                        };

                        let (row, column) = if is_header {
                            (0, idx + 1)
                        } else {
                            let idx = idx - self.cols;
                            let column = (idx / self.page_limit) + 1;
                            let row = (idx + 1) - ((idx / self.page_limit) * self.page_limit);
                            (row, column)
                        };

                        let resize = Resizing::new(cell_bounds, cell.bounds(), cursor, row, column);
                        self.resizing = resize;

                        if resize.is_some() {
                            self.reset_editing();
                            return event::Status::Captured;
                        }

                        let cell_bounds = cell.bounds().shrink(padding);

                        let Some(cursor_position) = cursor.position_over(cell_bounds) else {
                            return event::Status::Ignored;
                        };

                        let (idx, cell, value) = if is_header {
                            let (cell, _) = &self.headers[idx];
                            let col = self
                                .raw
                                .get_col(idx)
                                .expect("Cells update: Missing column in sheet");

                            let value = col.label().unwrap_or_default().to_owned();

                            (idx, cell, value)
                        } else {
                            let idx = idx - self.cols;
                            let cell = &self.cells[idx];
                            let (row, column) = (idx % self.page_limit, idx / self.page_limit);
                            let row = row + (self.page * self.page_limit);

                            let col = self
                                .raw
                                .get_col(column)
                                .expect("Cells update: Missing column in sheet");

                            let value = col.data_ref(row).map(cell_to_string).unwrap_or_default();

                            (idx, cell, value)
                        };

                        let target = {
                            let alignment_offset = alignment_offset(
                                cell_bounds.width,
                                cell.min_width(),
                                cell.horizontal_alignment(),
                            );

                            cursor_position.x - cell_bounds.x - alignment_offset
                        };

                        let click = mouse::Click::new(
                            cursor_position,
                            mouse::Button::Left,
                            self.last_click,
                        );

                        match click.kind() {
                            click::Kind::Single => {
                                let position = if target > 0.0 {
                                    find_cursor_position(cell_bounds, &value, self, &cell, target)
                                } else {
                                    None
                                }
                                .unwrap_or(0);

                                if self.keyboard_modifiers.shift() {
                                    self.cursor
                                        .select_range(self.cursor.start(&value), position);
                                } else {
                                    self.cursor.move_to(position);
                                }

                                self.is_text_dragging = true;
                            }
                            click::Kind::Double => {
                                let position =
                                    find_cursor_position(cell_bounds, &value, self, cell, target)
                                        .unwrap_or(0);
                                let (start, end) = word_boundary(&value, position);
                                self.cursor.select_range(start, end);
                                self.is_text_dragging = false;
                            }
                            click::Kind::Triple => {
                                self.cursor.select_all(&value);
                                self.is_text_dragging = false;
                            }
                        }

                        self.last_click = Some(click);
                        self.editing = Some(Editing::Cell {
                            index: idx,
                            value,
                            is_header,
                        });

                        return event::Status::Captured;
                    }
                    None => {
                        self.reset();

                        event::Status::Ignored
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { position, .. })
                if self.is_text_dragging =>
            {
                self.reset_resizing();
                let Some(Editing::Cell {
                    index,
                    value,
                    is_header,
                }) = &self.editing
                else {
                    return event::Status::Ignored;
                };

                let mut children = layout.children();
                let _numbering = children.next();
                let headers = children
                    .next()
                    .expect("Widget Update: Missing header cells")
                    .children();
                let cells = children
                    .next()
                    .expect("Widget Update: Missing cells")
                    .children();

                let (bounds, cell) = if *is_header {
                    let bounds = headers
                        .enumerate()
                        .find(|(idx, _)| *idx == *index)
                        // Pair node
                        .and_then(|(_, resize)| resize.children().next())
                        // Label node
                        .and_then(|pair| pair.children().next())
                        .map(|label| label.bounds())
                        .expect("Table Update: Editing selection header missing layout");
                    let (cell, _) = &self.headers[*index];
                    (bounds, cell)
                } else {
                    let bounds = cells
                        .enumerate()
                        .find(|(idx, _)| *idx == *index)
                        .map(|(_, resize)| {
                            resize
                                .children()
                                .next()
                                .expect("Table Update: Editing resize node missing cell layout")
                                .bounds()
                        })
                        .expect("Table Update: Editing selection missing layout");
                    let cell = &self.cells[*index];
                    (bounds, cell)
                };

                let target = {
                    let alignment_offset = alignment_offset(
                        bounds.width,
                        cell.min_width(),
                        cell.horizontal_alignment(),
                    );

                    position.x - bounds.x - alignment_offset
                };

                let position =
                    find_cursor_position(bounds, &value, self, cell, target).unwrap_or(0);

                self.cursor
                    .select_range(self.cursor.start(&value), position);

                return event::Status::Captured;
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { position, .. })
                if self.resizing.is_some() =>
            {
                let Some(resize) = self.resizing.as_mut() else {
                    return event::Status::Ignored;
                };
                let width = self.min_widths[resize.column];
                let height = self.min_heights[resize.row];
                let (new, diff) = resize.drag(position, width, height);

                self.min_widths[resize.column] = new.width;
                self.min_heights[resize.row] = new.height;

                self.scroll_cells(scroll_bounds, diff * (1.0 / Self::SCROLL_MULT));
                shell.invalidate_layout();
                return event::Status::Captured;
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, text, .. }) => {
                let Some(focus) = self.is_focused.as_mut() else {
                    return event::Status::Ignored;
                };

                let Some(Editing::Cell {
                    index,
                    value,
                    is_header,
                    ..
                }) = self.editing.as_mut()
                else {
                    return event::Status::Ignored;
                };

                let index = *index;
                let modifiers = self.keyboard_modifiers;
                focus.updated_at = Instant::now();

                let (cell, col, row, column) = if *is_header {
                    let (cell, _) = &mut self.headers[index];
                    let col = self
                        .raw
                        .get_col_mut(index)
                        .expect("Cells update: Missing column in sheet");
                    (cell, col, 0, index + 1)
                } else {
                    let cell = &mut self.cells[index];
                    let (row, column) = (index % self.page_limit, index / self.page_limit);
                    let row = row + (self.page * self.page_limit);

                    let col = self
                        .raw
                        .get_col_mut(column)
                        .expect("Cells update: Missing column in sheet");

                    (cell, col, row, column)
                };

                let col_kind = col.kind();

                if key.as_ref() == keyboard::Key::Character("a") && modifiers.command() {
                    self.cursor.select_all(&value);
                    return event::Status::Captured;
                }

                match text {
                    Some(text) if *is_header => {
                        if let Some(c) = text.chars().next().filter(|c| !c.is_control()) {
                            let mut editor = Editor::new(value, &mut self.cursor);
                            editor.insert(c);

                            cell.update(self::text(
                                value,
                                Self::MAX_CELL,
                                font,
                                cell.horizontal_alignment(),
                                size,
                            ));

                            focus.updated_at = Instant::now();

                            let min_bounds = cell.min_bounds().expand(padding);
                            let bounds = Size::new(self.min_widths[column], self.min_heights[row]);

                            if min_bounds.width > bounds.width {
                                self.min_widths[column] = min_bounds.width;
                                self.min_heights[row] = min_bounds.height;
                                shell.invalidate_layout();
                            }

                            return event::Status::Captured;
                        }
                    }
                    Some(text) => {
                        if let Some(c) = text
                            .chars()
                            .next()
                            .filter(|c| !c.is_control() && column_filter(col_kind, *c))
                        {
                            let mut editor = Editor::new(value, &mut self.cursor);
                            editor.insert(c);

                            cell.update(self::text(
                                value,
                                Self::MAX_CELL,
                                font,
                                cell.horizontal_alignment(),
                                size,
                            ));

                            focus.updated_at = Instant::now();

                            let column = column + 1;
                            let row = row + 1;
                            let min_bounds = cell.min_bounds().expand(padding);
                            let bounds = Size::new(self.min_widths[column], self.min_heights[row]);

                            if min_bounds.width > bounds.width || min_bounds.height > bounds.height
                            {
                                self.min_widths[column] = min_bounds.width;
                                self.min_heights[row] = min_bounds.height;
                                shell.invalidate_layout();
                            }
                            return event::Status::Captured;
                        }
                    }
                    None => {}
                }

                match key.as_ref() {
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        if *is_header {
                            col.set_header(value.clone());
                        } else {
                            if let Err(error) = self.raw.set_cell(&value, column, row) {
                                let msg = error.to_string();
                                let (cell, status) = &mut self.status;

                                cell.update(self::text(
                                    &msg,
                                    Self::MAX_CELL,
                                    font,
                                    cell.horizontal_alignment(),
                                    self.page_size,
                                ));

                                *status = msg;
                            };
                        }

                        self.reset();
                        shell.invalidate_layout();
                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                        let mut editor = Editor::new(value, &mut self.cursor);
                        editor.backspace();

                        cell.update(self::text(
                            value,
                            Self::MAX_CELL,
                            font,
                            cell.horizontal_alignment(),
                            size,
                        ));

                        //shell.invalidate_layout();
                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(keyboard::key::Named::Delete) => {
                        let mut editor = Editor::new(value, &mut self.cursor);
                        editor.delete();

                        cell.update(self::text(
                            value,
                            Self::MAX_CELL,
                            font,
                            cell.horizontal_alignment(),
                            size,
                        ));

                        //shell.invalidate_layout();
                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        if modifiers.shift() {
                            self.cursor.select_left(value);
                        } else {
                            self.cursor.move_left(value);
                        }

                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        if modifiers.shift() {
                            self.cursor.select_right(value);
                        } else {
                            self.cursor.move_right(value);
                        }

                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        self.reset();
                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(
                        keyboard::key::Named::Tab
                        | keyboard::key::Named::ArrowUp
                        | keyboard::key::Named::ArrowDown,
                    ) => {
                        return event::Status::Ignored;
                    }

                    _ => event::Status::Captured,
                }
            }
            _ => event::Status::Ignored,
        }
    }

    fn update_pagination<Message>(
        &mut self,
        event: event::Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let mut children = layout.children();

                let back = children
                    .next()
                    .expect("Widget Update: missing paginations: Back");

                if cursor.is_over(back.bounds()) && self.page != 0 {
                    self.page -= 1;
                    self.goto_input.1 = (self.page + 1).to_string();
                    shell.invalidate_layout();
                    return event::Status::Captured;
                }

                let pages = children
                    .next()
                    .expect("Widget Update: missing paginations: Pages");

                if cursor.is_over(pages.bounds()) {
                    let Some(idx) = pages
                        .children()
                        .enumerate()
                        .find(|(_, page)| cursor.is_over(page.bounds()))
                        .map(|(idx, _)| idx)
                    else {
                        return event::Status::Ignored;
                    };

                    let (_, value) = self
                        .paginations
                        .get(idx)
                        .expect("Widget Update: pages cells and layout not equal length");

                    match value.parse::<usize>() {
                        Ok(page) => self.page = page - 1,
                        Err(_) if value == PAGINATION_ELLIPSIS => {
                            let (_, left) = &self.paginations[idx - 1];
                            let (_, right) = &self.paginations[idx + 1];

                            let left = left.parse::<usize>().expect("No way this fails");
                            let right = right.parse::<usize>().expect("No way this fails");

                            let page = left + (right - left) / 2;

                            self.page = page;
                        }
                        Err(_) if value.is_empty() => self.page = 0,
                        Err(_) => {}
                    }

                    self.goto_input.1 = (self.page + 1).to_string();
                    shell.invalidate_layout();
                    return event::Status::Captured;
                }

                let next = children
                    .next()
                    .expect("Widget Update: missing paginations: Next");

                if cursor.is_over(next.bounds()) && self.page + 1 <= self.pages_end() {
                    self.page += 1;
                    self.goto_input.1 = (self.page + 1).to_string();
                    shell.invalidate_layout();
                    return event::Status::Captured;
                }

                event::Status::Ignored
            }
            _ => event::Status::Ignored,
        }
    }

    fn update_goto<Message>(
        &mut self,
        event: event::Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        font: Font,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let mut children = layout.children();

        let _ = children.next();

        let input = children.next().expect("Widget Update: Missing Goto Input");
        let go = children.next().expect("Widget Update: Missing Goto Go");

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                match cursor.position_over(input.bounds()) {
                    Some(cursor_position) => {
                        let target = {
                            let input_bounds = input.bounds().shrink(self.pages_padding);

                            let alignment_offset = alignment_offset(
                                input_bounds.width,
                                self.goto_input.0.min_width(),
                                Horizontal::Right,
                            );

                            cursor_position.x - input_bounds.x - alignment_offset
                        };

                        let click = mouse::Click::new(
                            cursor_position,
                            mouse::Button::Left,
                            self.last_click,
                        );

                        match click.kind() {
                            click::Kind::Single => {
                                let position = if target > 0.0 {
                                    let value = &self.goto_input.1;

                                    find_cursor_position(
                                        input.bounds().shrink(self.pages_padding),
                                        &value,
                                        self,
                                        &self.goto_input.0,
                                        target,
                                    )
                                } else {
                                    None
                                }
                                .unwrap_or(0);

                                if self.keyboard_modifiers.shift() {
                                    self.cursor.select_range(
                                        self.cursor.start(&self.goto_input.1),
                                        position,
                                    );
                                } else {
                                    self.cursor.move_to(position);
                                }
                                self.is_text_dragging = true;
                            }
                            click::Kind::Double => {
                                self.cursor.select_range(0, usize::MAX);

                                self.is_text_dragging = false;
                            }
                            click::Kind::Triple => {
                                self.cursor.select_all(&self.goto_input.1);
                                self.is_text_dragging = false;
                            }
                        }

                        self.last_click = Some(click);
                        self.editing =
                            Some(Editing::Goto(input.bounds().shrink(self.pages_padding)));

                        return event::Status::Captured;
                    }
                    None => {
                        self.reset();

                        if cursor.is_over(go.bounds()) {
                            let (_, page) = &self.goto_input;
                            match page.parse::<usize>() {
                                Ok(page) => {
                                    self.page = usize::clamp(page - 1, 0, self.pages_end());
                                    shell.invalidate_layout();
                                    return event::Status::Captured;
                                }
                                Err(_) if page.is_empty() => {
                                    self.page = 0;
                                    shell.invalidate_layout();
                                    return event::Status::Captured;
                                }
                                _ => {}
                            }
                        }

                        event::Status::Ignored
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { position, .. })
                if self.is_text_dragging =>
            {
                let text_bounds = input.bounds();

                let target = {
                    let alignment_offset = alignment_offset(
                        text_bounds.width,
                        self.goto_input.0.raw().min_width(),
                        Horizontal::Right,
                    );

                    position.x - text_bounds.x - alignment_offset
                };

                let (cell, value) = &self.goto_input;

                let position =
                    find_cursor_position(text_bounds, value, self, cell, target).unwrap_or(0);

                self.cursor
                    .select_range(self.cursor.start(&value), position);

                return event::Status::Captured;
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, text, .. }) => {
                let Some(focus) = self.is_focused.as_mut() else {
                    return event::Status::Ignored;
                };

                let modifiers = self.keyboard_modifiers;
                focus.updated_at = Instant::now();

                let (cell, value) = &mut self.goto_input;

                if key.as_ref() == keyboard::Key::Character("a") && modifiers.command() {
                    self.cursor.select_all(value);
                    return event::Status::Captured;
                }

                if let Some(text) = text {
                    if let Some(c) = text
                        .chars()
                        .next()
                        .filter(|c| !c.is_control() && c.is_digit(10))
                    {
                        let mut editor = Editor::new(value, &mut self.cursor);

                        editor.insert(c);

                        let pages_end = self.raw.height() / self.page_limit;
                        match value.parse::<usize>() {
                            Ok(page) if page > pages_end => *value = (pages_end + 1).to_string(),
                            Err(_) if value.is_empty() => {
                                *value = (self.page + 1).to_string();
                            }
                            _ => {}
                        }

                        cell.update(self::text(
                            value,
                            Self::MAX_CELL,
                            font,
                            Horizontal::Right,
                            self.page_size,
                        ));

                        focus.updated_at = Instant::now();

                        return event::Status::Captured;
                    }
                }

                match key.as_ref() {
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        if let Ok(page) = value.parse::<usize>() {
                            let page = if page == 0 { 0 } else { page - 1 };
                            self.page = usize::clamp(page, 0, self.pages_end());
                            self.reset();
                            shell.invalidate_layout();
                            return event::Status::Captured;
                        } else if value.is_empty() {
                            *value = (self.page + 1).to_string();

                            self.reset();
                            shell.invalidate_layout();
                            return event::Status::Captured;
                        }
                    }
                    keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                        let mut editor = Editor::new(value, &mut self.cursor);
                        editor.backspace();
                        cell.update(self::text(
                            value,
                            Self::MAX_CELL,
                            font,
                            Horizontal::Right,
                            self.page_size,
                        ));
                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(keyboard::key::Named::Delete) => {
                        let mut editor = Editor::new(value, &mut self.cursor);
                        editor.delete();
                        cell.update(self::text(
                            value,
                            Self::MAX_CELL,
                            font,
                            Horizontal::Right,
                            self.page_size,
                        ));
                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        if modifiers.shift() {
                            self.cursor.select_left(value)
                        } else {
                            self.cursor.move_left(value)
                        }
                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        if modifiers.shift() {
                            self.cursor.select_right(value)
                        } else {
                            self.cursor.move_right(value)
                        }
                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        self.reset();
                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(
                        keyboard::key::Named::Tab
                        | keyboard::key::Named::ArrowUp
                        | keyboard::key::Named::ArrowDown,
                    ) => {
                        return event::Status::Ignored;
                    }

                    _ => {}
                }

                event::Status::Captured
            }
            _ => event::Status::Ignored,
        }
    }

    fn on_update<Message>(
        &mut self,
        event: event::Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        padding: Padding,
        cell_padding: Padding,
        font: Font,
        size: Pixels,
        spacing: f32,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let mut children = layout.children();

        let cells = children
            .next()
            .expect("Widget Update: Missing cells layout");

        let status = children
            .next()
            .expect("Widget Update: Missing status layout");

        let pagination = children
            .next()
            .expect("Widget Update: Missing pagination layout");

        let goto = children.next().expect("Widget Update: Missing goto layout");

        match &event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                self.is_focused = if cursor.is_over(layout.bounds()) {
                    self.is_focused.or_else(|| {
                        let now = Instant::now();

                        Some(Focus {
                            updated_at: now,
                            now,
                            is_window_focused: true,
                        })
                    })
                } else {
                    None
                };

                if cursor.is_over(cells.bounds()) {
                    let mut cells_children = cells.children();
                    let numbering = cells_children
                        .next()
                        .expect("Widget Update: Missing numbering cells");
                    let headers = cells_children
                        .next()
                        .expect("Widget Update: Missing header cells");

                    let scroll_bounds = {
                        let diff = padding.vertical()
                            + pagination.bounds().height.max(goto.bounds().height)
                            + if self.multiple_pages() { spacing } else { 0.0 }
                            + status.bounds().height
                            + spacing
                            + headers.bounds().height;

                        let height = bounds.height - diff;
                        let width = bounds.width - padding.horizontal() - numbering.bounds().width;

                        Size::new(width, height)
                    };
                    return self.update_cells(
                        event,
                        cells,
                        cursor,
                        font,
                        size,
                        cell_padding,
                        shell,
                        scroll_bounds,
                    );
                }

                if cursor.is_over(pagination.bounds()) && self.multiple_pages() {
                    self.reset();
                    return self.update_pagination(event, pagination, cursor, shell);
                }

                if cursor.is_over(goto.bounds()) && self.multiple_pages() {
                    return self.update_goto(event, goto, cursor, font, shell);
                }

                match self.editing.as_ref() {
                    Some(Editing::Cell {
                        index,
                        value,
                        is_header,
                        ..
                    }) if !self.raw.is_empty() => {
                        let index = *index;
                        if *is_header {
                            self.raw
                                .get_col_mut(index)
                                .expect("Cells update: Missing column in sheet")
                                .set_header(value.clone());
                        } else {
                            let (row, column) = (index % self.page_limit, index / self.page_limit);

                            if let Err(error) = self.raw.set_cell(&value, column, row) {
                                let msg = error.to_string();
                                let (cell, status) = &mut self.status;

                                cell.update(self::text(
                                    &msg,
                                    Self::MAX_CELL,
                                    font,
                                    cell.horizontal_alignment(),
                                    self.page_size,
                                ));

                                *status = msg;
                            };
                        }

                        self.reset();
                        shell.invalidate_layout();
                        return event::Status::Ignored;
                    }
                    _ => {
                        self.reset();
                        return event::Status::Ignored;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                self.is_text_dragging = false;
                self.reset_resizing();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. })
                if self.is_text_dragging =>
            {
                match self.editing {
                    Some(Editing::Goto(_)) => {
                        return self.update_goto(event, goto, cursor, font, shell);
                    }
                    Some(Editing::Cell { .. }) => {
                        let mut cells_children = cells.children();
                        let numbering = cells_children
                            .next()
                            .expect("Widget Update: Missing numbering cells");
                        let headers = cells_children
                            .next()
                            .expect("Widget Update: Missing header cells");

                        let scroll_bounds = {
                            let diff = padding.vertical()
                                + pagination.bounds().height.max(goto.bounds().height)
                                + if self.multiple_pages() { spacing } else { 0.0 }
                                + status.bounds().height
                                + spacing
                                + headers.bounds().height;

                            let height = bounds.height - diff;
                            let width =
                                bounds.width - padding.horizontal() - numbering.bounds().width;

                            Size::new(width, height)
                        };
                        return self.update_cells(
                            event,
                            cells,
                            cursor,
                            font,
                            size,
                            cell_padding,
                            shell,
                            scroll_bounds,
                        );
                    }
                    None => {}
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. })
                if self.resizing.is_some() =>
            {
                let mut cells_children = cells.children();
                let numbering = cells_children
                    .next()
                    .expect("Widget Update: Missing numbering cells");
                let headers = cells_children
                    .next()
                    .expect("Widget Update: Missing header cells");

                let scroll_bounds = {
                    let diff = padding.vertical()
                        + pagination.bounds().height.max(goto.bounds().height)
                        + if self.multiple_pages() { spacing } else { 0.0 }
                        + status.bounds().height
                        + spacing
                        + headers.bounds().height;

                    let height = bounds.height - diff;
                    let width = bounds.width - padding.horizontal() - numbering.bounds().width;

                    Size::new(width, height)
                };
                return self.update_cells(
                    event,
                    cells,
                    cursor,
                    font,
                    size,
                    cell_padding,
                    shell,
                    scroll_bounds,
                );
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let delta = match *delta {
                    mouse::ScrollDelta::Pixels { x, y } => Vector::new(x, y),
                    // Intentionally multiplying by scroll mult twice. Result
                    // is smoother on windows
                    mouse::ScrollDelta::Lines { x, y } => Vector::new(x, y) * Self::SCROLL_MULT,
                };

                let mut cells_children = cells.children();
                let numbering = cells_children
                    .next()
                    .expect("Widget Update: Missing numbering cells");
                let headers = cells_children
                    .next()
                    .expect("Widget Update: Missing header cells");

                let scroll_bounds = {
                    let diff = padding.vertical()
                        + pagination.bounds().height.max(goto.bounds().height)
                        + if self.multiple_pages() { spacing } else { 0.0 }
                        + status.bounds().height
                        + spacing
                        + headers.bounds().height;

                    let height = bounds.height - diff;
                    let width = bounds.width - padding.horizontal() - numbering.bounds().width;

                    Size::new(width, height)
                };

                self.scroll_cells(scroll_bounds, delta);
                shell.invalidate_layout();
                return event::Status::Captured;
            }
            Event::Keyboard(keyboard::Event::KeyPressed { .. }) => match self.editing {
                Some(Editing::Goto(_)) => {
                    return self.update_goto(event, goto, cursor, font, shell)
                }
                Some(Editing::Cell { .. }) => {
                    let mut cells_children = cells.children();
                    let numbering = cells_children
                        .next()
                        .expect("Widget Update: Missing numbering cells");
                    let headers = cells_children
                        .next()
                        .expect("Widget Update: Missing header cells");

                    let scroll_bounds = {
                        let diff = padding.vertical()
                            + pagination.bounds().height.max(goto.bounds().height)
                            + if self.multiple_pages() { spacing } else { 0.0 }
                            + status.bounds().height
                            + spacing
                            + headers.bounds().height;

                        let height = bounds.height - diff;
                        let width = bounds.width - padding.horizontal() - numbering.bounds().width;

                        Size::new(width, height)
                    };
                    return self.update_cells(
                        event,
                        cells,
                        cursor,
                        font,
                        size,
                        cell_padding,
                        shell,
                        scroll_bounds,
                    );
                }
                None => {}
            },
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                self.keyboard_modifiers = *modifiers;
            }
            Event::Window(window::Event::Unfocused) => {
                if let Some(focus) = &mut self.is_focused {
                    focus.is_window_focused = false;
                }
            }
            Event::Window(window::Event::Focused) => {
                if let Some(focus) = &mut self.is_focused {
                    focus.is_window_focused = true;
                    focus.updated_at = Instant::now();

                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                if let Some(focus) = &mut self.is_focused {
                    if focus.is_window_focused {
                        focus.now = *now;

                        let millis_until_redraw = CURSOR_BLINK_INTERVAL_MILLIS
                            - (*now - focus.updated_at).as_millis() % CURSOR_BLINK_INTERVAL_MILLIS;

                        shell.request_redraw(window::RedrawRequest::At(
                            *now + Duration::from_millis(millis_until_redraw as u64),
                        ));
                    }
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }
}

enum State {
    Invalid { error: Error, message: Cell },
    Valid(Internal),
}

impl State {
    fn new(config: &Config<String>, font: Font, size: Pixels) -> Self {
        let sht = ColumnSheet::with_config(config.clone());

        match sht {
            Ok(sht) => Self::Valid(Internal::new(sht, font, size)),
            Err(error) => Self::Invalid {
                error,
                message: Cell::default(),
            },
        }
    }

    fn layout(
        &mut self,
        limits: Limits,
        width: Length,
        height: Length,
        font: Font,
        padding: Padding,
        cell_padding: Padding,
        size: Pixels,
        spacing: f32,
    ) -> Node {
        match self {
            Self::Invalid {
                message: paragraph,
                error,
            } => {
                let msg = error.to_string();
                let bounds = limits.resolve(width, height, Size::INFINITY);
                let text = text(&msg, bounds, font, Horizontal::Left, size);
                paragraph.update(text);
                Node::new(paragraph.min_bounds().expand(padding))
            }
            Self::Valid(internal) => internal.layout(
                limits,
                width,
                height,
                font,
                padding,
                cell_padding,
                size,
                spacing,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Focus {
    updated_at: Instant,
    now: Instant,
    is_window_focused: bool,
}

#[derive(Debug, Clone)]
enum Editing {
    Goto(Rectangle),
    Cell {
        index: usize,
        value: String,
        is_header: bool,
    },
}

fn text(
    content: &str,
    bounds: Size,
    font: Font,
    horizontal: Horizontal,
    size: Pixels,
) -> text::Text<&str> {
    text::Text {
        content,
        bounds,
        size,
        line_height: LineHeight::default(),
        horizontal_alignment: horizontal,
        vertical_alignment: Vertical::Center,
        font,
        shaping: Shaping::default(),
        wrapping: Wrapping::Word,
    }
}

fn draw<Renderer>(
    renderer: &mut Renderer,
    style: &iced::advanced::renderer::Style,
    layout: layout::Layout<'_>,
    paragraph: &Renderer::Paragraph,
    padding: Padding,
    viewport: &Rectangle,
) where
    Renderer: text::Renderer,
{
    let bounds = layout.bounds().shrink(padding);

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

    renderer.fill_paragraph(paragraph, Point::new(x, y), style.text_color, *viewport);
}

fn cell_to_string(cell: CellRef<'_>) -> String {
    match cell {
        CellRef::Text(value) => value.to_owned(),
        CellRef::I32(value) => value.to_string(),
        CellRef::U32(value) => value.to_string(),
        CellRef::ISize(value) => value.to_string(),
        CellRef::USize(value) => value.to_string(),
        CellRef::F32(value) => value.to_string(),
        CellRef::F64(value) => value.to_string(),
        CellRef::Bool(value) => value.to_string(),
        CellRef::None => "None".to_owned(),
    }
}

fn type_alignment(kind: DataType) -> Horizontal {
    match kind {
        DataType::Text | DataType::Bool => Horizontal::Left,
        _ => Horizontal::Right,
    }
}

fn gen_pagination(start: isize, end: isize, curr: isize) -> Vec<String> {
    let extra_left = (4 - (curr - start - 1)).max(0);
    let extra_right = (4 - (end - 1 - curr)).max(0);

    let curr_end = (curr + 3 + extra_left).min(end - 1);
    let curr_start = (curr - 3 - extra_right).max(start + 1);

    let mut output = Vec::with_capacity(11);
    output.push(start.to_string());

    if curr_start != start + 1 {
        output.push(PAGINATION_ELLIPSIS.to_owned());
    }

    for r in curr_start..=curr_end {
        output.push(r.to_string());
    }

    if curr_end != end - 1 {
        output.push(PAGINATION_ELLIPSIS.to_owned())
    }

    output.push(end.to_string());

    output
}

fn alignment_offset(
    text_bounds_width: f32,
    text_min_width: f32,
    alignment: alignment::Horizontal,
) -> f32 {
    if text_min_width > text_bounds_width {
        0.0
    } else {
        match alignment {
            alignment::Horizontal::Left => 0.0,
            alignment::Horizontal::Center => (text_bounds_width - text_min_width) / 2.0,
            alignment::Horizontal::Right => text_bounds_width - text_min_width,
        }
    }
}

fn measure_cursor_and_scroll_offset(
    paragraph: &impl text::Paragraph,
    text_bounds: Rectangle,
    cursor_index: usize,
) -> (f32, f32) {
    let grapheme_position = paragraph
        .grapheme_position(0, cursor_index)
        .unwrap_or(Point::ORIGIN);

    let offset = ((grapheme_position.x + 5.0) - text_bounds.width).max(0.0);

    (grapheme_position.x, offset)
}

fn offset(text_bounds: Rectangle, value: &str, state: &Internal, cell: &Cell) -> f32 {
    if state.is_focused() {
        let cursor = state.cursor;

        let focus_position = match cursor.state(value) {
            utils::State::Index(i) => i,
            utils::State::Selection { end, .. } => end,
        };

        let (_, offset) = measure_cursor_and_scroll_offset(cell.raw(), text_bounds, focus_position);

        offset
    } else {
        0.0
    }
}

fn find_cursor_position(
    text_bounds: Rectangle,
    value: &str,
    state: &Internal,
    cell: &Cell,
    x: f32,
) -> Option<usize> {
    let offset = offset(text_bounds, value, state, cell);
    let value = value.to_string();

    let char_offset = cell
        .raw()
        .hit_test(Point::new(x + offset, text_bounds.height / 2.0))
        .map(text::Hit::cursor)?;

    let res = value[..char_offset.min(value.len())].len();

    Some(res)
}

fn word_boundary(text: &str, index: usize) -> (usize, usize) {
    if index >= text.len() {
        return (text.len(), text.len());
    }

    let chars = text.chars().collect::<Vec<char>>();
    let len = chars.len();

    if !chars[index].is_alphanumeric() && chars[index] != '_' {
        return (index, index);
    }

    let mut start = index;
    let mut end = index;

    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }

    while end < len && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }

    if end + 1 < len {
        end += 1;
    }

    (start, end)
}

/// Returns true if the provided `character` is accepted by the given `DataType`
fn column_filter(kind: DataType, character: char) -> bool {
    match kind {
        DataType::Text => true,
        DataType::I32 | DataType::ISize => {
            character.is_digit(10) || character == '-' || character == '_'
        }
        DataType::U32 | DataType::USize => character.is_digit(10) || character == '_',
        DataType::F32 | DataType::F64 => {
            character.is_digit(10) || character == '-' || character == '_'
        }
        DataType::Bool => {
            let chars = [
                't', 'T', 'r', 'R', 'u', 'U', 'e', 'E', 'f', 'F', 'a', 'A', 'l', 'L', 's', 'S',
            ];

            chars.contains(&character)
        }
    }
}
