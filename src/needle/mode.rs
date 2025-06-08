// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ImguiMode {
    Background,
    ClockTimer,
    Fps,
    Invalid,
}

macro_rules! imgui_mode_from {
    ( $type:ty ) => {
        impl From<ImguiMode> for $type {
            fn from(mode: ImguiMode) -> Self {
                match mode {
                    ImguiMode::Background => 0,
                    ImguiMode::ClockTimer => 1,
                    ImguiMode::Fps => 2,
                    ImguiMode::Invalid => Self::MAX,
                }
            }
        }

        impl From<$type> for ImguiMode {
            fn from(val: $type) -> Self {
                match val {
                    0 => ImguiMode::Background,
                    1 => ImguiMode::ClockTimer,
                    2 => ImguiMode::Fps,
                    _ => ImguiMode::Invalid,
                }
            }
        }
    };
}

imgui_mode_from! { i8 }
imgui_mode_from! { u8 }
imgui_mode_from! { i16 }
imgui_mode_from! { u16 }
imgui_mode_from! { i32 }
imgui_mode_from! { u32 }
