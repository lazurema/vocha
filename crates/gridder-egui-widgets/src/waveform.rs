use std::{
    collections::HashMap,
    num::{NonZeroU16, NonZeroU32},
    sync::{Arc, Mutex},
};

use egui::epaint;

pub struct Waveform {
    desired_size: egui::Vec2,
    /// The number of points (logical pixels) to display per second of audio.
    points_per_second: f32,
    /// The horizontal offset of the waveform in points.
    offset_points: f32,
    /// A function that selects the paint mode based on the current zoom level (
    /// [`Self::points_per_second`]).
    paint_mode_selector: Box<dyn Fn(f32) -> WaveformPaintMode>,
    /// The color to paint the waveform with.
    color: egui::Color32,

    data: Arc<WaveData>,
    channel: u16,
}

type DrawerPublisher<'a> = egui::cache::FramePublisher<egui::Id, Arc<Mutex<DrawerWithCache>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveformPaintMode {
    /// A line connecting all sample points. When a sample spans multiple pixels,
    /// it's drawn as a horizontal line segment.
    Line,
    /// A filled area showing all samples extending from the center axis,
    /// with no mirroring.
    Bar,
    /// A filled area showing samples extending from the center axis in both
    /// directions, mirrored symmetrically.
    BarSymmetric,
    /// A filled area showing absolute values of samples above the center axis.
    BarAbsolutePositive,
    /// A filled area showing absolute values of samples below the center axis.
    BarAbsoluteNegative,
}

pub struct WaveData {
    /// The unique identifier of the waveform. This is used for caching.
    pub id: egui::Id,

    /// The sample rate of the audio data that the waveform represents.
    pub sample_rate: NonZeroU32,
    /// The number of channels of the audio data that the waveform represents.
    pub channels: NonZeroU16,
    /// The audio samples that the waveform represents. The samples should
    /// normally be in the range of -1.0 to 1.0.
    pub samples_interleaved: Arc<[f32]>,
}

fn default_paint_mode_selector(samples_per_pixel: f32) -> WaveformPaintMode {
    if samples_per_pixel <= 1.0 {
        WaveformPaintMode::Line
    } else if samples_per_pixel <= 10.0 {
        WaveformPaintMode::Bar
    } else {
        WaveformPaintMode::BarSymmetric
    }
}

impl Waveform {
    pub fn new(desired_size: egui::Vec2, data: Arc<WaveData>, channel: u16) -> Self {
        Self {
            desired_size,
            points_per_second: 100.0,
            offset_points: 0.0,
            paint_mode_selector: Box::new(default_paint_mode_selector),
            color: egui::Color32::GRAY,
            data,
            channel,
        }
    }

    pub fn points_per_second(mut self, points_per_second: f32) -> Self {
        self.points_per_second = points_per_second;
        self
    }

    pub fn offset_points(mut self, offset_points: f32) -> Self {
        self.offset_points = offset_points;
        self
    }

    pub fn paint_mode_selector(
        mut self,
        paint_mode_selector: Box<dyn Fn(f32) -> WaveformPaintMode>,
    ) -> Self {
        self.paint_mode_selector = paint_mode_selector;
        self
    }

    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = color;
        self
    }

    fn select_paint_mode(&self, pixels_per_point: f32) -> WaveformPaintMode {
        let samples_per_pixel =
            self.data.sample_rate.get() as f32 / self.points_per_second / pixels_per_point;
        (self.paint_mode_selector)(samples_per_pixel)
    }
}

impl egui::Widget for Waveform {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let resp = ui.allocate_response(self.desired_size, egui::Sense::empty());

        let pixels_per_point = ui.ctx().pixels_per_point();
        let height_pixels = (resp.rect.height() * pixels_per_point).floor() as usize;
        let side_height_pixels = if height_pixels % 2 == 0 {
            height_pixels / 2 - 1
        } else {
            height_pixels / 2
        };

        let drawer_id = self.data.id.with(self.channel);
        let drawer = ui.memory_mut(|mem| {
            let drawer_publisher = mem.caches.cache::<DrawerPublisher<'_>>();
            drawer_publisher.get(&drawer_id).cloned()
        });

