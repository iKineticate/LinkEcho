use anyhow::{anyhow, Ok, Result};
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use regex::Regex;

use super::rounded_corners::add_rounded_corners;
// use csscolorparser::Color;

#[derive(Debug, PartialEq, Clone)]
enum GradientDirection {
    Horizontal,
    Vertical,
    Angle(f32), // deg 等角度
    None,
}

pub fn get_background_image(color: String, scaling: u32, radius: u32) -> Result<RgbaImage> {
    let (width, height) = calculate_dimensions(scaling);

    let background_image = if color.starts_with("linear-gradient(") {
        let (direction, stops) = parse_gradient(&color)?;
        generate_gradient_image(width, height, direction, stops)
    } else {
        let solid_color = parse_color_str(&color)?;
        Ok(ImageBuffer::from_pixel(width, height, solid_color))
    }?;

    if radius != 0 {
        Ok(add_rounded_corners(&DynamicImage::from(background_image), radius))
    } else {
        Ok(background_image)
    }
}

fn calculate_dimensions(scaling: u32) -> (u32, u32) {
    let base = 256;
    let scaled = base * scaling / 100;
    (scaled.max(1), scaled.max(1))
}

/// 解析纯色
fn parse_color_str(color_str: &str) -> Result<Rgba<u8>> {
    let color = csscolorparser::parse(color_str)?;
    let [r, g, b, a] = color.to_rgba8();
    Ok(Rgba([r, g, b, a]))
}

/// 解析渐变参数
fn parse_gradient(gradient_str: &str) -> Result<(GradientDirection, Vec<(Rgba<u8>, f32)>)> {
    let re = Regex::new(r"linear-gradient\(([^)]+)\)").unwrap();
    let capstures = re.captures(gradient_str)
        .ok_or_else(|| anyhow!("Invalid gradient syntax"))?;

    let content = capstures[1].trim();
    let mut parts = content.splitn(2, ',');
    
    // 解析方向
    let direction_str = parts.clone().next()
        .ok_or_else(|| anyhow!("Missing gradient direction"))?
        .trim();
    let direction = parse_direction(direction_str)?;

    // 解析颜色停止点
    let stops_str = if direction == GradientDirection::None {
        parts
            .into_iter()
            .collect::<Vec<_>>()
            .join(",")
            .trim()
            .to_owned()
    } else {
        parts.nth(1)
            .ok_or_else(|| anyhow!("Missing color stops"))?
            .trim()
            .to_owned()
    };

    let stops = parse_color_stops(&stops_str)?;

    Ok((direction, stops))
}

/// 解析渐变方向
fn parse_direction(direction_str: &str) -> Result<GradientDirection> {
    let dir_str = direction_str.trim().to_lowercase();
    
    // 处理角度格式
    if let Some(degrees) = dir_str.strip_suffix("deg") {
        let angle = degrees.parse::<f32>()
            .map_err(|_| anyhow!("Invalid angle: {}", dir_str))?;
        return Ok(GradientDirection::Angle(angle));
    }

    // 处理关键词方向
    match dir_str.as_str() {
        // 水平
        "to right" | "to left" => Ok(GradientDirection::Horizontal),
        // 垂直
        "to bottom" | "to top" => Ok(GradientDirection::Vertical),
        // 对角线
        "to top right" | "to right top" => Ok(GradientDirection::Angle(45.0)),
        "to bottom right" | "to right bottom" => Ok(GradientDirection::Angle(135.0)),
        "to bottom left" | "to left bottom" => Ok(GradientDirection::Angle(225.0)),
        "to top left" | "to left top" => Ok(GradientDirection::Angle(315.0)),
        _ => {
            if csscolorparser::parse(&dir_str).is_ok() {
                Ok(GradientDirection::None)
            } else {
                Err(anyhow!("Unsupported direction: {}", dir_str))
            }
        },
    }
}

/// 解析颜色停止点
fn parse_color_stops(stops_str: &str) -> Result<Vec<(Rgba<u8>, f32)>> {
    let mut stops = Vec::new();
    let parts: Vec<&str> = stops_str.split(',').map(|s| s.trim()).collect();

    for part in parts {
        let (color_part, pos) = if let Some((color, pos)) = part.split_once(char::is_whitespace) {
            (color.trim(), Some(pos.trim()))
        } else {
            (part.trim(), None)
        };

        let color = parse_color_str(color_part)?;
        let position = match pos {
            Some(p) => parse_position(p)?,
            None => -1.0, // 标记未指定位置
        };
        stops.push((color, position));
    }

    // 自动分配位置
    assign_positions(&mut stops);
    stops.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
    Ok(stops)
}

