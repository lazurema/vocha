use egui::Response;

use crate::view_range::ViewRange;

pub struct HorizontalScrollAndZoomArea<'a> {
    view_range: &'a mut ViewRange,
}

impl<'a> HorizontalScrollAndZoomArea<'a> {
    pub fn new(view_range: &'a mut ViewRange) -> Self {
        Self { view_range }
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        content: impl FnOnce(&mut egui::Ui, &ViewRange) -> Response,
    ) {
        let Self { view_range } = self;

        let resp = content(ui, view_range);

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
                let cursor_ratio = ((cursor_pos.x as f64 - resp.rect.left() as f64)
                    / resp.rect.width() as f64)
                    .clamp(0.0, 1.0);
                view_range.zoom(zoom_delta, cursor_ratio);
            }

            let scroll_delta = ui.input(|input| input.smooth_scroll_delta());
            if scroll_delta.x.abs() > f32::EPSILON {
                let delta_ratio =
                    scroll_delta.x as f64 / resp.rect.width() as f64 * view_range.view_ratio();

                view_range.shift(-delta_ratio);
            }
        }
    }
}
