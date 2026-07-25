use resvg::tiny_skia::{
    FillRule, FilterQuality, IntSize, Paint, PathBuilder, Pattern, Pixmap, SpreadMode, Transform,
};
use smooth_frame::{PathCommand, SmoothRect};

pub fn add_corners(
    width: u32,
    height: u32,
    rgba_date: &[u8],
    radius: f64,
    smoothing: /* [0, 1] */ f64,
) -> Option<Vec<u8>> {
    let path = {
        let mut pb = PathBuilder::new();

        let smooth_rect_path = SmoothRect::new(width as f64, height as f64)
            .with_radius(radius)
            .with_smoothing(smoothing)
            .to_path();

        for command in smooth_rect_path.commands() {
            match command {
                PathCommand::MoveTo(p) => {
                    pb.move_to(p.x as f32, p.y as f32);
                }
                PathCommand::LineTo(p) => {
                    pb.line_to(p.x as f32, p.y as f32);
                }
                PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                    pb.cubic_to(
                        ctrl1.x as f32,
                        ctrl1.y as f32,
                        ctrl2.x as f32,
                        ctrl2.y as f32,
                        to.x as f32,
                        to.y as f32,
                    );
                }
                PathCommand::Close => {
                    pb.close();
                }
            }
        }

        pb.finish()?
    };

    let source_pixmap = Pixmap::from_vec(rgba_date.to_vec(), IntSize::from_wh(width, height)?)?;

    let shader = Pattern::new(
        source_pixmap.as_ref(),
        SpreadMode::Pad,
        FilterQuality::Bilinear,
        1.0,
        Transform::identity(),
    );

    let mut paint = Paint::default();
    paint.anti_alias = true; // 抗锯齿
    paint.shader = shader;

    let mut pixmap = Pixmap::new(width, height)?;
    pixmap.fill_path(
        &path,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );

    Some(pixmap.take())
}
