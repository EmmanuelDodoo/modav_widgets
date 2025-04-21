use iced::{
    advanced::{
        self,
        layout::{self, Limits, Node},
        mouse::{self, click},
        renderer::Quad,
        text::Paragraph,
        Shell,
    },
    alignment::Horizontal,
    event, font, keyboard,
    time::{Duration, Instant},
    touch, window, Background, Color, Event, Font, Padding, Pixels, Point, Rectangle, Renderer,
    Size, Vector,
};

use super::style::{Catalog, Style};
use super::utils::{self, Editor, KeyPress, Resizing, Selection};
use super::{
    alignment_offset, cell_to_string, column_filter, draw, find_cursor_position, gen_pagination,
    measure_cursor_and_scroll_offset, type_alignment, word_boundary, Cell, Table,
    PAGINATION_ELLIPSIS,
};

const BACK: &str = "‹ Back";
const NEXT: &str = "Next ›";
const GOTO_PAGE: &str = "Page:";
const GOTO_GO: &str = "Go";
const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;

pub struct State {
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

impl State {
    /// The maximum number of page numbers displayed
    const PAGINATION_LIMIT: usize = 11;
    /// The maximum size of a cell
    const MAX_CELL: Size = Size::new(f32::INFINITY, 45.0);
    /// Multiplier for each scroll step
    const SCROLL_MULT: f32 = 5.0;
    /// Spacing between cells
    const CELL_GAP: f32 = 3.5;
    /// Multiplier for column kind text size.
    const KIND_MULT: f32 = 0.9;

