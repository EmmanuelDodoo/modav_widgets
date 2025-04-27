#![allow(unused_imports, dead_code)]
use iced::{
    alignment::{Horizontal, Vertical},
    application, font,
    highlighter::{Highlighter, Settings},
    keyboard,
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
    Element, Font, Length, Task, Theme,
};

use modav_core::repr::col_sheet::{CellRef, ColumnSheet, DataType};

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
    Cell(String, usize, usize),
    Header(String, usize),
    Selection(Selection),
    AddLimit,
    SubLimit,
    None,
    Light,
    Dark,
}

struct App {
    sections: Vec<Section>,
    id_tracker: usize,
    theme: Theme,
    sht: ColumnSheet,
    status: Option<String>,
    limit: usize,
}

impl App {
    fn new() -> Self {
        //let path = "temp/air.csv".to_owned();
        //let path = "temp/empty.csv".to_owned();
        let path = "temp/mid1.csv".to_owned();
        let config = modav_core::repr::Config::new(path)
            .trim(true)
            .types(modav_core::repr::TypesStrategy::Infer)
            .labels(modav_core::repr::HeaderStrategy::ReadLabels);
        let sht = ColumnSheet::with_config(config).unwrap();

        Self {
            theme: Theme::TokyoNightStorm,
            sections: vec![],
            id_tracker: 0,
            sht,
            status: None,
            limit: 15,
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Test => println!("Testing"),
            Message::Light => self.theme = Theme::Light,
            Message::Dark => self.theme = Theme::TokyoNightStorm,
            Message::Cell(value, row, column) => {
                if let Err(error) = self.sht.set_cell(value, column, row) {
                    self.status.replace(error.to_string());
                } else {
                    self.status.take();
                }
            }
            Message::Header(value, column) => {
                if let Err(error) = self.sht.set_col_header(column, value) {
                    self.status.replace(error.to_string());
                } else {
                    self.status.take();
                }
            }
            Message::Selection(_selection) => {
                //dbg!(selection.list());
            }
            Message::AddLimit => self.limit += 1,
            Message::SubLimit => self.limit = (self.limit - 1).max(1),
            Message::None => {}
        };

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        //let text = text("Some Temporary Text");

        //let content = Table::new(&self.sht).height(Length::Fixed(350.0));
        let content = Table::new(&self.sht)
            .height(Length::Shrink)
            .page_limit(self.limit)
            .on_keypress(|key_press| {
                if key_press.key == keyboard::Key::Named(keyboard::key::Named::Home) {
                    Some(Message::Test)
                } else {
                    None
                }
            })
            .header_font(Font {
                style: font::Style::Italic,
                ..Default::default()
            })
            .numbering_font(Font {
                style: font::Style::Italic,
                ..Default::default()
            })
            .status_maybe(self.status.clone())
            .on_selection(Message::Selection)
            .on_header_input(Message::Header)
            .on_header_submit(Message::Header)
            .on_cell_submit(Message::Cell)
            .on_cell_input(Message::Cell);
        //let content = Table::new(&self.sht);
        //let content = scrollable(row!(content)).direction(Direction::Horizontal(Scrollbar::new()));
        //let content = text_input("Nothing", "\"Wisconsin Dells\"").on_input(|_| Message::None);

        let row = row!(
            button("Light").on_press(Message::Light),
            button("Dark").on_press(Message::Dark),
        )
        .spacing(75.0);
        let content = column!(row, content, "More")
            .spacing(50.0)
            .align_x(Horizontal::Center)
            .height(Length::Fill)
            .width(Length::Fill);

        let btns = column!(
            vertical_space(),
            button("Increase").on_press(Message::AddLimit),
            text(format!("Current page limit: {}", self.limit)),
            button("Reduce").on_press(Message::SubLimit),
            vertical_space(),
        )
        .spacing(20.0);

        let content = row!(btns, content).spacing(15.0);

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

pub fn cell_to_string(cell: CellRef<'_>) -> String {
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

impl RawTable for ColumnSheet {
    type ColumnKind = DataType;

    fn height(&self) -> usize {
        self.height()
    }

    fn width(&self) -> usize {
        self.width()
    }

    fn column_header(&self, index: usize) -> Option<String> {
        self.get_col(index)
            .and_then(|column| column.label().map(ToOwned::to_owned))
    }

    fn column_kind(&self, index: usize) -> Option<Self::ColumnKind> {
        self.get_col(index).map(|column| column.kind())
    }

    fn cell(&self, row: usize, column: usize) -> Option<String> {
        self.get_cell(column, row).map(cell_to_string)
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn column_filter(&self, kind: &Self::ColumnKind, character: char) -> bool {
        match kind {
            DataType::Text => true,
            DataType::I32 | DataType::ISize => {
                character.is_ascii_digit() || character == '-' || character == '_'
            }
            DataType::U32 | DataType::USize => character.is_ascii_digit() || character == '_',
            DataType::F32 | DataType::F64 => {
                character.is_ascii_digit() || character == '-' || character == '_'
            }
            DataType::Bool => {
                let chars = [
                    't', 'T', 'r', 'R', 'u', 'U', 'e', 'E', 'f', 'F', 'a', 'A', 'l', 'L', 's', 'S',
                ];

                chars.contains(&character)
            }
        }
    }

    fn kind_alignment(&self, kind: &Self::ColumnKind) -> Horizontal {
        match kind {
            DataType::Text | DataType::Bool => Horizontal::Left,
            _ => Horizontal::Right,
        }
    }
}
