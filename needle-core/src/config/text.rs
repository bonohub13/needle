use super::Position;
use serde::Deserialize;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Text {
    pub scale: f32,
    pub color: [u8; 4],
    pub position: Position,
}

impl Text {
    pub fn position(
        &self,
        screen_size: &winit::dpi::PhysicalSize<u32>,
        text_size: &[f32; 2],
        margin: f32,
    ) -> (f32, f32) {
        match self.position {
            Position::Center => self.center(screen_size, text_size),
            Position::Top => self.top(screen_size, text_size, margin),
            Position::Bottom => self.bottom(screen_size, text_size, margin),
            Position::Left => self.left(screen_size, text_size, margin),
            Position::Right => self.right(screen_size, text_size, margin),
            Position::TopLeft => {
                let top = self.top(screen_size, text_size, margin);
                let left = self.left(screen_size, text_size, margin);

                (left.0, top.1)
            }
            Position::TopRight => {
                let top = self.top(screen_size, text_size, margin);
                let right = self.right(screen_size, text_size, margin);

                (right.0, top.1)
            }
            Position::BottomLeft => {
                let bottom = self.bottom(screen_size, text_size, margin);
                let left = self.left(screen_size, text_size, margin);

                (left.0, bottom.1)
            }
            Position::BottomRight => {
                let bottom = self.bottom(screen_size, text_size, margin);
                let right = self.right(screen_size, text_size, margin);

                (right.0, bottom.1)
            }
        }
    }

    fn center(
        &self,
        screen_size: &winit::dpi::PhysicalSize<u32>,
        text_size: &[f32; 2],
    ) -> (f32, f32) {
        (
            (screen_size.width as f32 - text_size[0]) / 2.0,
            (screen_size.height as f32 - text_size[1]) / 2.0,
        )
    }

    fn top(
        &self,
        screen_size: &winit::dpi::PhysicalSize<u32>,
        text_size: &[f32; 2],
        margin: f32,
    ) -> (f32, f32) {
        (
            (screen_size.width as f32 - text_size[0]) / 2.0,
            margin * 2.0,
        )
    }

    fn bottom(
        &self,
        screen_size: &winit::dpi::PhysicalSize<u32>,
        text_size: &[f32; 2],
        margin: f32,
    ) -> (f32, f32) {
        (
            (screen_size.width as f32 - text_size[0]) / 2.0,
            screen_size.height as f32 - text_size[1] - (margin * 2.0),
        )
    }

    fn left(
        &self,
        screen_size: &winit::dpi::PhysicalSize<u32>,
        text_size: &[f32; 2],
        margin: f32,
    ) -> (f32, f32) {
        (margin, (screen_size.height as f32 - text_size[1]) / 2.0)
    }

    fn right(
        &self,
        screen_size: &winit::dpi::PhysicalSize<u32>,
        text_size: &[f32; 2],
        margin: f32,
    ) -> (f32, f32) {
        (
            screen_size.width as f32 - text_size[0] - margin,
            (screen_size.height as f32 - text_size[1]) / 2.0,
        )
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "# Text scale")?;
        writeln!(f, "scale = {}", self.scale)?;
        writeln!(f, "# Text color : [r, g, b, alpha]")?;
        writeln!(f, "#  Range : (0 - 255)")?;
        writeln!(
            f,
            "color = [{}, {}, {}, {}]",
            self.color[0], self.color[1], self.color[2], self.color[3]
        )?;
        writeln!(f, "# Position")?;
        writeln!(f, "#  Center (default)")?;
        writeln!(f, "#  Top")?;
        writeln!(f, "#  Bottom")?;
        writeln!(f, "#  Right")?;
        writeln!(f, "#  Left")?;
        writeln!(f, "#  TopRight")?;
        writeln!(f, "#  TopLeft")?;
        writeln!(f, "#  BottomRight")?;
        writeln!(f, "#  BottomLeft")?;
        write!(f, "position = {}", self.position)
    }
}
