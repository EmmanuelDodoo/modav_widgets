use std::ops::Range;
use std::sync::LazyLock;
use std::time::Instant;

use syntect::parsing;

use iced::{advanced::text, color, font::Style, Color, Font, Theme};

const SYNTAX: &str = r#"
   name: CSV 
   file_extensions: [csv]
   scope: text.csv


   contexts:
     main:
       - match: '\s*(?=")'
         scope: quote.field.begin
         push: begin_quote 

       - match: '[^\n,]+(?:,|$)'
         scope: constant.field.csv
     
     begin_quote:
       - match: '(?:"[^"]*")*(?:,|$)'
         scope: quoted.field.csv
         pop: true
"#;

static SYNTAXES: LazyLock<parsing::SyntaxSet> =
    LazyLock::new(parsing::SyntaxSet::load_defaults_nonewlines);

static CSV_SYNTAX: LazyLock<parsing::SyntaxSet> = LazyLock::new(csv_syntax);

const LINES: usize = 50;

const QUOTED: &str = "quoted.field.csv";
const UNQUOTED: &str = "constant.field.csv";

fn csv_syntax() -> parsing::SyntaxSet {
    let mut builder = parsing::SyntaxSetBuilder::new();

    let csv = parsing::SyntaxDefinition::load_from_str(SYNTAX, false, None).unwrap();
    builder.add(csv);

    builder.build()
}

pub struct CSVHighlighter {
    syntax: &'static parsing::SyntaxReference,
    current_line: usize,
    parse_states: Vec<parsing::ParseState>,
    colors: ColorState,
}

impl text::Highlighter for CSVHighlighter {
    type Settings = Theme;
    type Highlight = CSVHighlight;
    type Iterator<'a> = Box<dyn Iterator<Item = (Range<usize>, Self::Highlight)> + 'a>;

    fn new(settings: &Self::Settings) -> Self {
        let colors = ColorState::new(settings);

        let syntax = CSV_SYNTAX
            .find_syntax_by_token("csv")
            .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());

        let parse_state = parsing::ParseState::new(syntax);

        Self {
            current_line: 0,
            syntax,
            colors,
            parse_states: vec![parse_state],
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        self.colors = ColorState::new(new_settings);
        self.change_line(0)
    }

    fn current_line(&self) -> usize {
        self.current_line
    }

    fn change_line(&mut self, line: usize) {
        let snapshot = line / LINES;

        if snapshot <= self.parse_states.len() {
            self.parse_states.truncate(snapshot);
            self.current_line = snapshot * LINES;
        } else {
            self.parse_states.truncate(1);
            self.current_line = 0;
        }

        let parser = self
            .parse_states
            .last()
            .cloned()
            .unwrap_or_else(|| parsing::ParseState::new(self.syntax));

        self.parse_states.push(parser);
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        if self.current_line / LINES >= self.parse_states.len() {
            let parser = self
                .parse_states
                .last()
                .expect("Parse States must not be empty");

            self.parse_states.push(parser.clone());
        }

        self.current_line += 1;

        let parser = self
            .parse_states
            .last_mut()
            .expect("Parse States must not be empty");

        let ops = parser.parse_line(line, &CSV_SYNTAX).unwrap_or_default();

        let quoted = parsing::Scope::new(QUOTED).unwrap();

        let unquoted = parsing::Scope::new(UNQUOTED).unwrap();

        let iter = ScopeRangeIterator {
            ops,
            line_length: line.len(),
            index: 0,
            last_str_index: 0,
        }
        .filter_map(|(range, scope)| match scope {
            parsing::ScopeStackOp::Push(scope) if scope == quoted => Some((range, true)),
            parsing::ScopeStackOp::Push(scope) if scope == unquoted => Some((range, false)),
            _ => None,
        });

        let mut output = vec![];

        for (range, quoted) in iter {
            let color = self.colors.next().unwrap_or_default();
            let style = if quoted { Style::Italic } else { Style::Normal };
            let highlight = CSVHighlight { color, style };

            output.push((range, highlight))
        }

        self.colors.reset();
        Box::new(output.into_iter())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CSVHighlight {
    color: Color,
    style: Style,
}

impl CSVHighlight {
    pub fn into_format(self) -> text::highlighter::Format<Font> {
        let Self { color, style } = self;

        let font = Font {
            style,
            ..Font::MONOSPACE
        };

        text::highlighter::Format {
            color: Some(color),
            font: Some(font),
        }
    }
}

#[derive(Debug, Clone)]
struct ColorState {
    engine: Engine,
    prev: Vec<Color>,
    current: usize,
}

impl ColorState {
    fn new(theme: &Theme) -> Self {
        let palette = theme.extended_palette();
        let engine = Engine::new(palette.background.base.color, palette.is_dark);

        Self {
            prev: vec![],
            engine,
            current: 0,
        }
    }

    fn reset(&mut self) {
        self.current = 0;
    }
}

impl Iterator for ColorState {
    type Item = Color;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.prev.len() {
            let color = self.prev.get(self.current).copied();
            self.current += 1;
            return color;
        }

        self.current += 1;

        let color = self.engine.next();

        self.prev.push(color.unwrap_or_default());

        color
    }
}

struct ScopeRangeIterator {
    ops: Vec<(usize, parsing::ScopeStackOp)>,
    line_length: usize,
    index: usize,
    last_str_index: usize,
}

impl Iterator for ScopeRangeIterator {
    type Item = (std::ops::Range<usize>, parsing::ScopeStackOp);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.ops.len() {
            return None;
        }

