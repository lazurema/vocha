pub struct HorizontalScrollBar<'a> {
    start_percent: &'a mut f32,
    size_percent: f32,
}

impl<'a> HorizontalScrollBar<'a> {
    pub fn new(start_percent: &'a mut f32, size_percent: f32) -> Self {
        Self {
            start_percent,
            size_percent,
        }
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
            egui::pos2(rect.left() + *self.start_percent * rect.width(), rect.top()),
            egui::vec2(self.size_percent * rect.width(), rect.height()),
        );

        let mut delta_points = 0.0;
        Thumb::new(thumb_rect).show(ui, &mut delta_points);
        if delta_points != 0.0 {
            let delta_percent = delta_points / rect.width();
            *self.start_percent = (*self.start_percent + delta_percent).clamp(0.0, 1.0);
        }

        if let Some(cursor_pos) = ui.pointer_interact_pos()
            && resp.is_pointer_button_down_on()
        {
            let click_percent = (cursor_pos.x - rect.left()) / rect.width();
            *self.start_percent = (click_percent - self.size_percent / 2.0).clamp(0.0, 1.0);
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
