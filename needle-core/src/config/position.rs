use serde::Deserialize;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum Position {
    Center,
    Top,
    Bottom,
    Right,
    Left,
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let position = match self {
            Self::Center => "Center",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
            Self::Right => "Right",
            Self::Left => "Left",
            Self::TopRight => "TopRight",
            Self::TopLeft => "TopLeft",
            Self::BottomRight => "BottomRight",
            Self::BottomLeft => "BottomLeft",
        };

        write!(f, "\"{}\"", position)
    }
}
