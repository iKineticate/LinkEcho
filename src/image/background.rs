use super::Result;

use resvg::tiny_skia::*;

type Color = (
    /* r */ u8,
    /* g */ u8,
    /* b */ u8,
    /* a */ u8,
);

type Angle = f32;
type IsCircle = bool;

#[derive(Debug, PartialEq, Clone)]
pub enum GradientColors {
    /// 所有颜色自动平均分配位置
    Auto(Vec<Color>),
    /// 所有颜色均显式指定位置（位置会按升序排列）
    Positioned(Vec<(f32, Color)>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Gradient {
    None(Color),
    Linear(Option<Angle>, GradientColors),
    Radial(IsCircle, GradientColors),
}

#[derive(Debug, PartialEq, Clone)]
pub struct BackgroundSource {
    width: u32,
    height: u32,
    gradient: Gradient,
    buffer: Option<Vec<u8>>,
}

impl BackgroundSource {
    pub fn new(width: u32, height: u32, gradient: Gradient) -> Result<Self> {
        let buffer = match gradient {
            Gradient::None(color) => {
                let (r, g, b, a) = color;
                let mut buffer = vec![0; (width * height) as usize];
                for pixel in buffer.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&[r, g, b, a]);
                }
                Some(buffer)
            }
            Gradient::Linear(angle, ref colors) => {
                let buffer = create_liner_gradient(width, height, angle, colors)?;
                Some(buffer)
            }
            Gradient::Radial(shape, ref colors) => {
                let buffer = create_radial_gradient(width, height, shape, colors)?;
                Some(buffer)
            }
        };

        Ok(Self {
            width,
            height,
            gradient,
            buffer,
        })
    }
}

fn endpoints_from_angle(width: u32, height: u32, angle_deg: Option<Angle>) -> (Point, Point) {
    let w = width as f32;
    let h = height as f32;
    let angle_rad = angle_deg.unwrap_or_default().to_radians();
    let (sin, cos) = angle_rad.sin_cos();
    let center_x = w / 2.0;
    let center_y = h / 2.0;
    let half_diag = (w * w + h * h).sqrt() / 2.0;
    let offset_x = cos * half_diag;
    let offset_y = sin * half_diag;
    let start = Point::from_xy(center_x - offset_x, center_y - offset_y);
    let end = Point::from_xy(center_x + offset_x, center_y + offset_y);
    (start, end)
}

fn to_gradient_stops(colors: &GradientColors) -> Result<Vec<GradientStop>> {
    use resvg::tiny_skia::Color;
    Ok(match colors {
        GradientColors::Auto(colors) => {
            if colors.is_empty() {
                return Err("at least one color stop is required".into());
            }

            let n = colors.len();
            colors
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let pos = i as f32 / (n - 1).max(1) as f32;
                    GradientStop::new(pos, Color::from_rgba8(c.0, c.1, c.2, c.3))
                })
                .collect()
        }
        GradientColors::Positioned(stops) => {
            if stops.is_empty() {
                return Err("at least one color stop is required".into());
            }

            let mut sorted = stops.clone();
            sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            // 自动补齐：如果第一个不在 0.0，复制第一个颜色到 0.0
            if sorted.first().unwrap().0 > 0.0 {
                let first_color = sorted[0].1.clone();
                sorted.insert(0, (0.0, first_color));
            }

            // 自动补齐：如果最后一个不在 1.0，复制最后一个颜色到 1.0
            if sorted.last().unwrap().0 < 1.0 {
                let last_color = sorted.last().unwrap().1.clone();
                sorted.push((1.0, last_color));
            }

            sorted
                .iter()
                .map(|(pos, c)| {
                    if !(0.0..=1.0).contains(pos) {
                        return Err(
                            format!("invalid color position {pos}, must be in [0,1]").into()
                        );
                    };
                    Ok(GradientStop::new(
                        *pos,
                        Color::from_rgba8(c.0, c.1, c.2, c.3),
                    ))
                })
                .collect::<Result<Vec<_>>>()?
        }
    })
}

