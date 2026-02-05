use iced::theme::{Custom, Palette};
use iced::widget::container;
use iced::{Border, Color, Theme, border, color, gradient};
use std::sync::Arc;

pub fn paper() -> Theme {
    Theme::Custom(Arc::new(Custom::new(
        "Paper".into(),
        Palette {
            background: color!(0xf2eede),
            text: color!(0x555555),
            primary: color!(0x1a1a1a),
            success: color!(0x1e6fcc),
            warning: color!(0x216609),
            danger: color!(0xcc3e28),
        },
    )))
}

pub fn paper_dark() -> Theme {
    Theme::Custom(Arc::new(Custom::new(
        "Paper Dark".into(),
        Palette {
            background: color!(0x1f1e1a),
            text: color!(0xd4c8b0),
            primary: color!(0xe8dcc0),
            success: color!(0x1e6fcc),
            warning: color!(0x216609),
            danger: color!(0xcc3e28),
        },
    )))
}

pub fn user(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.primary.weak.color.into()),
        text_color: Some(palette.primary.weak.text),
        border: border::rounded(12),
        ..container::Style::default()
    }
}

pub fn assistant(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.weak.color.into()),
        text_color: Some(palette.background.weak.text),
        border: border::rounded(12),
        ..container::Style::default()
    }
}

pub fn sidebar(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn fade(theme: &Theme) -> container::Style {
    let bg = theme.palette().background;
    let bg_transparent = Color { a: 0.0, ..bg };
    gradient::Linear::new(std::f32::consts::PI)
        .add_stop(0.4, bg)
        .add_stop(0.7, bg_transparent)
        .into()
}
