use iced::{
    application,
    widget::{
        button, column, horizontal_space, row, text_editor,
        text_editor::{Action, Content},
    },
    Element, Length, Theme,
};

use highlighter::*;

fn main() -> iced::Result {
    application("Playground", App::update, App::view)
        .theme(App::theme)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Action(Action),
    Light,
    Dark,
}

struct App {
    content: Content,
    theme: Theme,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    fn new() -> Self {
        let content = std::fs::read_to_string("./examples/highlighter/dummy.csv").unwrap();

        Self {
            content: Content::with_text(&content),
            theme: Theme::SolarizedDark,
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Action(action) => self.content.perform(action),
            Message::Dark => self.theme = Theme::SolarizedDark,
            Message::Light => self.theme = Theme::SolarizedLight,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let btns = row!(
            horizontal_space(),
            button("Light").on_press(Message::Light),
            button("Dark").on_press(Message::Dark),
            horizontal_space(),
        )
        .spacing(20.0)
        .width(Length::Fill);

        let editor = text_editor(&self.content)
            .on_action(Message::Action)
            .highlight_with::<CSVHighlighter>(self.theme.clone(), |hl, _theme| hl.into_format())
            .padding([4, 8]);

        column!(btns, editor)
            .spacing(25.0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
