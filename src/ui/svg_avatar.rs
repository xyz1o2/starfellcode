use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::core::message::Role as AppRole;

// 角色配置常量
const USER_COLOR: Color = Color::Rgb(255, 105, 180); // 粉色
const ASSISTANT_COLOR: Color = Color::Rgb(255, 215, 0); // 黄色
const SYSTEM_COLOR: Color = Color::Rgb(0, 206, 209); // 青色
const ACCENT_COLOR: Color = Color::White;

/// 使用 Paragraph Widget 渲染头像，返回一个可渲染的组件
/// 每个像素用两个空格表示，以接近正方形
/// 使用 4x4 像素网格以减小头像尺寸
pub fn get_avatar_widget(role: &AppRole) -> Paragraph<'static> {
    let (color, pixels) = get_avatar_pixels(role);
    let mut lines = Vec::new();

    // 只渲染 4 行（缩小版本）
    for row_idx in 0..4 {
        let mut spans = Vec::new();
        let row = &pixels[row_idx * 2]; // 跳过一行以缩小
        
        for col_idx in 0..4 {
            let pixel_value = row[col_idx * 2]; // 跳过一列以缩小
            let style = match pixel_value {
                0 => Style::default(), // 透明/背景
                1 => Style::default().bg(color), // 主体色
                2 => Style::default().bg(ACCENT_COLOR), // 高亮色（眼睛等）
                _ => Style::default(),
            };
            // 每个像素用两个空格，使其接近正方形
            spans.push(Span::styled("  ", style));
        }
        lines.push(Line::from(spans));
    }

    Paragraph::new(lines)
}

/// 获取指定角色的 8x8 像素数据
fn get_avatar_pixels(role: &AppRole) -> (Color, Vec<Vec<u8>>) {
    match role {
        AppRole::User => (
            USER_COLOR,
            vec![
                vec![0, 0, 1, 1, 1, 1, 0, 0],
                vec![0, 1, 1, 1, 1, 1, 1, 0],
                vec![1, 1, 2, 1, 1, 2, 1, 1],
                vec![1, 1, 1, 1, 1, 1, 1, 1],
                vec![1, 1, 1, 1, 1, 1, 1, 1],
                vec![1, 1, 2, 1, 1, 2, 1, 1],
                vec![0, 1, 1, 1, 1, 1, 1, 0],
                vec![0, 0, 1, 1, 1, 1, 0, 0],
            ],
        ),
        AppRole::Assistant => (
            ASSISTANT_COLOR,
            vec![
                vec![0, 0, 1, 1, 1, 1, 0, 0],
                vec![0, 1, 1, 1, 1, 1, 1, 0],
                vec![1, 1, 2, 1, 1, 1, 1, 1],
                vec![1, 1, 1, 1, 1, 1, 1, 1],
                vec![1, 1, 1, 1, 1, 1, 1, 1],
                vec![1, 1, 1, 1, 1, 1, 1, 1],
                vec![0, 1, 1, 1, 1, 1, 1, 0],
                vec![0, 0, 1, 1, 1, 1, 0, 0],
            ],
        ),
        AppRole::System => (
            SYSTEM_COLOR,
            vec![
                vec![0, 0, 1, 1, 1, 1, 0, 0],
                vec![0, 1, 1, 1, 1, 1, 1, 0],
                vec![1, 1, 2, 1, 1, 2, 1, 1],
                vec![1, 1, 1, 1, 1, 1, 1, 1],
                vec![1, 1, 1, 0, 0, 1, 1, 1],
                vec![1, 1, 1, 0, 0, 1, 1, 1],
                vec![0, 1, 1, 1, 1, 1, 1, 0],
                vec![0, 0, 1, 1, 1, 1, 0, 0],
            ],
        ),
    }
}
