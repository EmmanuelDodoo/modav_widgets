use iced::{
    alignment::Horizontal,
    application, font,
    widget::{self, button, column, container, row, text},
    Element, Font, Length, Size, Task, Theme,
};

use std::iter::once;

use lilt::Easing;
use tree::{base::*, Tree};

fn main() -> iced::Result {
    application("Playground", App::update, App::view)
        .theme(App::theme)
        .window_size(Size::new(1200.0, 900.0))
        .run_with(|| {
            let task = font::load(include_bytes!("./fontello.ttf")).map(Message::FontLoading);
            (App::new(), task)
        })
}

#[derive(Debug, Clone)]
enum Message {
    FontLoading(Result<(), font::Error>),
    Button,
    None,
    Light,
    Dark,
    Input(String),
}

struct App {
    theme: Theme,
    input: String,
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
            input: String::from("Maybe a text input??"),
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
            Message::Button => {
                println!("Clicked a button!");
            }
            Message::FontLoading(_) => {}
            Message::Input(string) => {
                self.input = string;
            }
            Message::None => {}
        };

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let btns = row!(
            button("Light").on_press(Message::Light),
            button("Dark").on_press(Message::Dark),
        )
        .spacing(75.0);

        let icon = Icon {
            font: Font::with_name("fontello"),
            code_point: '\u{F0F6}',
            size: None,
            spacing: 5.0,
        };

        let base = {
            let with_icon = Tree::new(Base::new("Base widget with icon").icon(icon));

            Tree::with_children(Base::new("Base widget"), std::iter::once(with_icon))
        };

        let text = {
            let subs = ["more", "text"].into_iter().map(Tree::new);

            Tree::with_children(text("With text widget").center(), subs).width(150)
        };

        let buttons = {
            let one = Tree::new(button("Click me!").on_press(Message::Button));

            Tree::with_children(
                button("Some buttons as well").on_press(Message::None),
                once(one),
            )
        };

        let input = {
            let tooltip = widget::tooltip(
                "With a tooltip",
                "awesome iced",
                widget::tooltip::Position::Right,
            );
            Tree::with_children(
                widget::text_input("", &self.input).on_input(Message::Input),
                once(tooltip).map(Tree::new),
            )
            .width(200.0)
        };

        let animations = {
            let subs = (1..3).into_iter().map(|i| {
                let duration = (300 * i) as f32;
                let sub = Tree::new(Base::new("Empty section").icon(icon));
                let text = Base::new(format!("{}ms Duration", duration));

                Tree::with_children(text, once(sub)).animation_duration(duration)
            });

            Tree::with_children("Varying animation durations", subs)
        };

        let easings = {
            let easings = [Easing::EaseInOutQuad, Easing::EaseInOutExpo]
                .into_iter()
                .map(|easing| {
                    let base = Base::new(format!("{easing:?}"));
                    Tree::with_children(base, once(Tree::new("Empty"))).animation_easing(easing)
                });

            Tree::with_children("Varying easing functions", easings)
        };

        let subs = [base, text, buttons, input, animations, easings].into_iter();
        let root = Base::new("Tree widget").align_x(Horizontal::Center);
        let tree = Tree::with_children(root, subs)
            .width(300.0)
            .animation_duration(500.0);

        let content = column!(btns, tree, widget::text("All this and much more..."))
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
