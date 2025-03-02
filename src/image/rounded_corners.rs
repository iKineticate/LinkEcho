use image::{DynamicImage, RgbaImage};
use std::cmp::min;

pub fn add_rounded_corners(image: &DynamicImage, radius: u32) -> RgbaImage {
    let (width, height) = (image.width(), image.height());
    let radius = min(radius, min(width, height) / 2);
    let mut rounded_image = image.to_rgba8();

    for y in 0..height {
        for x in 0..width {
            let original_alpha = rounded_image.get_pixel(x, y).0[3];
            let alpha = calculate_alpha(x, y, width, height, radius, original_alpha);
            let pixel = rounded_image.get_pixel_mut(x, y);

            if alpha == 0 {
                pixel.0 = [0, 0, 0, 0];
            } else {
                pixel.0[3] = alpha;
            }
        }
    }

    rounded_image
}

/// 计算 Alpha 值（精准边缘模糊）
fn calculate_alpha(x: u32, y: u32, width: u32, height: u32, radius: u32, original_alpha: u8) -> u8 {
    let radius_f = radius as f32;
    let x_f = x as f32 + 0.5;
    let y_f = y as f32 + 0.5;
    let width_f = width as f32;
    let height_f = height as f32;

    // 判断是否在圆角处理范围内
    let is_in_corner = 
        (x < radius && y < radius) || 
        (x >= width - radius && y < radius) || 
        (x < radius && y >= height - radius) || 
        (x >= width - radius && y >= height - radius);

    if !is_in_corner {
        return original_alpha;
    }

    // 计算到对应圆心的距离
    let (cx, cy) = match (x < radius, y < radius) {
        (true, true) => (radius_f, radius_f),
        (false, true) => (width_f - radius_f, radius_f),
        (true, false)  => (radius_f, height_f - radius_f),
        (false, false)  => (width_f - radius_f, height_f - radius_f),
    };

    let dx = x_f - cx;
    let dy = y_f - cy;
    let distance = (dx * dx + dy * dy).sqrt();

    // 关键优化：使用非线性过渡 + 模糊半径
    let edge_softness = 1.2; // 调整模糊范围
    let alpha = if distance > radius_f + edge_softness {
        0.0
    } else if distance < radius_f - edge_softness {
        1.0
    } else {
        let t = (distance - (radius_f - edge_softness)) / (2.0 * edge_softness);
        smoothstep(0.0, 1.0, 1.0 - t) // 反向过渡
    };

    (alpha * 255.0) as u8
}

/// 平滑过渡函数（三阶插值消除锯齿）
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}