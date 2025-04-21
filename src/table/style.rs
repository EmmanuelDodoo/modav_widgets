#[allow(unused_imports)]
use super::Table;
use iced::{Background, Border, Color, Theme};

#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The [`Background`] of the [`Table`].
    pub background: Option<Background>,
    /// The [`Border`] of the [`Table`].
    pub border: Border,
    /// The text [`Color`] of the goto "Page:".
    pub goto_page_text: Color,
    /// The text [`Color`] of the status area.
    pub status_text: Color,
    /// The text [`Color`] of the [`Table`] headers.
    pub header_text: Color,
    /// The text [`Color`] of the [`Table`] header types.
    pub header_type: Color,
    /// The text [`Color`] of the go-to button.
    pub goto_text: Color,
    /// The text [`Color`] of the go-to button when hovered.
    pub hovered_goto_text: Color,
    /// The text [`Color`] of the go-to input area.
    pub goto_input_text: Color,
    /// The text [`Color`] of the pagination buttons.
    pub pagination_text: Color,
    /// The text [`Color`] of the pagination buttons when hovered.
    pub hovered_pagination_text: Color,
    /// The text [`Color`] of the pages area.
    pub page_text: Color,
    /// The text [`Color`] of the pages area when hovered.
    pub hovered_page_text: Color,
    /// The text [`Color`] of the pages area when selected.
    pub selected_page_text: Color,
    /// The [`Color`] of the cursor.
    pub cursor_color: Color,
    /// The [`Color`] of the cursor when selecting text.
    pub cursor_selection: Color,
    /// The two backgrounds used by alternate rows in the [`Table`].
    pub alternating_backgrounds: (Background, Background),
    /// The two text colors used by alternate rows in the [`Table`].
    pub alternating_text_color: (Color, Color),
    /// The border [`Background`] of a header when selected.
    pub selected_header_border: Background,
    /// The border [`Background`] of a header.
    pub header_background: Background,
    /// The border [`Background`] of a cell when selected.
    pub selected_cell_border: Background,
    /// The [`Background`] of a cell when selected.
    pub selected_cell_background: Background,
    /// The border [`Background`] of a cell.
    pub cell_border: Background,
    /// The [`Background`] of the status area.
    pub status_background: Background,
    /// The [`Border`] of the go-to button.
    pub goto_border: Border,
    /// The [`Background`] of the go-to button.
    pub goto_background: Background,
    /// The [`Background`] of the go-to button when hovered.
    pub hovered_goto_background: Background,
    /// The [`Background`] of the go-to input area.
    pub goto_input_background: Background,
    /// The [`Border`] of the pagination buttons.
    pub pagination_border: Border,
    /// The [`Background`] of the pagination buttons.
    pub pagination_background: Background,
    /// The [`Background`] of the pagination buttons when hovered.
    pub hovered_pagination_background: Background,
    /// The [`Border`] of the pages.
    pub page_border: Border,
    /// The [`Background`] of the pages.
    pub page_background: Background,
    /// The [`Background`] of the pages when hovered.
    pub hovered_page_background: Background,
    /// The [`Background`] of the current page.
    pub selected_page_background: Background,
}

/// The theme catalog of a [`Table`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Table`].
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

/// The default styling for aa [`iced::Theme`].
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

        status_text: status_background.text,
        status_background: Background::Color(status_background.color.scale_alpha(0.5)),

        header_background: Background::Color(header_background.color),
        header_text: header_background.text,
        header_type: header_background.text,
        selected_header_border: Background::Color(palette.primary.strong.color),

        goto_background: Background::Color(goto_background.color),
        goto_page_text: background.text,
        goto_text: goto_background.text,
        hovered_goto_background: Background::Color(goto_hovered.color),
        hovered_goto_text: goto_hovered.text,
        goto_input_background: Background::Color(goto_input_background.color),
        goto_input_text: goto_input_background.text,
        goto_border: rounded,

        pagination_background: Background::Color(pagination_background.color),
        pagination_text: pagination_background.text,
        hovered_pagination_background: Background::Color(pagination_hovered.color),
        hovered_pagination_text: pagination_hovered.text,
        pagination_border: rounded,

        page_background: Background::Color(page_background.color),
        page_text: page_background.text,
        hovered_page_background: Background::Color(hovered_page.color),
        hovered_page_text: hovered_page.text,
        selected_page_background: Background::Color(selected_page.color),
        selected_page_text: selected_page.text,
        page_border: rounded,

        cursor_color: cursor.color,
        cursor_selection: cursor.color.scale_alpha(0.5),

        alternating_text_color: (alt1.text, alt2.text),
        alternating_backgrounds: (Background::Color(alt1.color), Background::Color(alt2.color)),
        cell_border: Background::Color(palette.primary.weak.color),
        selected_cell_border: Background::Color(palette.primary.strong.color),
        selected_cell_background: Background::Color(palette.primary.weak.color.scale_alpha(0.75)),
    }
}