/// 解析位置值 (0%, 0.5, 100% 等)
fn parse_position(pos_str: &str) -> Result<f32> {
    if let Some(percent) = pos_str.strip_suffix('%') {
        Ok(percent.parse::<f32>()? / 100.0)
    } else {
        Ok(pos_str.parse::<f32>()?)
    }
}

/// 自动分配未指定位置
fn assign_positions(stops: &mut [(Rgba<u8>, f32)]) {
    let has_custom_pos = stops.iter().any(|(_, pos)| *pos >= 0.0);
    
    if !has_custom_pos {
        // 均匀分布
        let len = stops.len();
        for (i, (_, pos)) in stops.iter_mut().enumerate() {
            *pos = if len == 1 { 0.5 } else { i as f32 / (len - 1) as f32 };
        }
    } else {
        // 处理首尾
        if stops[0].1 < 0.0 { stops[0].1 = 0.0; }
        if stops.last().unwrap().1 < 0.0 { stops.last_mut().unwrap().1 = 1.0; }
        
        // 填充中间位置
        let mut prev_idx = 0;
        for i in 1..stops.len() {
            if stops[i].1 >= 0.0 {
                let step = (stops[i].1 - stops[prev_idx].1) / (i - prev_idx) as f32;
                for j in (prev_idx + 1)..i {
                    stops[j].1 = stops[prev_idx].1 + step * (j - prev_idx) as f32;
                }
                prev_idx = i;
            }
        }
    }
}

/// 生成渐变图片
fn generate_gradient_image(
    width: u32,
    height: u32,
    direction: GradientDirection,
    stops: Vec<(Rgba<u8>, f32)>,
) -> Result<RgbaImage> {
    let mut img = ImageBuffer::new(width, height);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let t = match direction {
            GradientDirection::Horizontal => x as f32 / (width - 1) as f32,
            GradientDirection::Vertical | GradientDirection::None => y as f32 / (height - 1) as f32,
            GradientDirection::Angle(deg) => {
                // 转换 CSS 角度到数学极坐标角度，并翻转 Y 轴
                let math_deg = 90.0 - deg; // CSS 0deg → 数学90度 (向上)
                let rad = math_deg.to_radians();

                // 计算方向向量 (考虑图像Y轴向下)
                let (dx, dy) = (rad.cos(), -rad.sin());

                // 计算投影范围
                let min_proj = (0.0 * dx + 0.0 * dy).min(0.0 * dx + height as f32 * dy)
                    .min(width as f32 * dx + 0.0 * dy)
                    .min(width as f32 * dx + height as f32 * dy);
                let max_proj = (0.0 * dx + 0.0 * dy).max(0.0 * dx + height as f32 * dy)
                    .max(width as f32 * dx + 0.0 * dy)
                    .max(width as f32 * dx + height as f32 * dy);
                
                let proj = x as f32 * dx + y as f32 * dy;
                let t = (proj - min_proj) / (max_proj - min_proj);
                t.clamp(0.0, 1.0)
            }
        };
        
        *pixel = interpolate_color(t, &stops);
    }

    Ok(img)
}

/// 颜色插值
fn interpolate_color(t: f32, stops: &[(Rgba<u8>, f32)]) -> Rgba<u8> {
    let t = t.clamp(0.0, 1.0);
    
    // 查找相邻停止点
    let (prev, next) = stops.windows(2)
        .find(|w| w[0].1 <= t && t <= w[1].1)
        .map(|w| (w[0], w[1]))
        .unwrap_or((stops[0], *stops.last().unwrap()));
    
    let ratio = (t - prev.1) / (next.1 - prev.1);
    lerp_color(prev.0, next.0, ratio)
}

/// 颜色线性插值
fn lerp_color(a: Rgba<u8>, b: Rgba<u8>, t: f32) -> Rgba<u8> {
    Rgba([
        lerp_channel(a[0], b[0], t),
        lerp_channel(a[1], b[1], t),
        lerp_channel(a[2], b[2], t),
        lerp_channel(a[3], b[3], t),
    ])
}

/// 单通道插值
fn lerp_channel(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 * (1.0 - t) + b as f32 * t).round() as u8
}