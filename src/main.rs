#![allow(unused_imports, dead_code)]
use iced::{
    alignment::{Horizontal, Vertical},
    application, font,
    highlighter::{Highlighter, Settings},
    widget::{
        button, center, column, container,
        container::bordered_box,
        horizontal_space, row, scrollable,
        scrollable::{Anchor, Direction, Scrollbar},
        text,
        text_editor::Action,
        text_editor::{Content, TextEditor},
        text_input,
        tooltip::Position,
        vertical_space, Tooltip,
    },
    Element, Length, Task, Theme,
};

mod custom;

use custom::menu::*;

mod highlighter;
use highlighter::*;

mod table;
use table::*;

fn main() -> iced::Result {
    application("Playground", App::update, App::view)
        .theme(App::theme)
        .run_with(|| {
            let font = font::load(include_bytes!("../fontello.ttf")).map(|_| Message::None);

            (App::new(), font)
        })
}

struct Section {
    id: usize,
    content: Content,
}

#[derive(Debug, Clone)]
enum Message {
    Test,
    None,
}

struct App {
    sections: Vec<Section>,
    id_tracker: usize,
    theme: Theme,
}

impl App {
    fn new() -> Self {
        Self {
            theme: Theme::Nord,
            sections: vec![],
            id_tracker: 0,
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Test => println!("Testing"),
            Message::None => {}
        };

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let text = text("Some Temporary Text");

        //let path = "temp/air.csv".to_owned();
        //let path = "temp/empty.csv".to_owned();
        let path = "temp/mid1.csv".to_owned();
        let config = modav_core::repr::Config::new(path)
            .trim(true)
            .types(modav_core::repr::TypesStrategy::Infer)
            .labels(modav_core::repr::HeaderStrategy::ReadLabels);

        //let content = Table::new(config).height(Length::Fixed(350.0));
        let content = Table::new(config).height(Length::Shrink);
        //let content = Table::new(config);
        //let content = scrollable(row!(content)).direction(Direction::Horizontal(Scrollbar::new()));
        //let content = text_input("Nothing", "\"Wisconsin Dells\"").on_input(|_| Message::None);

        let content = column!(text, content, "More")
            .spacing(50.0)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
            .width(Length::Fill);

        let content = container(content)
            .padding([4, 8])
            .width(Length::Fill)
            .height(Length::Fill);

        content.into()
    }
}

pub async fn load_file() -> Result<String, std::io::ErrorKind> {
    let path = "./src/main.rs";
    let res = tokio::fs::read_to_string(path)
        .await
        .map_err(|err| err.kind());

    res
}