    pub fn new<Message, Theme: Catalog>(table: &Table<'_, Message, Theme>) -> Self {
        let pages_padding = Padding::from([2, 6]);
        let size = table.text_size * 7.0 / 8.0;

        let dimensions = (table.raw.height(), table.raw.width());

        let headers = vec![(Cell::default(), Cell::default()); dimensions.0];

        let limit = table.page_limit;
        let min_widths = vec![0.0f32; dimensions.1 + 1];
        let min_heights = vec![0.0f32; limit + 1];

        let numbering = vec![Cell::default(); limit + 1];
        let pages_end = if limit == 0 {
            0
        } else {
            (table.raw.height() / limit) + 1
        };
        let paginations =
            vec![(Cell::default(), String::default()); Self::PAGINATION_LIMIT.min(pages_end)];

        let status = {
            let value = match table.status.as_ref() {
                Some(status) => status.clone(),
                None => format!("{} rows × {} columns", dimensions.0, dimensions.1),
            };
            let text = super::text(&value, Self::MAX_CELL, table.font, Horizontal::Left, size);
            (Cell::new(text), value)
        };

        let cells = vec![Cell::default(); limit * dimensions.1];

        let back = {
            let text = super::text(BACK, Self::MAX_CELL, table.font, Horizontal::Center, size);
            Cell::new(text)
        };

        let next = {
            let text = super::text(NEXT, Self::MAX_CELL, table.font, Horizontal::Center, size);
            Cell::new(text)
        };

        let goto_page = {
            let text = super::text(
                GOTO_PAGE,
                Self::MAX_CELL,
                table.font,
                Horizontal::Center,
                size,
            );
            Cell::new(text)
        };

        let goto_go = {
            let text = super::text(
                GOTO_GO,
                Self::MAX_CELL,
                table.font,
                Horizontal::Center,
                size,
            );
            Cell::new(text)
        };

        let goto_input = {
            let value = String::from("1");
            let text = super::text(&value, Self::MAX_CELL, table.font, Horizontal::Center, size);
            (Cell::new(text), value)
        };

        Self {
            //raw,
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
        self.rows / self.page_limit
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused.is_some()
    }

    pub fn cursor(&self) -> utils::Cursor {
        self.cursor
    }

    fn _reset_status(&mut self, font: Font) {
        let value = format!("{} rows × {} columns", self.rows, self.cols);

        let text = super::text(
            &value,
            Self::MAX_CELL,
            font,
            Horizontal::Left,
            self.page_size,
        );

        let (cell, status) = &mut self.status;

        cell.update(text);

        *status = value;
    }

    /// Resets both editing and resizing
    fn reset(&mut self) {
        self.reset_resizing();
        self.reset_editing();
        self.reset_selection();
        self.last_click = None;
        self.is_focused = None;
        self.keyboard_modifiers = keyboard::Modifiers::default()
    }

    fn reset_editing(&mut self) {
        self.is_text_dragging = false;
        self.editing = None;
        self.cursor = utils::Cursor::default();
    }

    fn reset_resizing(&mut self) {
        self.resizing = None;
    }

    fn reset_selection(&mut self) {
        self.selection = None;
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

    fn layout_cells<Message, Theme: Catalog>(&mut self, table: &Table<'_, Message, Theme>) -> Node {
        let font = table.font;
        let padding = table.cell_padding;
        let size = table.text_size;

        let gap = Self::CELL_GAP;
        // Adds numbering column
        let dimensions = (self.rows, self.cols + 1);
        // Adds headers row
        let page_limit = self.page_limit + 1;

        let numbering_max = dimensions.0;
        let numbering_max = Cell::new(super::text(
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
                let col = table.raw.get_col(column).expect("Missing column in sheet");
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
                    let text = super::text(label, Self::MAX_CELL, font, Horizontal::Center, size);
                    header.update(text);
                    let font = Font {
                        style: font::Style::Italic,
                        ..font
                    };
                    let text = super::text(
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

                    let text = super::text(value, Self::MAX_CELL, font, horizontal, size);
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

                paragraph.update(super::text(
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

    fn layout_pagination<Message, Theme: Catalog>(
        &mut self,
        table: &Table<'_, Message, Theme>,
    ) -> Node {
        if table.raw.is_empty() {
            return Node::with_children(Size::ZERO, vec![Node::default(); 3]);
        }

        let font = table.font;
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
            let text = super::text(
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

    fn layout_goto<Message, Theme: Catalog>(&mut self, table: &Table<'_, Message, Theme>) -> Node {
        if table.raw.is_empty() {
            return Node::with_children(Size::ZERO, vec![Node::default(); 3]);
        }
        let font = table.font;
        let max = Cell::new(super::text(
            &(self.pages_end() + 1).to_string(),
            Self::MAX_CELL,
            font,
            Horizontal::Right,
            self.page_size,
        ));

        let page = self.goto_page.min_bounds().expand(self.pages_padding);
        let go = self.goto_go.min_bounds().expand(self.pages_padding);

        let (input, value) = &mut self.goto_input;

        input.update(super::text(
            value,
            Self::MAX_CELL,
            font,
            Horizontal::Right,
            self.page_size,
        ));

        let min_bounds = max.min_bounds();
        let input = Size::new(min_bounds.width + 5.0, min_bounds.height).expand(self.pages_padding);

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

    fn layout_status<Message, Theme: Catalog>(
        &mut self,
        table: &Table<'_, Message, Theme>,
        max_width: f32,
    ) -> Node {
        if table.raw.is_empty() {
            return Node::default();
        }

        let bounds = Size::new(max_width, f32::INFINITY);
        let (cell, value) = &mut self.status;
        let value = match table.status.as_ref() {
            Some(status) => status,
            None => value,
        };

        cell.update(super::text(
            value,
            bounds,
            table.font,
            Horizontal::Center,
            self.page_size,
        ));

        let size = cell.min_bounds().expand(self.pages_padding);

        Node::new(size)
    }

    pub fn layout<Message, Theme: Catalog>(
        &mut self,
        table: &Table<'_, Message, Theme>,
        limits: Limits,
    ) -> Node {
        let spacing = if table.raw.is_empty() {
            0.0
        } else {
            table.spacing
        };
        let padding = table.padding;

        let content_limits = limits
            .width(table.width)
            .height(table.height)
            .shrink(table.padding);

        let mut pagination = if self.multiple_pages() {
            self.layout_pagination(table)
        } else {
            Node::default()
        };
        let pagination_size = pagination.size();

        let mut goto = if self.multiple_pages() {
            self.layout_goto(table)
        } else {
            Node::default()
        };
        let goto_size = goto.size();

        let actions = Size::new(
            pagination_size.width + spacing + goto_size.width,
            pagination_size.height.max(goto_size.height),
        );

        let actions_spacing = if self.multiple_pages() { spacing } else { 0.0 };

        let mut status = self.layout_status(table, content_limits.max().width);
        let status_size = status.size();
        status.translate_mut(Vector::new(
            padding.left,
            padding.top + actions.height + actions_spacing,
        ));

        let cells = self.layout_cells(table).translate(Vector::new(
            padding.left,
            padding.top + actions.height + actions_spacing + status_size.height + spacing,
        ));
        let cells_size = cells.size();

        let total_size = Size::new(
            actions.width.max(cells_size.width),
            actions.height + actions_spacing + status_size.height + spacing + cells_size.height,
        )
        .expand(padding);

        let size = limits.resolve(table.width, table.height, total_size);

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
        style: Style,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((cell, content), layout) in self.paginations.iter().zip(layout.children()) {
            let bounds = layout.bounds();
            let (background, text_color) = if (self.page + 1).to_string() == *content {
                (style.selected_page_background, style.selected_page_text)
            } else if cursor.is_over(bounds) {
                (style.hovered_page_background, style.hovered_page_text)
            } else {
                (style.page_background, style.page_text)
            };

            if let Some(clipped_viewport) = bounds.intersection(viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds: clipped_viewport,
                        border: style.page_border,
                        ..Default::default()
                    },
                    background,
                );

                draw(
                    renderer,
                    text_color,
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
        style: Style,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        {
            let back = children.next().expect("Missing paginations: Back");

            let (background, text_color) = if self.page == 0 {
                (
                    style.pagination_background.scale_alpha(0.5),
                    style.pagination_text.scale_alpha(0.5),
                )
            } else if cursor.is_over(back.bounds()) {
                (
                    style.hovered_pagination_background,
                    style.hovered_pagination_text,
                )
            } else {
                (style.pagination_background, style.pagination_text)
            };

            if let Some(bounds) = back.bounds().intersection(viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds,
                        border: style.pagination_border,
                        ..Default::default()
                    },
                    background,
                );
                draw(
                    renderer,
                    text_color,
                    back,
                    self.page_back.raw(),
                    self.pages_padding,
                    viewport,
                );
            }
        };

        let pages = children.next().expect("Missing paginations: Pages");

        self.draw_pages(renderer, pages, style, cursor, viewport);

        {
            let next = children.next().expect("Missing paginations: Next");

            let (background, text_color) = if self.page == self.pages_end() {
                (
                    style.pagination_background.scale_alpha(0.5),
                    style.pagination_text.scale_alpha(0.5),
                )
            } else if cursor.is_over(next.bounds()) {
                (
                    style.hovered_pagination_background,
                    style.hovered_pagination_text,
                )
            } else {
                (style.pagination_background, style.pagination_text)
            };

            if let Some(bounds) = next.bounds().intersection(viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds,
                        border: style.pagination_border,
                        ..Default::default()
                    },
                    background,
                );
                draw(
                    renderer,
                    text_color,
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
        style: Style,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        {
            let page = children.next().expect("Widget draw: Missing Goto Page");

            if let Some(bounds) = page.bounds().intersection(viewport) {
                draw(
                    renderer,
                    style.goto_page_text,
                    page,
                    self.goto_page.raw(),
                    self.pages_padding,
                    &bounds,
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
                    style.goto_input_background,
                );
                draw(
                    renderer,
                    style.goto_input_text,
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
                let (background, text_color) = if cursor.is_over(go.bounds()) {
                    (style.hovered_goto_background, style.hovered_goto_text)
                } else {
                    (style.goto_background, style.goto_text)
                };
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds,
                        border: style.goto_border,
                        ..Default::default()
                    },
                    background,
                );
                draw(
                    renderer,
                    text_color,
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
        style: Style,
        viewport: &Rectangle,
    ) {
        if let Some(bounds) = layout.bounds().intersection(viewport) {
            <Renderer as advanced::Renderer>::fill_quad(
                renderer,
                Quad {
                    bounds,
                    ..Default::default()
                },
                style.status_background,
            );

            draw(
                renderer,
                style.status_text,
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
        style: Style,
        viewport: Rectangle,
        padding: Padding,
    ) {
        let og_viewport = viewport;
        if let Some(clipped) = layout.bounds().intersection(&viewport) {
            <Renderer as advanced::Renderer>::fill_quad(
                renderer,
                Quad {
                    bounds: clipped,
                    ..Default::default()
                },
                style.cell_border,
            );
        }

        let mut editing: Option<Rectangle> = None;

        let mut children = layout.children();
        let numbering = children
            .next()
            .expect("Widget draw: Missing numbering cells");
        let headers = children.next().expect("Widget draw: Missing header cells");
        let cells = children.next().expect("Widget draw: Missing cells layout");

        let mut top_left: Option<Size> = None;

        let numbering_viewport = {
            let moved = viewport + Vector::new(0.0, headers.bounds().height);

            let size = Size::new(moved.width, moved.height - headers.bounds().height);

            Rectangle::new(moved.position(), size)
        };

        for (idx, (number, layout)) in self.numbering.iter().zip(numbering.children()).enumerate() {
            let bounds = layout.bounds();

            if let Some(clipped_viewport) = bounds.intersection(&numbering_viewport) {
                let child = layout
                    .children()
                    .next()
                    .expect("Table draw: Resize node missing child layout");

                top_left = Some(Size::new(child.bounds().width, 0.0));

                if let Some(clipped_viewport) = child.bounds().intersection(&clipped_viewport) {
                    let (background, text_color) = if idx % 2 == 1 {
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

                    <Renderer as advanced::Renderer>::fill_quad(
                        renderer,
                        Quad {
                            bounds: clipped_viewport,
                            ..Default::default()
                        },
                        background,
                    );

                    draw(
                        renderer,
                        text_color,
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

            let is_selected = self
                .selection
                .as_ref()
                .map(|selection| selection.header(idx))
                .unwrap_or_default();

            top_left = top_left.map(|size| Size::new(size.width, pair.bounds().height));

            if is_selected {
                let bounds = pair.bounds().expand([Self::CELL_GAP, Self::CELL_GAP]);
                if let Some(clipped_viewport) = bounds.intersection(&viewport) {
                    <Renderer as advanced::Renderer>::fill_quad(
                        renderer,
                        Quad {
                            bounds: clipped_viewport,
                            ..Default::default()
                        },
                        style.selected_header_border,
                    );
                }
            }

            if let Some(clipped_viewport) = pair.bounds().intersection(&viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds: clipped_viewport,
                        ..Default::default()
                    },
                    style.header_background,
                );
            }

            if let Some(label_viewport) = label.bounds().intersection(&viewport) {
                draw(
                    renderer,
                    style.header_text,
                    label,
                    header.raw(),
                    Padding::from(0),
                    &label_viewport,
                )
            }

            if let Some(kind_viewport) = knd.bounds().intersection(&viewport) {
                draw(
                    renderer,
                    style.header_type,
                    knd,
                    kind.raw(),
                    Padding::from(0),
                    &kind_viewport,
                )
            }

            if let Some(Editing::Cell {
                index,
                is_header: true,
                ..
            }) = &self.editing
            {
                if idx == *index {
                    editing.replace(label.bounds());
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
                    Background::Color(Color::TRANSPARENT),
                );

                let (row, column) = (idx % self.page_limit, idx / self.page_limit);

                let (selection, is_selected) = self
                    .selection
                    .as_ref()
                    .map(|selection| {
                        (
                            selection.border(row, column),
                            selection.contains(row, column),
                        )
                    })
                    .unwrap_or_default();

                let selection = {
                    let mut padding = Padding::ZERO;

                    if (selection & 1) == 1 {
                        padding = padding.left(Self::CELL_GAP);
                    }

                    if ((selection >> 1) & 1) == 1 {
                        padding = padding.top(Self::CELL_GAP);
                    }

                    if ((selection >> 2) & 1) == 1 {
                        padding = padding.right(Self::CELL_GAP);
                    }

                    if ((selection >> 3) & 1) == 1 {
                        padding = padding.bottom(Self::CELL_GAP);
                    }

                    padding
                };

                if let Some(selection_viewport) =
                    child.bounds().expand(selection).intersection(&viewport)
                {
                    <Renderer as advanced::Renderer>::fill_quad(
                        renderer,
                        Quad {
                            bounds: selection_viewport,
                            border: iced::Border::default().rounded(2.0),
                            ..Default::default()
                        },
                        style.selected_cell_border,
                    );
                }

                if let Some(clipped_viewport) = child.bounds().intersection(&clipped_viewport) {
                    let row = idx % self.page_limit;

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

                    <Renderer as advanced::Renderer>::fill_quad(
                        renderer,
                        Quad {
                            bounds: clipped_viewport,
                            ..Default::default()
                        },
                        cell_background,
                    );

                    if is_selected && self.editing.is_none() {
                        <Renderer as advanced::Renderer>::fill_quad(
                            renderer,
                            Quad {
                                bounds: clipped_viewport,
                                ..Default::default()
                            },
                            style.selected_cell_background,
                        );
                    }

                    draw(
                        renderer,
                        text_color,
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

        if let Some(size) = top_left {
            let bounds = Rectangle::new(layout.position(), size);

            if let Some(clipped) = bounds.intersection(&og_viewport) {
                <Renderer as advanced::Renderer>::fill_quad(
                    renderer,
                    Quad {
                        bounds: clipped,
                        ..Default::default()
                    },
                    style.header_background,
                );
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
                        style,
                        cell,
                        clipped_bounds,
                        bounds,
                        value,
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
                        style,
                        cell,
                        clipped_bounds,
                        bounds,
                        value,
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
        style: Style,
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
                                    width: 1.0,
                                    height,
                                },
                                ..Quad::default()
                            },
                            style.cursor_color,
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
                            style.cursor_selection,
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

    pub fn draw<Message, Theme: Catalog>(
        &self,
        table: &Table<'_, Message, Theme>,
        renderer: &mut Renderer,
        layout: layout::Layout<'_>,
        style: Style,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let padding = table.padding;
        let spacing = table.spacing;

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
            self.draw_cells(renderer, cells, style, clipped_viewport, table.cell_padding)
        };

        self.draw_status(renderer, status, style, viewport);

        if self.multiple_pages() {
            self.draw_pagination(renderer, pagination, style, cursor, viewport);

            self.draw_goto(renderer, goto, style, cursor, viewport);
        }

        if let Some(Editing::Goto(bounds)) = &self.editing {
            self.draw_edit(
                renderer,
                style,
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

        for (idx, resize) in headers.children().enumerate() {
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

            match &self.editing {
                Some(Editing::Cell {
                    index,
                    is_header: true,
                    ..
                }) if *index == idx && cursor.is_over(label) => {
                    return mouse::Interaction::Text;
                }
                _ if cursor.is_over(pair) => {
                    return mouse::Interaction::Cell;
                }
                _ if cursor.is_over(resize) => {
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
                _ => {}
            }
        }

        let cells = children.next().expect("Widget Interaction: Missing cells");

        for (idx, cell) in cells.children().enumerate() {
            let resize = cell.bounds();
            let child = cell
                .children()
                .next()
                .expect("Table Interaction: Resize node missing child layout")
                .bounds();

            match &self.editing {
                Some(Editing::Cell {
                    index,
                    is_header: false,
                    ..
                }) if *index == idx && cursor.is_over(child) => {
                    return mouse::Interaction::Text;
                }
                _ if cursor.is_over(child) => {
                    return mouse::Interaction::Cell;
                }
                _ if cursor.is_over(resize) => {
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
                _ => {}
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

        if cursor.is_over(back.bounds()) && self.page != 0 {
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

        if cursor.is_over(next.bounds()) && self.page != self.pages_end() {
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

    pub fn mouse_interaction(
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

    fn update_cells_click<Message, Theme: Catalog>(
        &mut self,
        table: &Table<'_, Message, Theme>,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let padding = table.cell_padding;
        let mut children = layout.children();
        let numbering = children
            .next()
            .expect("Widget Update: Missing numbering cells");

        if let Some((idx, numbering)) = numbering
            .children()
            .enumerate()
            .filter(|(idx, _)| *idx != 0)
            .find(|(_, child)| cursor.is_over(child.bounds()))
        {
            let row = idx - 1;
            let bounds = numbering.bounds();
            // Guaranteed by the find above
            let cursor_position = cursor.position_over(bounds).unwrap();
            let click = mouse::Click::new(cursor_position, mouse::Button::Left, self.last_click);

            self.last_click = Some(click);
            self.reset_editing();
            self.selection
                .replace(Selection::row(row, self.cols.saturating_sub(1)));
            if let Some(callback) = table.on_selection.as_ref() {
                // Guaranteed by the Selection::row above
                let msg = callback(self.selection.clone().unwrap());
                shell.publish(msg);
            }
            return event::Status::Captured;
        }

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
                let cell = cell
                    .children()
                    .next()
                    .expect("Table Update: Resize node missing child layout");

                let cursor_position = cursor.position_over(cell.bounds());

                let (row, column) = if is_header {
                    (0, idx + 1)
                } else {
                    let idx = idx - self.cols;
                    let column = (idx / self.page_limit) + 1;
                    let row = (idx + 1) - ((idx / self.page_limit) * self.page_limit);
                    (row, column)
                };

                let resize = Resizing::new(cell_bounds, cell.bounds(), cursor, row, column);

                if resize.is_some() {
                    self.resizing = resize;
                    self.reset_editing();
                    return event::Status::Captured;
                }

                let Some(cursor_position) = cursor_position else {
                    return event::Status::Ignored;
                };

                let click =
                    mouse::Click::new(cursor_position, mouse::Button::Left, self.last_click);

                let (row, column) = if is_header {
                    (0, idx)
                } else {
                    let idx = idx - self.cols;
                    let column = idx / self.page_limit;
                    let row = idx % self.page_limit;
                    (row, column)
                };

                let cell_bounds = cell.bounds().shrink(padding);

                let Some(cursor_position) = cursor.position_over(cell_bounds) else {
                    return event::Status::Ignored;
                };

                let (idx, cell, value) = if is_header {
                    let (cell, _) = &self.headers[idx];
                    let col = table
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

                    let col = table
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

                let (editing_idx, editing_is_header) = match self.editing.as_ref() {
                    Some(Editing::Cell {
                        index, is_header, ..
                    }) => (Some(*index), *is_header),
                    _ => (None, false),
                };

                match click.kind() {
                    click::Kind::Single if self.keyboard_modifiers.shift() && !is_header => {
                        self.last_click = Some(click);
                        if let Some(selection) = self.selection.as_mut() {
                            selection.block(row, column);

                            if let Some(callback) = table.on_selection.as_ref() {
                                let msg = callback(selection.clone());
                                shell.publish(msg);
                            }

                            self.reset_editing();
                            return event::Status::Captured;
                        }
                    }
                    click::Kind::Single if self.keyboard_modifiers.command() && !is_header => {
                        self.last_click = Some(click);
                        if let Some(selection) = self.selection.as_mut() {
                            selection.scattered(row, column);

                            if let Some(callback) = table.on_selection.as_ref() {
                                let msg = callback(selection.clone());
                                shell.publish(msg);
                            }

                            self.reset_editing();
                            return event::Status::Captured;
                        }
                    }
                    click::Kind::Single
                        if editing_idx.is_some()
                            && editing_idx.unwrap() == idx
                            && is_header == editing_is_header =>
                    {
                        // Needs to be in sync with kind::Double
                        let position = if target > 0.0 {
                            find_cursor_position(cell_bounds, &value, self, cell, target)
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

                        self.last_click = Some(click);
                        self.editing = Some(Editing::Cell {
                            index: idx,
                            value,
                            is_header,
                        });

                        return event::Status::Captured;
                    }
                    click::Kind::Single if is_header => {
                        self.last_click = Some(click);
                        self.reset_editing();
                        self.selection.replace(Selection::column(
                            column,
                            (self.page_limit * (self.page + 1)).saturating_sub(1),
                        ));

                        if let Some(callback) = table.on_selection.as_ref() {
                            // Guaranteed by the Selection::column above
                            let msg = callback(self.selection.clone().unwrap());
                            shell.publish(msg);
                        }

                        return event::Status::Captured;
                    }
                    click::Kind::Single => {
                        self.last_click = Some(click);
                        match editing_idx {
                            Some(index) if is_header == editing_is_header && index == idx => {}
                            _ => self.reset_editing(),
                        }
                        self.selection.replace(Selection::new(row, column));
                        if let Some(callback) = table.on_selection.as_ref() {
                            // Guaranteed by the Selection::new above
                            let msg = callback(self.selection.clone().unwrap());
                            shell.publish(msg);
                        }
                        return event::Status::Captured;
                    }
                    click::Kind::Double if self.editing.is_some() => {
                        let position =
                            find_cursor_position(cell_bounds, &value, self, cell, target)
                                .unwrap_or(0);
                        let (start, end) = word_boundary(&value, position);
                        self.cursor.select_range(start, end);
                        self.is_text_dragging = false;

                        self.last_click = Some(click);
                        self.editing = Some(Editing::Cell {
                            index: idx,
                            value,
                            is_header,
                        });
                        return event::Status::Captured;
                    }
                    click::Kind::Double => {
                        // Needs to be in sync with kind::Single
                        // editing.is_some()
                        let position = if target > 0.0 {
                            find_cursor_position(cell_bounds, &value, self, cell, target)
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

                        self.last_click = Some(click);
                        self.editing = Some(Editing::Cell {
                            index: idx,
                            value,
                            is_header,
                        });

                        return event::Status::Captured;
                    }
                    click::Kind::Triple if self.editing.is_some() => {
                        self.cursor.select_all(&value);
                        self.is_text_dragging = false;

                        self.last_click = Some(click);
                        self.editing = Some(Editing::Cell {
                            index: idx,
                            value,
                            is_header,
                        });

                        return event::Status::Captured;
                    }
                    // todo!: Cannot realistically trigger this condition atm
                    click::Kind::Triple => {
                        self.last_click = Some(click);
                        self.reset_editing();
                        self.selection
                            .replace(Selection::row(row, self.cols.saturating_sub(1)));
                        if let Some(callback) = table.on_selection.as_ref() {
                            // Guaranteed by the Selection::row above
                            let msg = callback(self.selection.clone().unwrap());
                            shell.publish(msg);
                        }
                        return event::Status::Captured;
                    }
                }

                event::Status::Ignored
            }
            None => {
                self.reset();

                event::Status::Ignored
            }
        }
    }

    fn update_cells<Message, Theme: Catalog>(
        &mut self,
        table: &Table<'_, Message, Theme>,
        event: event::Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
        scroll_bounds: Size,
    ) -> event::Status {
        if table.raw.is_empty() {
            return event::Status::Ignored;
        }

        let font = table.font;
        let size = table.text_size;
        let padding = table.cell_padding;

        if matches!(
            &event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. })
        ) {
            return self.update_cells_click(table, layout, cursor, shell);
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

                        let Some(cursor_position) = cursor.position_over(cell_bounds) else {
                            return event::Status::Ignored;
                        };

                        let (idx, cell, value) = if is_header {
                            let (cell, _) = &self.headers[idx];
                            let col = table
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

                            let col = table
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
                                    find_cursor_position(cell_bounds, &value, self, cell, target)
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

                        event::Status::Captured
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

                let position = find_cursor_position(bounds, value, self, cell, target).unwrap_or(0);

                self.cursor.select_range(self.cursor.start(value), position);

                event::Status::Captured
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
                event::Status::Captured
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

                let (cell, col_kind, row, column) = if *is_header {
                    let (cell, _) = &mut self.headers[index];
                    let col = table
                        .raw
                        .get_col(index)
                        .expect("Cells update: Missing column in sheet")
                        .kind();
                    (cell, col, 0, index + 1)
                } else {
                    let cell = &mut self.cells[index];
                    let (row, column) = (index % self.page_limit, index / self.page_limit);
                    let row = row + (self.page * self.page_limit);

                    let col = table
                        .raw
                        .get_col(column)
                        .expect("Cells update: Missing column in sheet")
                        .kind();

                    (cell, col, row, column)
                };

                if key.as_ref() == keyboard::Key::Character("a") && modifiers.command() {
                    self.cursor.select_all(value);
                    return event::Status::Captured;
                }

                match text {
                    Some(text) if *is_header => {
                        if let Some(c) = text.chars().next().filter(|c| !c.is_control()) {
                            let mut editor = Editor::new(value, &mut self.cursor);
                            editor.insert(c);

                            cell.update(super::text(
                                value,
                                Self::MAX_CELL,
                                font,
                                cell.horizontal_alignment(),
                                size,
                            ));

                            focus.updated_at = Instant::now();

                            if let Some(callback) = table.on_header_input.as_ref() {
                                let msg = callback(value.clone(), column.saturating_sub(1));
                                shell.publish(msg);
                            }

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

                            cell.update(super::text(
                                value,
                                Self::MAX_CELL,
                                font,
                                cell.horizontal_alignment(),
                                size,
                            ));

                            focus.updated_at = Instant::now();

                            if let Some(callback) = table.on_cell_input.as_ref() {
                                let msg = callback(value.clone(), row, column);
                                shell.publish(msg);
                            }

                            let column = column + 1;
                            let row = (index % self.page_limit) + 1;
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
                            if let Some(callback) = table.on_header_submit.as_ref() {
                                let msg = callback(value.clone(), column - 1);
                                shell.publish(msg)
                            }
                        } else if let Some(callback) = table.on_cell_submit.as_ref() {
                            let msg = callback(value.clone(), row, column);
                            shell.publish(msg);
                        }

                        self.reset();
                        shell.invalidate_layout();
                        event::Status::Captured
                    }
                    keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                        let mut editor = Editor::new(value, &mut self.cursor);
                        editor.backspace();

                        cell.update(super::text(
                            value,
                            Self::MAX_CELL,
                            font,
                            cell.horizontal_alignment(),
                            size,
                        ));

                        if *is_header {
                            if let Some(callback) = table.on_header_input.as_ref() {
                                let msg = callback(value.clone(), column.saturating_sub(1));
                                shell.publish(msg);
                            }
                        } else if let Some(callback) = table.on_cell_input.as_ref() {
                            let msg = callback(value.clone(), row, column);
                            shell.publish(msg)
                        }

                        event::Status::Captured
                    }
                    keyboard::Key::Named(keyboard::key::Named::Delete) => {
                        let mut editor = Editor::new(value, &mut self.cursor);
                        editor.delete();

                        cell.update(super::text(
                            value,
                            Self::MAX_CELL,
                            font,
                            cell.horizontal_alignment(),
                            size,
                        ));

                        if *is_header {
                            if let Some(callback) = table.on_header_input.as_ref() {
                                let msg = callback(value.clone(), column.saturating_sub(1));
                                shell.publish(msg);
                            }
                        } else if let Some(callback) = table.on_cell_input.as_ref() {
                            let msg = callback(value.clone(), row, column);
                            shell.publish(msg)
                        }

                        event::Status::Captured
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                        if modifiers.shift() {
                            self.cursor.select_left(value);
                        } else {
                            self.cursor.move_left(value);
                        }

                        event::Status::Captured
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        if modifiers.shift() {
                            self.cursor.select_right(value);
                        } else {
                            self.cursor.move_right(value);
                        }

                        event::Status::Captured
                    }
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        self.reset();
                        event::Status::Captured
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        if modifiers.shift() {
                            self.cursor.select_to_start(value);
                        } else {
                            self.cursor.move_to(0);
                        }

                        event::Status::Captured
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        if modifiers.shift() {
                            self.cursor.select_to_end(value);
                        } else {
                            self.cursor.move_to_end(value);
                        }

                        event::Status::Captured
                    }
                    keyboard::Key::Named(keyboard::key::Named::Tab) => event::Status::Ignored,

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

                if cursor.is_over(next.bounds()) && self.page < self.pages_end() {
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

    fn update_goto<Message, Theme: Catalog>(
        &mut self,
        table: &Table<'_, Message, Theme>,
        event: event::Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let font = table.font;

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
                                        value,
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

                        event::Status::Captured
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

                self.cursor.select_range(self.cursor.start(value), position);

                event::Status::Captured
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
                        .filter(|c| !c.is_control() && c.is_ascii_digit())
                    {
                        let mut editor = Editor::new(value, &mut self.cursor);

                        editor.insert(c);

                        let pages_end = table.raw.height() / self.page_limit;
                        match value.parse::<usize>() {
                            Ok(page) if page > pages_end => *value = (pages_end + 1).to_string(),
                            Err(_) if value.is_empty() => {
                                *value = (self.page + 1).to_string();
                            }
                            _ => {}
                        }

                        cell.update(super::text(
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
                        cell.update(super::text(
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
                        cell.update(super::text(
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
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        self.cursor.move_to(0);
                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        self.cursor.move_to_end(value);
                        return event::Status::Captured;
                    }
                    keyboard::Key::Named(keyboard::key::Named::Tab) => {
                        return event::Status::Ignored;
                    }

                    _ => {}
                }

                event::Status::Captured
            }
            _ => event::Status::Ignored,
        }
    }

    pub fn on_update<Message, Theme: Catalog>(
        &mut self,
        table: &Table<'_, Message, Theme>,
        event: event::Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let padding = table.padding;
        let spacing = table.spacing;

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
                    return self.update_cells(table, event, cells, cursor, shell, scroll_bounds);
                }

                if cursor.is_over(pagination.bounds()) && self.multiple_pages() {
                    self.reset();
                    return self.update_pagination(event, pagination, cursor, shell);
                }

                if cursor.is_over(goto.bounds()) && self.multiple_pages() {
                    return self.update_goto(table, event, goto, cursor, shell);
                }

                match self.editing.take() {
                    Some(Editing::Cell {
                        index,
                        value,
                        is_header,
                        ..
                    }) => {
                        if is_header {
                            if let Some(callback) = table.on_header_submit.as_ref() {
                                let msg = callback(value, index);
                                shell.publish(msg);
                            }
                        } else {
                            let (row, column) = (index % self.page_limit, index / self.page_limit);

                            if let Some(callback) = table.on_cell_submit.as_ref() {
                                let msg = callback(value, row, column);
                                shell.publish(msg);
                            }
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
                        return self.update_goto(table, event, goto, cursor, shell);
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
                            table,
                            event,
                            cells,
                            cursor,
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
                return self.update_cells(table, event, cells, cursor, shell, scroll_bounds);
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
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                text,
                ..
            }) if self.editing.is_none() && cursor.is_over(layout.bounds()) => {
                if let Some(callback) = table.on_keypress.as_ref() {
                    let msg = callback(KeyPress {
                        key: key.clone(),
                        modifiers: *modifiers,
                        text: text.as_ref().map(|text| text.to_string()),
                    });

                    if let Some(msg) = msg {
                        shell.publish(msg);
                        return event::Status::Ignored;
                    }
                }

                let Some(selection) = self.selection.as_mut() else {
                    return event::Status::Ignored;
                };

                match key {
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight)
                        if self.keyboard_modifiers.shift() =>
                    {
                        selection.grow(
                            0,
                            self.page_limit.saturating_sub(1),
                            1,
                            self.cols.saturating_sub(1),
                        );
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                        selection.move_right(self.cols.saturating_sub(1))
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft)
                        if self.keyboard_modifiers.shift() =>
                    {
                        selection.shrink(0, 1)
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => selection.move_left(),
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown)
                        if self.keyboard_modifiers.shift() =>
                    {
                        selection.grow(
                            1,
                            self.page_limit.saturating_sub(1),
                            0,
                            self.cols.saturating_sub(1),
                        );
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown)
                    | keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        selection.move_down(self.page_limit.saturating_sub(1))
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp)
                        if self.keyboard_modifiers.shift() =>
                    {
                        selection.shrink(1, 0)
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => selection.move_up(),
                    _ => return event::Status::Ignored,
                }

                if let Some(callback) = table.on_selection.as_ref() {
                    let msg = callback(selection.clone());
                    shell.publish(msg);
                }
                return event::Status::Captured;
            }
            Event::Keyboard(keyboard::Event::KeyPressed { .. }) => match self.editing {
                Some(Editing::Goto(_)) => {
                    return self.update_goto(table, event, goto, cursor, shell)
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
                    return self.update_cells(table, event, cells, cursor, shell, scroll_bounds);
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
