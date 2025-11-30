use ratatui::{
    style::Color,
    symbols::Marker,
    widgets::canvas::{Canvas, Rectangle},
};
use crate::core::message::Role as AppRole;

// 角色配置常量
const USER_COLOR: Color = Color::Rgb(255, 105, 180); // 粉色
const ASSISTANT_COLOR: Color = Color::Rgb(255, 215, 0); // 黄色
const SYSTEM_COLOR: Color = Color::Rgb(0, 206, 209); // 青色
const ACCENT_COLOR: Color = Color::White;

/// 使用 Canvas Widget 渲染头像，返回一个可渲染的组件
pub fn get_avatar_widget(role: &AppRole) -> impl ratatui::widgets::Widget + '_ {
    let (color, pixels) = get_avatar_pixels(role);

    Canvas::default()
        .marker(Marker::HalfBlock)
        .x_bounds([0.0, 8.0]) // 8x8 像素网格
        .y_bounds([0.0, 8.0])
        .paint(move |ctx| {
            for (y, row) in pixels.iter().enumerate() {
                for (x, &pixel_value) in row.iter().enumerate() {
                    if pixel_value == 0 {
                        continue; // 跳过背景色
                    }
                    let pixel_color = match pixel_value {
                        1 => color,
                        2 => ACCENT_COLOR,
                        _ => continue,
                    };
                    // 在 Canvas 上绘制一个 1x1 的矩形来代表一个像素
                    ctx.draw(&Rectangle {
                        x: x as f64,
                        y: y as f64,
                        width: 1.0,
                        height: 1.0,
                        color: pixel_color,
                    });
                }
            }
        })
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
