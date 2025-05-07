use iced::{
    alignment::Horizontal,
    application, font,
    widget::{button, column, container, focus_next, focus_previous, row, text},
    Element, Font, Length, Task, Theme,
};

use lilt::Easing;
use tree::*;

fn main() -> iced::Result {
    application("Playground", App::update, App::view)
        .theme(App::theme)
        .run_with(|| {
            let task = font::load(include_bytes!("./fontello.ttf")).map(Message::FontLoading);
            (App::new(), task)
        })
}

#[derive(Debug, Clone)]
enum Message {
    FontLoading(Result<(), font::Error>),
    Light,
    Dark,
    Next,
    Prev,
}

struct App {
    theme: Theme,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    fn new() -> Self {
        Self {
            theme: Theme::TokyoNightStorm,
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Light => self.theme = Theme::TokyoNightLight,
            Message::Dark => self.theme = Theme::TokyoNightStorm,
            Message::FontLoading(Err(error)) => {
                eprintln!("{error:?}")
            }
            Message::Next => return focus_next(),
            Message::Prev => return focus_previous(),
            Message::FontLoading(_) => {}
        };

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let text = text("Some Temporary Text");

        let btns = row!(
            button("Light").on_press(Message::Light),
            button("Dark").on_press(Message::Dark),
        )
        .spacing(75.0);

        let icon = Icon {
            font: Font::with_name("fontello"),
            code_point: '\u{F0F6}',
            size: None,
            spacing: 10.0,
        };

        let subs = (0..3).map(|i| {
            let subs = (0..i).map(|i| Tree::new(format!("Sub-sub {i}")).icon(icon));

            Tree::with_children(format!("Sub Tree {i}"), subs)
                .animation_easing(Easing::EaseInOutQuad)
        });

        let tree = Tree::with_children("Test tree".to_owned(), subs)
            .icon(icon)
            .width(200.0);

        let focs = row!(
            button("Prev").on_press(Message::Prev),
            button("Next").on_press(Message::Next)
        )
        .spacing(20.0);

        let content = column!(btns, tree, text, focs)
            .align_x(Horizontal::Center)
            .spacing(15.0)
            .width(Length::Fill)
            .height(Length::Fill);

        let content = container(content)
            .padding([4, 8])
            .width(Length::Fill)
            .height(Length::Fill);

        content.into()
    }
}
