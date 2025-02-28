#![allow(unused_imports, dead_code, unused_variables)]
use iced::{
    application, font,
    widget::{
        center, container,
        container::bordered_box,
        row, text, text_editor,
        text_editor::{Action, Content, TextEditor},
        tooltip::Position,
        Tooltip,
    },
    Element, Length, Theme,
};

mod custom;

use custom::menu::*;

mod highlighter;
use highlighter::*;

fn main() -> iced::Result {
    application("Playground", App::update, App::view)
        .theme(App::theme)
        .run_with(|| {
            let font = font::load(include_bytes!("../fontello.ttf")).map(|_| Message::None);

            (App::new(), font)
        })
}

#[derive(Debug, Clone)]
enum Message {
    Test,
    Action(Action),
    None,
}

struct App {
    content: Content,
    theme: Theme,
}

impl App {
    fn new() -> Self {
        Self {
            content: Content::default(),
            theme: Theme::TokyoNightLight,
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Test => println!("Testing"),
            Message::Action(action) => self.content.perform(action),
            Message::None => {}
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let _extension = "rs";

        text_editor(&self.content)
            .on_action(Message::Action)
            .height(Length::Fill)
            .highlight_with::<CSVHighlighter>(self.theme.clone(), |hl, _theme| hl.into_format())
            .padding([4, 8])
            .into()
    }
}
