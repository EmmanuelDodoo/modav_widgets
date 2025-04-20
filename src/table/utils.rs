use iced::{color, mouse, Background, Border, Color, Point, Rectangle, Size, Theme, Vector};
use std::collections::HashSet;
use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy)]
pub enum State {
    Index(usize),
    Selection { start: usize, end: usize },
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            state: State::Index(0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    state: State,
}

impl Cursor {
    pub fn state(&self, value: &str) -> State {
        let len = value.len();
        match self.state {
            State::Index(idx) => State::Index(idx.min(len)),
            State::Selection { start, end } => {
                let start = start.min(len);
                let end = end.min(len);

                if start == end {
                    State::Index(start)
                } else {
                    State::Selection { start, end }
                }
            }
        }
    }

    pub fn selection(&self, value: &str) -> Option<(usize, usize)> {
        match self.state(value) {
            State::Selection { start, end } => Some((start.min(end), start.max(end))),
            State::Index(_) => None,
        }
    }

    pub fn move_to(&mut self, position: usize) {
        self.state = State::Index(position);
    }

    pub fn move_to_end(&mut self, value: &str) {
        self.state = State::Index(value.len());
    }

    pub fn move_left(&mut self, value: &str) {
        match self.state(value) {
            State::Index(idx) if idx > 0 => self.move_to(idx - 1),
            State::Selection { start, end } => self.move_to(start.min(end)),
            State::Index(_) => self.move_to(0),
        }
    }

    pub fn move_right(&mut self, value: &str) {
        self.move_right_by_amount(value, 1)
    }

    pub fn move_right_by_amount(&mut self, value: &str, amount: usize) {
        match self.state(value) {
            State::Index(idx) => self.move_to(idx.saturating_add(amount).min(value.len())),
            State::Selection { start, end } => self.move_to(end.max(start)),
        }
    }

    pub fn select_range(&mut self, start: usize, end: usize) {
        if start == end {
            self.state = State::Index(start);
        } else {
            self.state = State::Selection {
                start: start.min(end),
                end: end.max(start),
            }
        }
    }

    pub fn select_to_start(&mut self, value: &str) {
        match self.state(value) {
            State::Index(index) => self.select_range(0, index),
            State::Selection { end, .. } => self.select_range(0, end),
        }
    }

    pub fn select_to_end(&mut self, value: &str) {
        match self.state(value) {
            State::Index(index) => self.select_range(index, value.len()),
            State::Selection { start, .. } => self.select_range(start, value.len()),
        }
    }

    pub fn select_all(&mut self, value: &str) {
        self.select_range(0, value.len())
    }

    pub fn select_left(&mut self, value: &str) {
        match self.state(value) {
            State::Index(index) if index > 0 => {
                self.select_range(index, index - 1);
            }
            State::Selection { start, end } if end > 0 => {
                self.select_range(start.saturating_sub(1), end);
            }
            _ => {}
        }
    }

    pub fn select_right(&mut self, value: &str) {
        match self.state(value) {
            State::Index(index) if index < value.len() => {
                self.select_range(index, index + 1);
            }
            State::Selection { start, end } if end < value.len() => {
                self.select_range(start, end + 1);
            }
            _ => {}
        }
    }

    pub fn start(&self, value: &str) -> usize {
        let start = match self.state {
            State::Index(idx) => idx,
            State::Selection { start, .. } => start,
        };

        start.min(value.len())
    }

    pub fn end(&self, value: &str) -> usize {
        let end = match self.state {
            State::Index(idx) => idx,
            State::Selection { end, .. } => end,
        };

        end.min(value.len())
    }

    pub fn left(&self, value: &str) -> usize {
        match self.state(value) {
            State::Index(idx) => idx,
            State::Selection { start, end } => start.min(end),
        }
    }

    pub fn right(&self, value: &str) -> usize {
        match self.state(value) {
            State::Index(idx) => idx,
            State::Selection { start, end } => start.max(end),
        }
    }
}

pub struct Editor<'a> {
    value: &'a mut String,
    cursor: &'a mut Cursor,
}

