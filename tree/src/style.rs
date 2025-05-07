use iced::{Background, Border, Color, Shadow, Theme};

#[allow(unused_imports)]
use crate::Tree;

#[derive(Debug, Clone, Copy, PartialEq)]
/// The possible status of a [`Tree`].
pub enum Status {
    /// The [`Tree`] is selected.
    Active,
    /// The [`Tree`] is being hovered on.
    Hovered,
    /// The default [`Tree`] status.
    Idle,
}

#[derive(Debug, Clone, Copy)]
/// The appearance of a [`Tree`].
pub struct Style {
    /// The [`Background`] of the root.
    pub background: Background,
    /// The [`Border`] of the root.
    pub border: Border,
    /// The [`Shadow`] of the root.
    pub shadow: Shadow,
    /// The [`Color`] of the root's text.
    pub text_color: Color,
}

/// The theme catalog of a [`Tree`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// The styling function for a [`Tree`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`Tree`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let border = Border::default().width(1.5).rounded(4.5);
    let shadow = Shadow::default();

    match status {
        Status::Idle => {
            let background = palette.background.weak;
            let border = border.color(background.color);

            Style {
                border,
                background: background.color.into(),
                text_color: background.text,
                shadow,
            }
        }
        Status::Hovered => {
            let background = palette.secondary.weak;
            let border = border.color(background.color);

            Style {
                border,
                background: background.color.into(),
                text_color: background.text,
                shadow,
            }
        }
        Status::Active => {
            let background = palette.primary.weak;
            let border = border.color(background.color);

            Style {
                border,
                background: background.color.into(),
                text_color: background.text,
                shadow,
            }
        }
    }
}
