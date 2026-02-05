use iced::theme::{Custom, Palette};
use iced::{color, Theme};
use std::sync::Arc;

pub fn paper() -> Theme {
    Theme::Custom(Arc::new(Custom::new("Paper".into(), Palette {
        background: color!(0xf2eede),
        text: color!(0x555555),
        primary: color!(0x1a1a1a),
        success: color!(0x1e6fcc),
        warning: color!(0x216609),
        danger: color!(0xcc3e28),
    })))
}

pub fn paper_dark() -> Theme {
    Theme::Custom(Arc::new(Custom::new("Paper Dark".into(), Palette {
        background: color!(0x1f1e1a),
        text: color!(0xd4c8b0),
        primary: color!(0xe8dcc0),
        success: color!(0x1e6fcc),
        warning: color!(0x216609),
        danger: color!(0xcc3e28),
    })))
}
