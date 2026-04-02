use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Parse a string containing ANSI escape codes into a Vec of ratatui Lines.
pub fn parse_ansi_text(input: &str) -> Vec<Line<'static>> {
    input.lines().map(parse_ansi_line).collect()
}

fn parse_ansi_line(line: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut style = Style::default();
    let mut buf = String::new();
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                // Flush current buffer
                if !buf.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut buf), style));
                }
                chars.next(); // consume '['
                let mut params = String::new();
                while let Some(&pc) = chars.peek() {
                    if pc.is_ascii_alphabetic() {
                        let code = chars.next().unwrap();
                        if code == 'm' {
                            style = apply_sgr(&params, style);
                        }
                        break;
                    }
                    params.push(chars.next().unwrap());
                }
            }
        } else {
            buf.push(c);
        }
    }

    if !buf.is_empty() {
        spans.push(Span::styled(buf, style));
    }

    Line::from(spans)
}

fn apply_sgr(params: &str, mut style: Style) -> Style {
    if params.is_empty() {
        return Style::default();
    }

    let mut codes = params.split(';').filter_map(|s| s.parse::<u8>().ok()).peekable();

    while let Some(code) = codes.next() {
        match code {
            0 => style = Style::default(),
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            3 => style = style.add_modifier(Modifier::ITALIC),
            4 => style = style.add_modifier(Modifier::UNDERLINED),
            7 => style = style.add_modifier(Modifier::REVERSED),
            22 => style = style.remove_modifier(Modifier::BOLD | Modifier::DIM),
            23 => style = style.remove_modifier(Modifier::ITALIC),
            24 => style = style.remove_modifier(Modifier::UNDERLINED),
            27 => style = style.remove_modifier(Modifier::REVERSED),
            30..=37 => style = style.fg(ansi_color(code - 30)),
            38 => style = style.fg(parse_extended_color(&mut codes)),
            39 => style = style.fg(Color::Reset),
            40..=47 => style = style.bg(ansi_color(code - 40)),
            48 => style = style.bg(parse_extended_color(&mut codes)),
            49 => style = style.bg(Color::Reset),
            90..=97 => style = style.fg(ansi_bright_color(code - 90)),
            100..=107 => style = style.bg(ansi_bright_color(code - 100)),
            _ => {}
        }
    }
    style
}

fn parse_extended_color(codes: &mut std::iter::Peekable<impl Iterator<Item = u8>>) -> Color {
    match codes.next() {
        Some(5) => {
            // 256-color: \e[38;5;Nm
            codes.next().map(|n| Color::Indexed(n)).unwrap_or(Color::Reset)
        }
        Some(2) => {
            // RGB: \e[38;2;R;G;Bm
            let r = codes.next().unwrap_or(0);
            let g = codes.next().unwrap_or(0);
            let b = codes.next().unwrap_or(0);
            Color::Rgb(r, g, b)
        }
        _ => Color::Reset,
    }
}

fn ansi_color(n: u8) -> Color {
    match n {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        _ => Color::Reset,
    }
}

fn ansi_bright_color(n: u8) -> Color {
    match n {
        0 => Color::DarkGray,
        1 => Color::LightRed,
        2 => Color::LightGreen,
        3 => Color::LightYellow,
        4 => Color::LightBlue,
        5 => Color::LightMagenta,
        6 => Color::LightCyan,
        7 => Color::Gray,
        _ => Color::Reset,
    }
}
