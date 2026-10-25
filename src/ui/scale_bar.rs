use egui::{Color32, Pos2, Stroke};
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub enum Unit {
    Raw,
    #[default]
    Mm,
    Cm,
    M,
    Inch,
}
impl Unit {
    pub const ALL: [Self; 5] = [Self::Raw, Self::Mm, Self::Cm, Self::M, Self::Inch];
    pub fn label(self) -> &'static str {
        match self {
            Self::Raw => "units",
            Self::Mm => "mm",
            Self::Cm => "cm",
            Self::M => "m",
            Self::Inch => "inch",
        }
    }
    pub fn millimeters_per_unit(self) -> Option<f64> {
        match self {
            Self::Raw => None,
            Self::Mm => Some(1.),
            Self::Cm => Some(10.),
            Self::M => Some(1000.),
            Self::Inch => Some(25.4),
        }
    }
}
pub struct Scale {
    pub length: f64,
    pub width: f32,
    pub segments: usize,
}
pub fn calculate(
    radius: f64,
    span: f64,
    height_px: u32,
    pixels_per_point: f32,
    target: f32,
) -> Option<Scale> {
    if ![radius, span, pixels_per_point as f64, target as f64]
        .iter()
        .all(|v| v.is_finite() && *v > 0.)
        || height_px == 0
    {
        return None;
    }
    let per_point = span * radius / height_px as f64 * pixels_per_point as f64;
    let desired = per_point * target as f64;
    if !desired.is_finite() || desired <= 0. {
        return None;
    }
    let base = 10f64.powf(desired.log10().floor());
    let fraction = desired / base;
    let (factor, segments) = if fraction >= 5. {
        (5., 5)
    } else if fraction >= 2. {
        (2., 4)
    } else {
        (1., 5)
    };
    let length = factor * base;
    let width = (length / per_point) as f32;
    (width.is_finite() && width > 0. && length.is_finite() && length > 0.).then_some(Scale {
        length,
        width,
        segments,
    })
}
pub fn draw(ui: &mut egui::Ui, scale: Option<Scale>, unit: Unit) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().min(140.), 32.),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    let Some(scale) = scale else {
        painter.text(
            rect.left_top(),
            egui::Align2::LEFT_TOP,
            "Scale unavailable",
            egui::FontId::proportional(11.),
            Color32::GRAY,
        );
        return;
    };
    let start = rect.left_top() + egui::vec2(0., 5.);
    painter.line_segment(
        [start, start + egui::vec2(scale.width, 0.)],
        Stroke::new(1.5, Color32::WHITE),
    );
    for i in 0..=scale.segments * 2 {
        let x = start.x + scale.width * i as f32 / (scale.segments * 2) as f32;
        let h = if i % 2 == 0 { 7. } else { 4. };
        painter.line_segment(
            [Pos2::new(x, start.y - 2.), Pos2::new(x, start.y + h)],
            Stroke::new(1., Color32::WHITE),
        );
    }
    let value = if scale.length < 0.001 || scale.length >= 1e6 {
        format!("{:.2e}", scale.length)
    } else {
        format!("{:.3}", scale.length)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    };
    painter.text(
        start + egui::vec2(0., 12.),
        egui::Align2::LEFT_TOP,
        format!("0       {value} {}", unit.label()),
        egui::FontId::proportional(11.),
        Color32::WHITE,
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nice_lengths() {
        for (target, expected) in [
            (1.9, 1.),
            (2.1, 2.),
            (4.9, 2.),
            (5.1, 5.),
            (9.9, 5.),
            (10., 10.),
        ] {
            assert_eq!(calculate(1., 1., 1, 1., target).unwrap().length, expected);
        }
    }
    #[test]
    fn zoom_and_dpi() {
        let base = calculate(10., 2.8, 700, 1., 120.).unwrap();
        let zoom = calculate(10., 1.4, 700, 1., 120.).unwrap();
        assert!(
            ((zoom.width as f64 / zoom.length) / (base.width as f64 / base.length) - 2.).abs()
                < 1e-6
        );
        let dpi = calculate(10., 2.8, 1400, 2., 120.).unwrap();
        assert_eq!(base.length, dpi.length);
        assert_eq!(base.width, dpi.width);
        let dpi = calculate(10., 2.8, 1050, 1.5, 120.).unwrap();
        assert_eq!(base.width, dpi.width);
    }
    #[test]
    fn invalid_and_extreme() {
        assert!(calculate(1., 1., 0, 1., 100.).is_none());
        assert!(calculate(f64::NAN, 1., 10, 1., 100.).is_none());
        for r in [f32::MAX as f64, f32::from_bits(1) as f64] {
            assert!(calculate(r, 2.8, 700, 2., 100.).is_some());
        }
    }
    #[test]
    fn input_unit_interpretation() {
        assert_eq!(Unit::Inch.millimeters_per_unit(), Some(25.4));
        assert_eq!(Unit::Cm.millimeters_per_unit(), Some(10.));
        assert_eq!(Unit::Raw.millimeters_per_unit(), None);
    }
}