        let drawer = DrawerWithCache::reuse_or_new(
            drawer,
            self.select_paint_mode(pixels_per_point),
            side_height_pixels,
            self.points_per_second,
            pixels_per_point,
            self.data.clone(),
            self.channel,
        );
        drawer
            .lock()
            .expect("Failed to acquire lock on drawer.")
            .paint(ui, resp.rect, self.offset_points, self.color);
        ui.memory_mut(|mem| {
            let drawer_publisher = mem.caches.cache::<DrawerPublisher<'_>>();
            drawer_publisher.set(drawer_id, drawer);
        });

        resp
    }
}

const MAX_TEXTURE_WIDTH: usize = 1024;

struct DrawerWithCache {
    mode: WaveformPaintMode,
    /// The height of the waveform on one side of the center axis (excluding the
    /// center axis itself) in physical pixels.
    side_height_pixels: usize,
    points_per_second: f32,
    pixels_per_point: f32,

    data: Arc<WaveData>,
    channel: u16,

    tex_manager: Option<Arc<epaint::mutex::RwLock<epaint::TextureManager>>>,
    /// A hash map from the index of the texture to the texture id.
    ///
    /// The index of the texture is determined by the column of physical pixels
    /// that the texture represents. Texture `k` covers physical pixel columns
    /// `k * MAX_TEXTURE_WIDTH` to `(k + 1) * MAX_TEXTURE_WIDTH - 1`.
    saved_texture_ids: HashMap<usize, egui::TextureId>,

    /// Audio samples (of the rendered channel) per physical pixel.
    samples_per_pixel: f32,
    /// Total width of the waveform in physical pixels.
    width_pixels: usize,
}

impl DrawerWithCache {
    fn new(
        mode: WaveformPaintMode,
        side_height_pixels: usize,
        points_per_second: f32,
        pixels_per_point: f32,
        waveform_data: Arc<WaveData>,
        channel: u16,
    ) -> Self {
        // Audio samples (of the rendered channel) per physical pixel.
        let samples_per_pixel =
            waveform_data.sample_rate.get() as f32 / points_per_second / pixels_per_point;
        // Total waveform width in physical pixels.
        let width_pixels = ((waveform_data.samples_interleaved.len() as f32
            / waveform_data.channels.get() as f32)
            / samples_per_pixel)
            .ceil() as usize;

        Self {
            mode,
            side_height_pixels,
            points_per_second,
            pixels_per_point,
            data: waveform_data,
            channel,
            tex_manager: None,
            saved_texture_ids: HashMap::new(),
            samples_per_pixel,
            width_pixels,
        }
    }

    fn is_valid_for(
        &self,
        mode: WaveformPaintMode,
        side_height_pixels: usize,
        points_per_second: f32,
        pixels_per_point: f32,
        wave_data: &WaveData,
        channel: u16,
    ) -> bool {
        self.mode == mode
            && self.side_height_pixels == side_height_pixels
            && self.points_per_second == points_per_second
            && self.pixels_per_point == pixels_per_point
            && self.data.id == wave_data.id
            && self.channel == channel
    }

    fn reuse_or_new(
        zelf: Option<Arc<Mutex<Self>>>,
        mode: WaveformPaintMode,
        side_height_pixels: usize,
        points_per_second: f32,
        pixels_per_point: f32,
        wave_data: Arc<WaveData>,
        channel: u16,
    ) -> Arc<Mutex<Self>> {
        if let Some(zelf) = zelf
            && zelf
                .lock()
                .expect("Failed to acquire lock on drawer.")
                .is_valid_for(
                    mode,
                    side_height_pixels,
                    points_per_second,
                    pixels_per_point,
                    &wave_data,
                    channel,
                )
        {
            zelf.clone()
        } else {
            Arc::new(Mutex::new(Self::new(
                mode,
                side_height_pixels,
                points_per_second,
                pixels_per_point,
                wave_data,
                channel,
            )))
        }
    }

    fn height_pixels(&self) -> usize {
        self.side_height_pixels * 2 + 1
    }

