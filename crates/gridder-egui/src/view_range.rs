pub struct ViewRange {
    start_ratio: f64,
    end_ratio: f64,
}

impl Default for ViewRange {
    fn default() -> Self {
        Self {
            start_ratio: 0.0,
            end_ratio: 1.0,
        }
    }
}

impl ViewRange {
    pub fn start_ratio(&self) -> f64 {
        self.start_ratio
    }
    pub fn end_ratio(&self) -> f64 {
        self.end_ratio
    }
    pub fn view_ratio(&self) -> f64 {
        self.end_ratio - self.start_ratio
    }

    #[inline(always)]
    pub fn start_points(&self, total_points: impl Into<f64>) -> f64 {
        self.start_ratio * total_points.into()
    }
    #[inline(always)]
    pub fn end_points(&self, total_points: impl Into<f64>) -> f64 {
        self.end_ratio * total_points.into()
    }
    #[inline(always)]
    pub fn view_points(&self, total_points: impl Into<f64>) -> f64 {
        self.view_ratio() * total_points.into()
    }

    pub fn update(&mut self, start_ratio: impl Into<f64>, end_ratio: impl Into<f64>) {
        let start_ratio = start_ratio.into();
        let end_ratio = end_ratio.into();

        self.start_ratio = start_ratio.clamp(0.0, 1.0);
        self.end_ratio = end_ratio.clamp(self.start_ratio, 1.0);
    }

    pub fn shift(&mut self, delta_ratio: impl Into<f64>) {
        let new_start = (self.start_ratio + delta_ratio.into()).clamp(0.0, 1.0 - self.view_ratio());
        self.update(new_start, new_start + self.view_ratio());
    }

    pub fn move_to(&mut self, pointer_ratio: impl Into<f64>) {
        let pointer_ratio = pointer_ratio.into();
        let new_start =
            (pointer_ratio - self.view_ratio() / 2.0).clamp(0.0, 1.0 - self.view_ratio());
        self.update(new_start, new_start + self.view_ratio());
    }

    /// ## Parameters
    ///
    /// - `pointer_ratio`: The ratio of the pointer position within the view (0.0
    ///   at the left edge, 1.0 at the right edge).
    ///
    /// ## TODO
    ///
    /// Fix the flickering issue.
    pub fn zoom(&mut self, zoom_delta: impl Into<f64>, pointer_ratio: impl Into<f64>) {
        let zoom_delta = zoom_delta.into();
        let pointer_ratio = pointer_ratio.into();

        let view_ratio = self.view_ratio();
        let new_view_ratio = (view_ratio / zoom_delta).clamp(f64::MIN_POSITIVE, 1.0);
        let zoom_center = self.start_ratio + pointer_ratio * view_ratio;
        let new_start =
            (zoom_center - pointer_ratio * new_view_ratio).clamp(0.0, 1.0 - new_view_ratio);
        self.update(new_start, new_start + new_view_ratio);
    }
}
