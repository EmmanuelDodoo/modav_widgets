#![allow(unused_imports, dead_code)]
use iced::{
    application, font,
    widget::{center, container, container::bordered_box, row, text, tooltip::Position, Tooltip},
    Element, Font, Length, Theme,
};

mod custom;

use custom::menu::*;

fn main() -> iced::Result {
    application("Playground", App::update, App::view)
        .theme(|_| Theme::TokyoNightLight)
        .run_with(|| {
            let font = font::load(include_bytes!("../fontello.ttf")).map(|_| Message::None);

            (App::new(), font)
        })
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Open,
    Close,
    Test,
    None,
}

struct App {
    open: bool,
}

impl App {
    fn new() -> Self {
        Self { open: false }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Test => println!("Testing"),
            Message::Open => {
                println!("Opening");
                self.open = true
            }
            Message::Close => {
                println!("Closing");
                self.open = false;
            }
            Message::None => {}
        }
    }

    fn main<'a>() -> Section<'a, Message> {
        let font = Font::with_name("fontello");

        let one = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example Menu");
            Item::new(icon, text).message(Message::Open)
        };

        let two: Item<'_, Message> = {
            let tip = "Some test tooltip";
            let icon = text("\u{F0F6}").font(font);
            let tip = Tooltip::new(icon, tip, Position::Right);

            let text = text("Cringe Two");
            Item::with_tooltip(tip, text).message(Message::Test)
        };

        let three: Item<'_, Message> = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example 3");
            Item::new(icon, text)
        };

        let four: Item<'_, Message> = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example 3");
            Item::new(icon, text)
        };

        let five: Item<'_, Message> = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example 3");
            Item::new(icon, text)
        };

        let six: Item<'_, Message> = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example 3");
            Item::new(icon, text)
        };

        let seven: Item<'_, Message> = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Example 3");
            Item::new(icon, text)
        };

        section![one, two, three, four, five, six, seven]
    }

    fn footer<'a>() -> Section<'a, Message> {
        let font = Font::with_name("fontello");

        let one = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Settings");
            Item::new(icon, text)
        };

        let two = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("About");
            Item::new(icon, text)
        };
        let three = {
            let icon = text("\u{F0F6}").font(font);
            let text = text("Help");
            Item::new(icon, text)
        };

        section![one, two, three]
    }

    fn view(&self) -> Element<'_, Message> {
        let header: Item<'_, Message> = {
            let base = Font {
                //weight: font::Weight::Semibold,
                //style: font::Style::Italic,
                ..Default::default()
            };

            let font = Font::with_name("fontello");

            let header = {
                let icon = text("\u{F0F6}").font(font).size(24);
                let text = text("Header").font(base).size(24);
                Item::new(icon, text)
            };

            header
        };

        let main = Self::main().spacing(20.0);
        let footer = Self::footer();

        let cols = Collapsible::new(header, main, footer);
        //let cols = Collapsible::no_header(main, footer);
        //let cols = Collapsible::no_footer(header, main);
        //let cols = Collapsible::only_main(main);

        let content = container(cols).style(bordered_box);

        let context = container(
            context!(text("one"), text( "two"), text("three"); Message::Close)
                .spacing(100.0)
                .height(Length::Fill),
        )
        .style(bordered_box);

        let content: Element<'_, Message> = if self.open {
            row!(content, context).into()
        } else {
            content.into()
        };

        center(content).center(Length::Fill).into()
    }
}