    fn paint(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        offset_points: f32,
        color: egui::Color32,
    ) {
        let tex_manager = ui.ctx().tex_manager().clone();

        let painter = ui.painter_at(rect);

        let visible_texture_range =
            self.calculate_visible_texture_range(offset_points, rect.width());
        // Width of one texture in logical points: MAX_TEXTURE_WIDTH physical
        // pixels / scale.
        let texture_width_points = MAX_TEXTURE_WIDTH as f32 / self.pixels_per_point;
        // Height of the texture image in logical points.
        let texture_height_points = self.height_pixels() as f32 / self.pixels_per_point;

        for texture_index in visible_texture_range.clone() {
            let texture_id = if let Some(texture_id) = self.saved_texture_ids.get(&texture_index) {
                *texture_id
            } else {
                let tex_manager = &mut tex_manager.write();

                let texture_id = tex_manager.alloc(
                    "waveform".to_owned(),
                    self.draw_texture(texture_index).into(),
                    egui::TextureOptions::default(),
                );
                self.saved_texture_ids.insert(texture_index, texture_id);
                texture_id
            };

            // Texture k starts at physical pixel k * MAX_TEXTURE_WIDTH, which
            // in logical points (relative to the widget's left edge) is:
            // `k * MAX_TEXTURE_WIDTH / pixels_per_point - offset_points`
            let texture_rect = egui::Rect::from_min_size(
                rect.min
                    + egui::vec2(
                        texture_index as f32 * texture_width_points - offset_points,
                        0.0,
                    ),
                egui::vec2(texture_width_points, texture_height_points),
            );
            let uv = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1.0, 1.0));
            painter.image(texture_id, texture_rect, uv, color);
        }

        self.saved_texture_ids
            .retain(|&texture_index, &mut texture_id| {
                if visible_texture_range.contains(&texture_index) {
                    true
                } else {
                    if let Some(tex_manager) = &self.tex_manager {
                        tex_manager.write().free(texture_id);
                    }
                    false
                }
            });

        if self.tex_manager.is_none() {
            // For `Drop`.
            self.tex_manager = Some(tex_manager);
        }
    }

    fn calculate_visible_texture_range(
        &self,
        offset_points: f32,
        width_points: f32,
    ) -> std::ops::Range<usize> {
        // Convert from logical points to physical pixels.
        let offset_pixels = (offset_points * self.pixels_per_point).floor() as usize;

        if offset_pixels >= self.width_pixels {
            return 0..0;
        }

        let display_width_pixels = (width_points * self.pixels_per_point).ceil() as usize;
        // Clamp to the actual waveform extent.
        let end_pixel = (offset_pixels + display_width_pixels).min(self.width_pixels);

        let first_texture_index = offset_pixels / MAX_TEXTURE_WIDTH;
        // Ceiling division so the texture containing `end_pixel - 1` is
        // included.
        let last_texture_index = (end_pixel + MAX_TEXTURE_WIDTH - 1) / MAX_TEXTURE_WIDTH;

        first_texture_index..last_texture_index
    }

    fn draw_texture(&self, texture_index: usize) -> egui::ColorImage {
        if self.samples_per_pixel <= 1.0 {
            self.draw_texture_multiple_pixels_per_sample(texture_index)
        } else {
            self.draw_texture_multiple_samples_per_pixel(texture_index)
        }
    }

    fn draw_texture_multiple_pixels_per_sample(&self, texture_index: usize) -> egui::ColorImage {
        let mut image = egui::ColorImage::filled(
            [MAX_TEXTURE_WIDTH, self.height_pixels()],
            egui::Color32::TRANSPARENT,
        );
        let drawer = self.value_drawer();

        for i in 0..MAX_TEXTURE_WIDTH {
            let sample_index = (((texture_index * MAX_TEXTURE_WIDTH + i) as f32
                * self.samples_per_pixel)
                .round() as usize)
                * self.data.channels.get() as usize
                + self.channel as usize;
            let Some(sample) = self.data.samples_interleaved.get(sample_index) else {
                continue;
            };
            (drawer)(&mut image, i, *sample, self.side_height_pixels);
        }

        image
    }

    fn draw_texture_multiple_samples_per_pixel(&self, texture_index: usize) -> egui::ColorImage {
        let mut image = egui::ColorImage::filled(
            [MAX_TEXTURE_WIDTH, self.height_pixels()],
            egui::Color32::TRANSPARENT,
        );
        let drawer = self.value_drawer();

        for i in 0..MAX_TEXTURE_WIDTH {
            let start_sample_index = ((texture_index * MAX_TEXTURE_WIDTH + i) as f32
                * self.samples_per_pixel)
                .floor() as usize
                * self.data.channels.get() as usize
                + self.channel as usize;
            let end_sample_index = (((texture_index * MAX_TEXTURE_WIDTH + i + 1) as f32
                * self.samples_per_pixel)
                .ceil() as usize)
                * self.data.channels.get() as usize
                + self.channel as usize;

            let mut max_positive_value = 0.0f32;
            let mut min_negative_value = 0.0f32;
            for sample_index in
                (start_sample_index..end_sample_index).step_by(self.data.channels.get() as usize)
            {
                let Some(sample) = self.data.samples_interleaved.get(sample_index) else {
                    continue;
                };
                if sample >= &0.0 {
                    max_positive_value = max_positive_value.max(*sample);
                } else {
                    min_negative_value = min_negative_value.min(*sample);
                }
            }
            if max_positive_value > 0.0 {
                (drawer)(&mut image, i, max_positive_value, self.side_height_pixels);
            }
            if min_negative_value < 0.0 {
                (drawer)(&mut image, i, min_negative_value, self.side_height_pixels);
            }
        }

        image
    }

    fn value_drawer(&self) -> impl Fn(&mut egui::ColorImage, usize, f32, usize) + '_ {
        fn draw_in_line_mode(
            image: &mut egui::ColorImage,
            x: usize,
            value: f32,
            side_height_pixels: usize,
        ) {
            let value = value.clamp(-1.0, 1.0);
            let y = (side_height_pixels as f32 * (1.0 - value)).round() as usize;
            image[(x, y)] = egui::Color32::WHITE;
        }
        fn draw_in_bar_mode(
            image: &mut egui::ColorImage,
            x: usize,
            value: f32,
            side_height_pixels: usize,
        ) {
            let value = value.clamp(-1.0, 1.0);
            let center_y = side_height_pixels;
            let value_y = (side_height_pixels as f32 * value.abs()).round() as usize;
            if value >= 0.0 {
                for y in (center_y - value_y)..center_y {
                    image[(x, y)] = egui::Color32::WHITE;
                }
            } else {
                for y in center_y..(center_y + value_y) {
                    image[(x, y)] = egui::Color32::WHITE;
                }
            }
        }
        fn draw_in_bar_symmetric_mode(
            image: &mut egui::ColorImage,
            x: usize,
            value: f32,
            side_height_pixels: usize,
        ) {
            let value = value.clamp(-1.0, 1.0);
            let center_y = side_height_pixels;
            let value_y = (side_height_pixels as f32 * value.abs()).round() as usize;
            for y in (center_y - value_y)..(center_y + value_y) {
                image[(x, y)] = egui::Color32::WHITE;
            }
        }
        fn draw_in_bar_absolute_positive_mode(
            image: &mut egui::ColorImage,
            x: usize,
            value: f32,
            side_height_pixels: usize,
        ) {
            let value = value.clamp(-1.0, 1.0);
            let center_y = side_height_pixels;
            let value_y = (side_height_pixels as f32 * value.abs()).round() as usize;
            for y in (center_y - value_y)..center_y {
                image[(x, y)] = egui::Color32::WHITE;
            }
        }
        fn draw_in_bar_absolute_negative_mode(
            image: &mut egui::ColorImage,
            x: usize,
            value: f32,
            side_height_pixels: usize,
        ) {
            let value = value.clamp(-1.0, 1.0);
            let center_y = side_height_pixels;
            let value_y = (side_height_pixels as f32 * value.abs()).round() as usize;
            for y in center_y..(center_y + value_y) {
                image[(x, y)] = egui::Color32::WHITE;
            }
        }

        match self.mode {
            WaveformPaintMode::Line => draw_in_line_mode,
            WaveformPaintMode::Bar => draw_in_bar_mode,
            WaveformPaintMode::BarSymmetric => draw_in_bar_symmetric_mode,
            WaveformPaintMode::BarAbsolutePositive => draw_in_bar_absolute_positive_mode,
            WaveformPaintMode::BarAbsoluteNegative => draw_in_bar_absolute_negative_mode,
        }
    }
}

impl Drop for DrawerWithCache {
    fn drop(&mut self) {
        if let Some(tex_manager) = &self.tex_manager {
            let mut tex_manager = tex_manager.write();
            for texture_id in self.saved_texture_ids.values() {
                tex_manager.free(*texture_id);
            }
        }
    }
}
