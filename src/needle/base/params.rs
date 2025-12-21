// Copyright 2025 Kensuke Saito
// SPDX-License-Identifier: MIT

// Imgui Tags
const NEEDLE_IMGUI_SAVE_COUNT: usize = 2;
const NEEDLE_IMGUI_DESCRIPTION_COUNT: usize = 4;
//  - Background
const BACKGROUND_COLOR_COUNT: usize = 4;
//  - Window
const WINDOW_TYPE_COUNT: usize = 2;
//  - Clock Timer
const CLOCK_TIMER_FONT_COLOR_COUNT: usize = 3;
const CLOCK_TIMER_POSITION_COUNT: usize = 9;
//  - FPS
const FPS_FONT_COLOR_COUNT: usize = 3;
const FPS_POSITION_COUNT: usize = 4;
//  - Overlay
const OVERLAY_POSITION_COUNT: usize = 2;
const OVERLAY_SIZE_COUNT: usize = 2;
const OVERLAY_COLOR_COUNT: usize = 4;

// Background related parameters
impl<'background> super::NeedleBase<'background> {
    /* UI elements */
    pub(crate) const BACKGROUND_COLOR_TAG: &'background str = "Color:";
    pub(crate) const BACKGROUND_COLOR_RANGE: [f32; 2] = [0f32, 1f32];

    #[inline]
    pub(crate) const fn background_color() -> [&'background str; BACKGROUND_COLOR_COUNT] {
        [
            "red (background)",
            "green (background)",
            "blue (background)",
            "alpha (background)",
        ]
    }
}

impl<'window> super::NeedleBase<'window> {
    pub(crate) const WINDOW_TYPE_TAG: &'window str = "Window option:";

    #[inline]
    pub(crate) const fn window_type() -> [&'window str; WINDOW_TYPE_COUNT] {
        ["Fullscreen", "Windowed"]
    }
}

// Clock Timer related parameters
impl<'clock_timer> super::NeedleBase<'clock_timer> {
    /* UI elements */
    pub(crate) const CLOCK_TIMER_LIST_ROW_LENGTH: i32 = 5;
    pub(crate) const CLOCK_TIMER_FONT_TAG: &'clock_timer str = "Font";
    pub(crate) const CLOCK_TIMER_FONT_COLOR_TAG: &'clock_timer str = "Font Color";
    pub(crate) const CLOCK_TIMER_FONT_SCALE_TAG: &'clock_timer str = "Font Scale";
    pub(crate) const CLOCK_TIMER_FONT_SCALE_RANGE: [u8; 2] = [1, u8::MAX];
    pub(crate) const CLOCK_TIMER_POSITION_TAG: &'clock_timer str = "Clock Position";
    pub(crate) const CLOCK_TIMER_MODE_TAG: &'clock_timer str = "Mode:";
    pub(crate) const CLOCK_TIMER_FORMAT_MODE_TAG: &'clock_timer str = "Format Mode";
    pub(crate) const CLOCK_TIMER_CLOCK_MODE_TAG: &'clock_timer str = "Clock Mode";
    pub(crate) const CLOCK_TIMER_CLOCK_MODE_INFO: &'clock_timer str =
        "Press \"SPACE\" to start/stop timer";
    pub(crate) const CLOCK_TIMER_CLOCK_MODE_DURATION_TAG: &'clock_timer str = "Countdown Duration";

    #[inline]
    pub(crate) const fn clock_font_color() -> [&'clock_timer str; CLOCK_TIMER_FONT_COLOR_COUNT] {
        ["red (text)", "green (text)", "blue (text)"]
    }

    #[inline]
    pub(crate) const fn clock_position() -> [&'clock_timer str; CLOCK_TIMER_POSITION_COUNT] {
        [
            "Center",
            "Top",
            "Bottom",
            "Left",
            "Right",
            "Top Left",
            "Top Right",
            "Bottom Left",
            "Bottom Right",
        ]
    }
}

