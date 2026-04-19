use crate::view_range::ViewRange;

pub struct HorizontalScrollBar<'a> {
    view_range: &'a mut ViewRange,
}

impl<'a> HorizontalScrollBar<'a> {
    pub fn new(view_range: &'a mut ViewRange) -> Self {
        Self { view_range }
    }
}

impl egui::Widget for HorizontalScrollBar<'_> {
    /// ## Note
    ///
    /// GitHub Copilot originally wrote a barely functioning version, which was
    /// then fixed (90% rewritten) by human.
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, resp) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), ui.spacing().interact_size.y * 0.5),
            egui::Sense::click_and_drag(),
        );

        ui.painter().rect_filled(
            rect,
            rect.height() / 2.0,
            ui.style().visuals.extreme_bg_color,
        );

        let thumb_rect = egui::Rect::from_min_size(
            egui::pos2(
                rect.left() + (self.view_range.start_points(rect.width())) as f32,
                rect.top(),
            ),
            egui::vec2(
                self.view_range.view_points(rect.width()) as f32,
                rect.height(),
            ),
        );

        let mut delta_points = 0.0;
        Thumb::new(thumb_rect).show(ui, &mut delta_points);
        if delta_points != 0.0 {
            let delta_ratio = delta_points / rect.width();
            self.view_range.shift(delta_ratio);
        }

        if let Some(cursor_pos) = ui.pointer_interact_pos()
            && resp.is_pointer_button_down_on()
        {
            self.view_range
                .move_to((cursor_pos.x as f64 - rect.left() as f64) / rect.width() as f64);
        }

        resp
    }
}

struct Thumb {
    rect: egui::Rect,
}

impl Thumb {
    pub fn new(rect: egui::Rect) -> Self {
        Self { rect }
    }

    pub fn show(self, ui: &mut egui::Ui, delta_points: &mut f32) {
        let resp = ui.interact(self.rect, ui.id(), egui::Sense::click_and_drag());

        let thumb_color = if resp.dragged() {
            *delta_points = resp.drag_delta().x;
            ui.style().visuals.widgets.active.bg_fill
        } else if resp.is_pointer_button_down_on() {
            ui.style().visuals.widgets.active.bg_fill
        } else if resp.hovered() {
            ui.style().visuals.widgets.hovered.bg_fill
        } else {
            ui.style().visuals.widgets.inactive.bg_fill
        };

        ui.painter()
            .rect_filled(self.rect, self.rect.height() / 2.0, thumb_color);
    }
}