impl<'a> Editor<'a> {
    pub fn new(value: &'a mut String, cursor: &'a mut Cursor) -> Self {
        Self { value, cursor }
    }

    pub fn contents(&self) -> String {
        self.value.to_string()
    }

    pub fn insert(&mut self, character: char) {
        if let Some((left, right)) = self.cursor.selection(self.value) {
            self.cursor.move_left(self.value);
            self.value.replace_range(left..right, "");
        }

        self.value.insert(self.cursor.end(self.value), character);
        self.cursor.move_right(&self.value)
    }

    pub fn backspace(&mut self) {
        match self.cursor.selection(&self.value) {
            Some((start, end)) => {
                self.cursor.move_left(self.value);
                self.value.replace_range(start..end, "");
            }
            None => {
                let start = self.cursor.start(&self.value);

                if start > 0 {
                    self.cursor.move_left(&self.value);
                    self.value.remove(start - 1);
                }
            }
        }
    }

    pub fn delete(&mut self) {
        match self.cursor.selection(&self.value) {
            Some(_) => {
                self.backspace();
            }
            None => {
                let end = self.cursor.end(&self.value);

                if end < self.value.len() {
                    self.value.remove(end);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Selection {
    Block {
        rows: RangeInclusive<usize>,
        columns: RangeInclusive<usize>,
    },
    Scattered {
        cells: HashSet<(usize, usize)>,
        last: (usize, usize),
    },
}

impl Selection {
    pub fn new(row: usize, column: usize) -> Self {
        Self::Block {
            rows: row..=row,
            columns: column..=column,
        }
    }

    pub fn row(row: usize, column_len: usize) -> Self {
        Self::Block {
            rows: row..=row,
            columns: 0..=column_len,
        }
    }

    pub fn column(column: usize, limit: usize) -> Self {
        Self::Block {
            rows: 0..=limit,
            columns: column..=column,
        }
    }

    pub fn block(&mut self, row: usize, column: usize) {
        match self {
            Self::Block { rows, columns } => {
                if !rows.contains(&row) {
                    let start = *rows.start().min(&row);
                    let end = *rows.end().max(&row);

                    *rows = start..=end
                }

                if !columns.contains(&column) {
                    let start = *columns.start().min(&column);
                    let end = *columns.end().max(&column);

                    *columns = start..=end
                }
            }
            Self::Scattered { cells, last } => {
                let rows = row.min(last.0)..=row.max(last.0);
                let columns = (column.min(last.1)..=column.max(last.1)).collect::<Vec<usize>>();
                *last = (row, column);

                for row in rows {
                    let set = columns.iter().map(|column| (row, *column));

                    cells.extend(set);
                }
            }
        }
    }

    pub fn scattered(&mut self, row: usize, column: usize) {
        match self {
            Self::Block { rows, columns } => {
                let rows = rows.collect::<Vec<usize>>();
                let columns = columns.collect::<Vec<usize>>();
                let mut cells = HashSet::new();
                cells.insert((row, column));

                for row in rows {
                    let set = columns.iter().map(|column| (row, *column));

                    cells.extend(set)
                }

                *self = Self::Scattered {
                    cells,
                    last: (row, column),
                }
            }
            Self::Scattered { cells, last } => {
                cells.insert((row, column));
                *last = (row, column)
            }
        }
    }

    pub fn contains(&self, row: usize, column: usize) -> bool {
        match self {
            Self::Block { rows, columns } => rows.contains(&row) && columns.contains(&column),
            Self::Scattered { cells, .. } => cells.contains(&(row, column)),
        }
    }

    pub fn border(&self, row: usize, column: usize) -> u8 {
        match self {
            Self::Block { rows, columns } => {
                // bottom, right, top, left
                let mut out = 0;

                if !self.contains(row, column) {
                    return 0;
                }

                if *rows.start() == row {
                    // top
                    out = out | (1 << 1);
                }

                if *rows.end() == row {
                    // bottom
                    out = out | (1 << 3);
                }

                if *columns.start() == column {
                    // left
                    out = out | (1 << 0);
                }

                if *columns.end() == column {
                    // right
                    out = out | (1 << 2);
                }

                out
            }
            Self::Scattered { cells, .. } => {
                if cells.contains(&(row, column)) {
                    return 15;
                }

                0
            }
        }
    }

    pub fn header(&self, column: usize) -> bool {
        match self {
            Self::Block { columns, .. } => columns.contains(&column),
            Self::Scattered { cells, .. } => cells.iter().any(|(_, col)| *col == column),
        }
    }

    pub fn move_to(&mut self, row: usize, column: usize) {
        *self = Self::Block {
            rows: row..=row,
            columns: column..=column,
        };
    }

    pub fn move_right(&mut self, column_limit: usize) {
        match self {
            Self::Block { columns, rows } => {
                let row = *rows.start();
                let column = (*columns.start() + 1).min(column_limit);

                self.move_to(row, column);
            }
            Self::Scattered { last, .. } => {
                let row = last.0;
                let column = (last.1 + 1).min(column_limit);
                self.move_to(row, column);
            }
        }
    }

    pub fn move_left(&mut self) {
        match self {
            Self::Block { columns, rows } => {
                let row = *rows.start();
                let column = columns.start().saturating_sub(1);

                self.move_to(row, column);
            }
            Self::Scattered { last, .. } => {
                let row = last.0;
                let column = last.1.saturating_sub(1);
                self.move_to(row, column);
            }
        }
    }

    pub fn move_down(&mut self, row_limit: usize) {
        match self {
            Self::Block { rows, columns } => {
                let column = *columns.start();
                let row = (*rows.start() + 1).min(row_limit);

                self.move_to(row, column);
            }
            Self::Scattered { last, .. } => {
                let column = last.1;
                let row = (last.0 + 1).min(row_limit);
                self.move_to(row, column)
            }
        }
    }

    pub fn move_up(&mut self) {
        match self {
            Self::Block { rows, columns } => {
                let column = *columns.start();
                let row = rows.start().saturating_sub(1);

                self.move_to(row, column);
            }
            Self::Scattered { last, .. } => {
                let column = last.1;
                let row = last.0.saturating_sub(1);
                self.move_to(row, column)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Drag {
    Vertical,
    Horizontal,
    Diagonal,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Resizing {
    kind: Drag,
    cursor: Point,
    pub(crate) row: usize,
    pub(crate) column: usize,
}

impl Resizing {
    pub(crate) fn new(
        parent: Rectangle,
        child: Rectangle,
        cursor: mouse::Cursor,
        row: usize,
        column: usize,
    ) -> Option<Self> {
        let horizontal = {
            let height = parent.height;
            let width = parent.width - child.width;
            let position = parent.position() + Vector::new(child.width, 0.0);

            let bounds = Rectangle::new(position, Size::new(width, height));
            cursor.is_over(bounds)
        };

        let vertical = {
            let height = parent.height - child.height;
            let width = parent.width;
            let position = parent.position() + Vector::new(0.0, child.height);

            let bounds = Rectangle::new(position, Size::new(width, height));
            cursor.is_over(bounds)
        };

        let kind = if horizontal && vertical {
            Drag::Diagonal
        } else if horizontal {
            Drag::Horizontal
        } else if vertical {
            Drag::Vertical
        } else {
            return None;
        };

        let cursor = cursor.position_over(parent)?;

        Some(Self {
            kind,
            row,
            column,
            cursor,
        })
    }

    /// Returns the new minimum dimensions after a drag
    pub(crate) fn drag(&mut self, position: Point, width: f32, height: f32) -> (Size, Vector) {
        let diff = position - self.cursor;
        self.cursor = position;

        match self.kind {
            Drag::Vertical => {
                let size = Size::new(width, height + diff.y);
                let diff = Vector::new(0.0, diff.y);

                (size, diff)
            }
            Drag::Horizontal => {
                let size = Size::new(width + diff.x, height);
                let diff = Vector::new(-diff.x, 0.0);

                (size, diff)
            }
            Drag::Diagonal => (
                Size::new(width + diff.x, height + diff.y),
                Vector::new(-diff.x, diff.y),
            ),
        }
    }

    pub(crate) fn interaction(self) -> mouse::Interaction {
        match self.kind {
            Drag::Vertical => mouse::Interaction::ResizingVertically,
            Drag::Horizontal => mouse::Interaction::ResizingHorizontally,
            Drag::Diagonal => mouse::Interaction::ResizingDiagonallyDown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Option<Background>,
    pub goto_page_text: Color,
    pub border: Border,
    pub status_text_color: Color,
    pub header_text_color: Color,
    pub header_type_color: Color,
    pub goto_text_color: Color,
    pub hovered_goto_text_color: Color,
    pub goto_input_text_color: Color,
    pub pagination_text_color: Color,
    pub hovered_pagination_text_color: Color,
    pub page_text_color: Color,
    pub hovered_page_text_color: Color,
    pub selected_page_text_color: Color,
    pub cursor_color: Color,
    pub cursor_selection: Color,
    pub alternating_backgrounds: (Background, Background),
    pub alternating_text_color: (Color, Color),
    pub selected_header_background: Background,
    pub header_background: Background,
    pub selected_cell_border: Background,
    pub selected_cell_background: Background,
    pub cell_border: Background,
    pub status_background: Background,
    pub goto_border: Border,
    pub goto_background: Background,
    pub hovered_goto_background: Background,
    pub goto_input_background: Background,
    pub pagination_border: Border,
    pub pagination_background: Background,
    pub hovered_pagination_background: Background,
    pub page_border: Border,
    pub page_background: Background,
    pub hovered_page_background: Background,
    pub selected_page_background: Background,
}

pub trait Catalog {
    type Class<'a>;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let background = palette.background.weak;
    let status_background = palette.secondary.weak;
    let header_background = palette.secondary.base;
    let goto_background = palette.secondary.weak;
    let goto_hovered = palette.secondary.strong;
    let goto_input_background = palette.background.strong;
    let pagination_background = goto_background;
    let pagination_hovered = goto_hovered;
    let page_background = goto_background;
    let hovered_page = goto_hovered;
    let selected_page = palette.primary.weak;

    let (alt1, alt2) = (palette.secondary.weak, palette.secondary.strong);

    let cursor = palette.primary.strong;
    let rounded = Border::default().rounded(3.0);

    Style {
        background: Some(Background::Color(background.color)),
        border: Border::default(),

        status_text_color: status_background.text,
        status_background: Background::Color(status_background.color.scale_alpha(0.5)),

        header_background: Background::Color(header_background.color),
        header_text_color: header_background.text,
        header_type_color: header_background.text,
        selected_header_background: Background::Color(palette.primary.strong.color),

        goto_background: Background::Color(goto_background.color),
        goto_page_text: background.text,
        goto_text_color: goto_background.text,
        hovered_goto_background: Background::Color(goto_hovered.color),
        hovered_goto_text_color: goto_hovered.text,
        goto_input_background: Background::Color(goto_input_background.color),
        goto_input_text_color: goto_input_background.text,
        goto_border: rounded,

        pagination_background: Background::Color(pagination_background.color),
        pagination_text_color: pagination_background.text,
        hovered_pagination_background: Background::Color(pagination_hovered.color),
        hovered_pagination_text_color: pagination_hovered.text,
        pagination_border: rounded,

        page_background: Background::Color(page_background.color),
        page_text_color: page_background.text,
        hovered_page_background: Background::Color(hovered_page.color),
        hovered_page_text_color: hovered_page.text,
        selected_page_background: Background::Color(selected_page.color),
        selected_page_text_color: selected_page.text,
        page_border: rounded,

        cursor_color: cursor.color,
        cursor_selection: cursor.color.scale_alpha(0.5),

        alternating_text_color: (alt1.text, alt2.text),
        alternating_backgrounds: (Background::Color(alt1.color), Background::Color(alt2.color)),
        cell_border: Background::Color(palette.primary.weak.color),
        selected_cell_border: Background::Color(palette.primary.strong.color),
        selected_cell_background: Background::Color(palette.primary.weak.color.scale_alpha(0.40)),
    }
}