// FPS related parameters
impl<'fps> super::NeedleBase<'fps> {
    pub(crate) const FPS_LIST_ROW_LENGTH: i32 = 5;
    pub(crate) const FPS_VISUALIZATION_TAG: &'fps str = "Toggle FPS visualization";
    pub(crate) const FPS_VISUALIZATION_RANGE: [i32; 2] = [0, 1];
    pub(crate) const FPS_FONT_COLOR_TAG: &'fps str = "Font Color:";
    pub(crate) const FPS_FONT_COLOR_RANGE: [u8; 2] = [u8::MIN, u8::MAX];
    pub(crate) const FPS_POSITION_TAG: &'fps str = "FPS Position";

    #[inline]
    pub(crate) const fn fps_enable(enable: bool) -> &'fps str {
        if enable {
            "Enable"
        } else {
            "Disable"
        }
    }

    #[inline]
    pub(crate) const fn fps_font_color() -> [&'fps str; FPS_FONT_COLOR_COUNT] {
        ["red (fps)", "green (fps)", "blue (fps)"]
    }

    #[inline]
    pub(crate) const fn fps_position() -> [&'fps str; FPS_POSITION_COUNT] {
        ["Top Left", "Top Right", "Bottom Left", "Bottom Right"]
    }
}

// Overlay related parameters
#[allow(dead_code)]
impl<'overlay> super::NeedleBase<'overlay> {
    pub(crate) const OVERLAY_LIST_ROW_LENGTH: i32 = 10;
    pub(crate) const OVERLAY_ADD_TAG: &'overlay str = "Add Overlay:";
    pub(crate) const OVERLAY_REMOVE_TAG: &'overlay str = "Remove Overlay:";
    pub(crate) const OVERLAY_LIST_TAG: &'overlay str = "Overlays:";
    pub(crate) const OVERLAY_NAME_TAG: &'overlay str = "Name:";
    pub(crate) const OVERLAY_VERTEX_SHADER_TAG: &'overlay str = "Vertex shader path";
    pub(crate) const OVERLAY_VERTEX_SHADER_DEFAULT_PATH: &'overlay str = "overlay.vert.spv";
    pub(crate) const OVERLAY_FRAGMENT_SHADER_TAG: &'overlay str = "Fragment shader path";
    pub(crate) const OVERLAY_FRAGMENT_SHADER_DEFAULT_PATH: &'overlay str = "overlay.frag.spv";
    pub(crate) const OVERLAY_POSITION_TAG: &'overlay str = "Position";
    pub(crate) const OVERLAY_POSITION_RANGE: [f32; 2] = [-1f32, 1f32];
    pub(crate) const OVERLAY_SIZE_TAG: &'overlay str = "Size";
    pub(crate) const OVERLAY_SIZE_RANGE: [f32; 2] = [0f32, 1f32];
    pub(crate) const OVERLAY_COLOR_TAG: &'overlay str = "Color";
    pub(crate) const OVERLAY_COLOR_RANGE: [f32; 2] = [0f32, 1f32];

    #[inline]
    pub(crate) const fn overlay_position() -> [&'overlay str; OVERLAY_POSITION_COUNT] {
        ["x (overlay)", "y (overlay)"]
    }

    #[inline]
    pub(crate) const fn overlay_size() -> [&'overlay str; OVERLAY_SIZE_COUNT] {
        ["width (overlay)", "height (overlay)"]
    }

    #[inline]
    pub(crate) const fn overlay_color() -> [&'overlay str; OVERLAY_COLOR_COUNT] {
        [
            "red (overlay)",
            "green (overlay)",
            "blue (overlay)",
            "alpha (overlay)",
        ]
    }
}

// General parameters
impl<'needle> super::NeedleBase<'needle> {
    /* UI elements */
    pub(crate) const WINDOW_TITLE: &'needle str = "Needle Settings";
    pub(crate) const WINDOW_SIZE: [f32; 2] = [800f32, 600f32];
    pub(crate) const SETTINGS_TAG: &'needle str = "Settings";
    pub(crate) const SAVE_TAG: &'needle str = "Save";

    #[inline]
    pub(crate) const fn save() -> [&'needle str; NEEDLE_IMGUI_SAVE_COUNT] {
        ["Press \"INSERT\" to toggle menu.", "Save config:"]
    }

    #[inline]
    pub(crate) const fn description() -> [&'needle str; NEEDLE_IMGUI_DESCRIPTION_COUNT] {
        [
            "Repository:",
            "  - https://github.com/bonohub13/needle",
            "License:",
            "  - MIT",
        ]
    }
}
