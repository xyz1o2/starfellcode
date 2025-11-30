use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use crate::core::message::Role as AppRole;

/// 像素头像数据
#[derive(Clone, Debug)]
pub struct PixelData {
    pub color: Color,
    pub map: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl PixelData {
    pub fn new(color: Color, map: &[u8], width: usize) -> Self {
        let height = map.len() / width;
        Self {
            color,
            map: map.to_vec(),
            width,
            height,
        }
    }
}

// 4x4 像素头像（紧凑）
pub const AVATAR_USER: [u8; 16] = [
    1,1,1,1,
    1,2,2,1,
    1,1,1,1,
    0,1,1,0,
];

pub const AVATAR_AI: [u8; 16] = [
    1,1,1,1,
    1,1,1,1,
    1,1,0,0,
    0,1,1,0,
];

pub const AVATAR_SYS: [u8; 16] = [
    1,1,1,1,
    1,2,2,1,
    1,0,0,1,
    0,1,1,0,
];

// 8x8 像素图案（与 HTML 示例一致）
pub const AVATAR_SYS_MAP: [u8; 64] = [
    0,0,1,1,1,1,0,0,
    0,1,1,1,1,1,1,0,
    1,1,2,1,1,2,1,1,
    1,1,1,1,1,1,1,1,
    1,0,1,1,1,1,0,1,
    1,0,0,0,0,0,0,1,
    0,1,1,0,0,1,1,0,
    0,0,1,1,1,1,0,0,
];

pub const AVATAR_USER_MAP: [u8; 64] = [
    0,0,1,1,1,1,0,0,
    0,1,1,1,1,1,1,0,
    1,1,2,1,1,2,1,1,
    1,1,1,1,1,1,1,1,
    1,1,1,0,0,1,1,1,
    0,1,1,1,1,1,1,0,
    0,0,1,0,0,1,0,0,
    0,0,1,1,1,1,0,0,
];

pub const AVATAR_AI_MAP: [u8; 64] = [
    0,0,1,1,1,1,0,0,
    0,1,1,1,1,1,1,0,
    1,1,1,1,1,1,1,1,
    1,1,1,1,1,0,0,0,
    1,1,1,1,0,0,0,0,
    1,1,1,1,1,0,0,0,
    0,1,1,1,1,1,1,0,
    0,0,1,1,1,1,0,0,
];

pub fn get_avatar(role: &AppRole, user_color: Color, ai_color: Color) -> PixelData {
    match role {
        AppRole::User => PixelData::new(
            user_color,
            &AVATAR_USER,
            4, // width (4x4)
        ),
        AppRole::Assistant => PixelData::new(
            ai_color,
            &AVATAR_AI,
            4, // width (4x4)
        ),
        AppRole::System => PixelData::new(
            ai_color,
            &AVATAR_SYS,
            4, // width (4x4)
        ),
    }
}

/// 渲染正方形头像（4x4 像素，每个像素 1 个空格 + 背景色）
pub fn render_avatar_square(avatar: &PixelData) -> Vec<Line<'static>> {
    let mut lines = Vec::with_capacity(avatar.height);
    let black = Color::Rgb(0, 0, 0);

    for row in 0..avatar.height {
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(avatar.width);
        for col in 0..avatar.width {
            let v = avatar.map[row * avatar.width + col];
            let style = match v {
                0 => Style::default().bg(black),
                1 => Style::default().bg(avatar.color),
                2 => Style::default().bg(Color::White),
                _ => Style::default().bg(black),
            };
            // 每个像素 = 1 个空格 + 背景色
            spans.push(Span::styled(" ", style));
        }
        lines.push(Line::from(spans));
    }

    lines
}

/// 渲染纯 8x8 像素网格（无任何外边框/包裹）。
/// - 每个像素 = 一个空格，使用背景色填充
/// - 0 = 透明(用黑色背景显示以保持形状)，1 = 主色，2 = 白色（眼睛）
pub fn render_avatar_raw(avatar: &PixelData) -> Vec<Line<'static>> {
    let mut lines = Vec::with_capacity(avatar.height);
    let black = Color::Rgb(0, 0, 0);

    for row in 0..avatar.height {
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(avatar.width);
        for col in 0..avatar.width {
            let v = avatar.map[row * avatar.width + col];
            let style = match v {
                0 => Style::default().bg(black),
                1 => Style::default().bg(avatar.color),
                2 => Style::default().bg(Color::White),
                _ => Style::default().bg(black),
            };
            spans.push(Span::styled(" ", style));
        }
        lines.push(Line::from(spans));
    }

    lines
}