fn create_liner_gradient(
    width: u32,
    height: u32,
    angle: Option<Angle>,
    colors: &GradientColors,
) -> Result<Vec<u8>> {
    let stops = to_gradient_stops(colors)?;

    let (start, end) = endpoints_from_angle(width, height, angle);

    let shader = LinearGradient::new(start, end, stops, SpreadMode::Pad, Transform::identity())
        .ok_or("failed to create linear gradient")?;

    let mut paint = Paint::default();
    paint.shader = shader;

    let mut pixmap = Pixmap::new(width, height).ok_or("failed to create pixmap")?;
    let rect = Rect::from_xywh(0.0, 0.0, width as f32, height as f32).unwrap();
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);

    Ok(pixmap.take())
}

fn create_radial_gradient(
    width: u32,
    height: u32,
    shape: IsCircle,
    colors: &GradientColors,
) -> Result<Vec<u8>> {
    let stops = to_gradient_stops(colors)?;

    let max_dim = width.max(height) as f32;
    let end_radius = max_dim / 2.0;

    let center = Point::from_xy(width as f32 / 2.0, height as f32 / 2.0);

    let shader = RadialGradient::new(
        center,
        0.0f32,
        center,
        end_radius,
        stops,
        SpreadMode::Pad,
        Transform::identity(),
    )
    .ok_or("failed to create radial gradient")?;

    let mut paint = Paint::default();
    paint.shader = shader;

    let mut pixmap = Pixmap::new(width, height).ok_or("failed to create pixmap")?;
    let rect = Rect::from_xywh(0.0, 0.0, width as f32, height as f32).unwrap();

    let transform = if !shape {
        let scale_x = width as f32 / max_dim;
        let scale_y = height as f32 / max_dim;
        Transform::from_translate(-center.x, -center.y)
            .post_scale(scale_x, scale_y)
            .post_translate(center.x, center.y)
    } else {
        Transform::default()
    };

    pixmap.fill_rect(rect, &paint, transform, None);

    Ok(pixmap.take())
}

#[test]
fn test_create_liner_gradient() {
    let start = std::time::Instant::now();

    let (width, height) = (256, 256);

    let buffer = create_liner_gradient(
        width,
        height,
        Some(90.0),
        &GradientColors::Positioned(vec![
            (0.0, (230, 100, 100, 255)),
            (0.3, (146, 152, 228, 255)),
            (1.0, (0, 235, 129, 255)),
        ]),
    )
    .unwrap();

    image::RgbaImage::from_vec(width, height, buffer)
        .unwrap()
        .save(r"C:\Users\11593\Downloads\test1.png")
        .unwrap();

    println!("Time: {:?}", start.elapsed().as_secs_f64());
}

#[test]
fn test_create_radial_gradient() {
    let start = std::time::Instant::now();

    let (width, height) = (256, 256);

    let buffer = create_radial_gradient(
        width,
        height,
        true,
        &GradientColors::Positioned(vec![
            (0.0, (230, 100, 100, 255)),
            (1.0, (146, 152, 228, 255)),
            (0.3, (0, 235, 129, 255)),
        ]),
    )
    .unwrap();

    image::RgbaImage::from_vec(width, height, buffer)
        .unwrap()
        .save(r"C:\Users\11593\Downloads\test2.png")
        .unwrap();

    println!("Time: {:?}", start.elapsed().as_secs_f64());
}

#[test]
fn add_corners_background() {
    let start = std::time::Instant::now();

    let (width, height) = (256, 256);

    let buffer = create_liner_gradient(
        width,
        height,
        Some(90.0),
        &GradientColors::Auto(vec![
            (255, 255, 255, 255),
            (146, 152, 228, 255),
            (0, 235, 129, 255),
        ]),
    )
    .unwrap();

    let buffer = super::corner::add_corners(width, height, &buffer, 58.0, 0.6).unwrap();

    image::RgbaImage::from_vec(width, height, buffer)
        .unwrap()
        .save(r"C:\Users\11593\Downloads\test3.png")
        .unwrap();

    println!("Time: {:?}", start.elapsed().as_secs_f64());
}

#[test]
fn test_save_ico() {
    let icon_image =
        super::icon::IconImage::default(r"C:\Users\11593\Downloads\test3.png").unwrap();
    icon_image
        .save_to_ico(r"C:\Users\11593\Downloads\test3.ico")
        .unwrap();
}