        let next_str_i = if self.index == self.ops.len() {
            self.line_length
        } else {
            self.ops[self.index].0
        };

        let range = self.last_str_index..next_str_i;
        self.last_str_index = next_str_i;

        let op = if self.index == 0 {
            parsing::ScopeStackOp::Noop
        } else {
            self.ops[self.index - 1].1.clone()
        };

        self.index += 1;
        Some((range, op))
    }
}

fn rand_f32() -> f32 {
    let nanos = Instant::now().elapsed().as_nanos() as u64;
    let x = (nanos ^ (nanos >> 33)).wrapping_mul(0x62A9D9ED799705F5);
    ((x >> 32) as f32) / (u32::MAX as f32)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Engine {
    hue: f32,
    sat: f32,
    lumi: f32,
    rng: f32,
}

impl Engine {
    const RATIO: f32 = 0.60;

    pub fn new(seed: Color, is_dark: bool) -> Self {
        let Color { r, g, b, .. } = seed;

        let (lumi, sat) = if is_dark { (0.65, 0.69) } else { (0.42, 0.77) };
        let hue = hue(r, g, b) / 360.0;
        let rng = rand_f32();

        Self {
            hue,
            rng,
            sat,
            lumi,
        }
    }

    pub fn generate(&mut self) -> Color {
        let hue = ((rand_f32() * 10.) + Self::RATIO + self.hue) % 1.0;

        self.hue = hue;

        let (r, g, b) = hsl_to_rgb(hue, self.sat, self.lumi);

        color!(r, g, b)
    }
}

impl Iterator for Engine {
    type Item = Color;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.generate())
    }
}

fn hue(r: f32, g: f32, b: f32) -> f32 {
    let min = f32::min(b, f32::min(r, g));
    let max = f32::max(b, f32::max(r, g));
    let diff = max - min;

    let h = if r == max {
        (g - b) / diff
    } else if g == max {
        2.0 + ((b - r) / diff)
    } else {
        4.0 + ((r - g) / diff)
    };

    let h = h * 60.0;

    if h < 0.0 {
        h + 360.0
    } else {
        h
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        let val = l * 255.0;
        return (val, val, val);
    }

    let h = (h * 360.0).min(360.0);
    let s = s.min(1.0);
    let l = l.min(1.0);

    let c = (1. - ((2.0 * l) - 1.).abs()) * s;
    let x = c * (1. - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = l - (c / 2.);

    let (r, g, b) = if 0. <= h && h < 60. {
        (c, x, 0.)
    } else if 60.0 <= h && h < 120. {
        (x, c, 0.)
    } else if 120.0 <= h && h < 180. {
        (0., c, x)
    } else if 180.0 <= h && h < 240. {
        (0., x, c)
    } else if 240.0 <= h && h < 300. {
        (x, 0., c)
    } else {
        (c, 0., x)
    };

    let r = (r + m) * 255.;
    let g = (g + m) * 255.;
    let b = (b + m) * 255.;

    (r, g, b)
}
