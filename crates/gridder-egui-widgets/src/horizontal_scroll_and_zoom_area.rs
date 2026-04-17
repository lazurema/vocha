use egui::Response;

pub struct HorizontalScrollAndZoomArea<'a> {
    /// The number of points (logical pixels) per second.
    points_per_second: &'a mut f32,
    /// The horizontal offset in points.
    offset_points: &'a mut f32,
    /// The maximum number of seconds that can be shown in the view. This is
    /// used to limit the zoom level.
    max_seconds_in_view: f32,
}

impl<'a> HorizontalScrollAndZoomArea<'a> {
    pub fn new(
        points_per_second: &'a mut f32,
        offset_points: &'a mut f32,
        max_seconds_in_view: f32,
    ) -> Self {
        Self {
            points_per_second,
            offset_points,
            max_seconds_in_view,
        }
    }

    /// ## TODO
    ///
    /// Improve the signature of `content`.
    pub fn show(
        self,
        ui: &mut egui::Ui,
        content: impl FnOnce(&mut egui::Ui, f32, f32) -> Response,
    ) {
        let Self {
            points_per_second,
            offset_points,
            max_seconds_in_view: max_seconds,
        } = self;

        let resp = content(ui, *points_per_second, *offset_points);

        let cursor_pos = if let Some(info) = ui.multi_touch() {
            Some(info.center_pos)
        } else {
            ui.pointer_latest_pos()
        };

        if let Some(cursor_pos) = cursor_pos
            && resp.rect.contains(cursor_pos)
        {
            let zoom_delta = ui.input(|input| input.zoom_delta_2d().x);
            if (zoom_delta - 1.0).abs() > f32::EPSILON {
                // Using f64 to mitigate precision issues.
                let cursor_rel_x = cursor_pos.x as f64 - resp.rect.left() as f64;
                let cursor_seconds =
                    (*offset_points as f64 + cursor_rel_x) / *points_per_second as f64;

                *points_per_second *= zoom_delta;
                if resp.rect.width() / *points_per_second > max_seconds {
                    *points_per_second = resp.rect.width() / max_seconds;
                }

                *offset_points = (cursor_seconds * *points_per_second as f64 - cursor_rel_x) as f32;
            }

            let scroll_delta = ui.input(|input| input.smooth_scroll_delta());
            if scroll_delta.x.abs() > f32::EPSILON {
                *offset_points -= scroll_delta.x;
            }

            *offset_points = offset_points.clamp(
                0.0,
                (max_seconds * *points_per_second - resp.rect.width()).max(0.0),
            );
        }
    }
}
