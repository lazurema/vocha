use std::{
    io::Read,
    num::{NonZeroU16, NonZeroU32},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread,
};

use eframe::egui::{self, Widget as _};

use gridder_egui::{
    horizontal_scroll_and_zoom_area::HorizontalScrollAndZoomArea,
    horzontal_scroll_bar::HorizontalScrollBar,
    waveform::{WaveData, Waveform},
};
use rodio::Source as _;

use sha2::Digest as _;
use textgrid_rs::TextGrid;

use crate::l10n::{L10N, Term};

pub static SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &["wav", "mp3"];

pub struct ProjectPreview {
    audio_file_path: Option<PathBuf>,
    textgrid_file_path: Option<PathBuf>,

    is_from_dropping: bool,
}

struct HoveredOrDroppedFile<'a> {
    path: Option<&'a Path>,
}

impl ProjectPreview {
    /// Unfortunately, `ui.response().contains_pointer()` always returns `false`
    /// while files are being dragged.
    /// See: <https://github.com/emilk/egui/issues/4655>
    ///
    /// TODO(umajho): Investigate this in `egui`.
    ///
    /// As of now (2026/04/14), there is no luck: <https://github.com/rust-windowing/winit/issues/720#issuecomment-1290156438>
    ///
    /// A member of `winit` said there: “The cursor position should be
    /// broadcasted during the drag and drop. It could be that a particular
    /// platform isn't doing so which could indicate a bug.” If that’s the case,
    /// then at least both macOS and Windows are affected.
    pub fn extract_from_ui(ui: &mut egui::Ui) -> Option<ProjectPreview> {
        if true || ui.response().contains_pointer() {
            ui.input(|input| {
                ProjectPreview::try_from_dropped_files(input.raw.dropped_files.iter()).or_else(
                    || ProjectPreview::try_from_hovered_files(input.raw.hovered_files.iter()),
                )
            })
        } else {
            None
        }
    }

    fn try_from_files<'a>(
        files: impl Iterator<Item = HoveredOrDroppedFile<'a>>,
        is_from_dropping: bool,
    ) -> Option<ProjectPreview> {
        let mut ret = Self {
            audio_file_path: None,
            textgrid_file_path: None,
            is_from_dropping,
        };

        for hovered_file in files {
            if let Some(path) = &hovered_file.path
                && let Some(extension) = path.extension()
                && let Some(extension) = extension.to_str()
            {
                match extension.to_lowercase().as_str() {
                    "textgrid" => {
                        if ret.textgrid_file_path.is_some() {
                            // TODO: either support multiple files or show an
                            // error message.
                            return None;
                        }
                        ret.textgrid_file_path = Some(path.to_path_buf())
                    }
                    ext if SUPPORTED_AUDIO_EXTENSIONS.contains(&ext) => {
                        if ret.audio_file_path.is_some() {
                            // TODO: either support multiple files or show an
                            // error message.
                            return None;
                        }
                        ret.audio_file_path = Some(path.to_path_buf())
                    }
                    _ => {
                        // TODO: show an error message.
                        return None;
                    }
                }
            }
        }

        if ret.audio_file_path.is_some() || ret.textgrid_file_path.is_some() {
            Some(ret)
        } else {
            None
        }
    }

    pub fn try_from_hovered_files<'a>(
        hovered_files: impl Iterator<Item = &'a egui::HoveredFile>,
    ) -> Option<Self> {
        Self::try_from_files(
            hovered_files.map(|f| HoveredOrDroppedFile {
                path: f.path.as_deref(),
            }),
            false,
        )
    }

    pub fn try_from_dropped_files<'a>(
        dropped_files: impl Iterator<Item = &'a egui::DroppedFile>,
    ) -> Option<Self> {
        Self::try_from_files(
            dropped_files.map(|f| HoveredOrDroppedFile {
                path: f.path.as_deref(),
            }),
            true,
        )
    }

    pub fn ui(&self, ui: &mut egui::Ui, l10n: &L10N) {
        egui::Label::new(l10n.tl(&Term::ProjectPreviewText {
            has_audio: self.audio_file_path.is_some(),
            has_textgrid: self.textgrid_file_path.is_some(),
        }))
        .selectable(false)
        .ui(ui);
    }
}

pub struct Project {
    uuid: uuid::Uuid,

    audio_path: Option<PathBuf>,
    audio: ProjectAudioLifeCycle,
    textgrid_path: Option<PathBuf>,
    textgrid: ProjectTextGridLifeCycle,

    last_frame_title_name: Option<Option<String>>,

    points_per_second: f32,
    offset_points: f32,
}

enum ProjectAudioLifeCycle {
    Absent,
    Loading(Mutex<mpsc::Receiver<Result<ProjectAudio, String>>>),
    Loaded(ProjectAudio),
    Error(String),
}

#[derive(Clone)]
struct ProjectAudio {
    id: egui::Id,
    sample_rate: NonZeroU32,
    channels: NonZeroU16,
    samples_interleaved: Arc<[f32]>,
}

enum ProjectTextGridLifeCycle {
    Absent,
    Loading(Mutex<mpsc::Receiver<Result<TextGrid, String>>>),
    Loaded(TextGrid),
    Error(String),
}

impl Project {
    pub fn try_from_preview_from_dropping_files(preview: &ProjectPreview) -> Option<Self> {
        if !preview.is_from_dropping {
            return None;
        }

        let mut zelf = Self::new();

        if let Some(audio_file_path) = &preview.audio_file_path {
            zelf.load_audio(audio_file_path);
        }
        if let Some(textgrid_file_path) = &preview.textgrid_file_path {
            zelf.load_textgrid(textgrid_file_path);
        }

        Some(zelf)
    }

    fn new() -> Self {
        Self {
            uuid: uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
            audio_path: None,
            audio: ProjectAudioLifeCycle::Absent,
            textgrid_path: None,
            textgrid: ProjectTextGridLifeCycle::Absent,
            last_frame_title_name: None,
            points_per_second: 500.0,
            offset_points: 0.0,
        }
    }

    pub fn id(&self) -> uuid::Uuid {
        self.uuid
    }

    fn title_name(&self) -> Option<String> {
        if let Some(audio_file_path) = &self.audio_path {
            Some(audio_file_path.display().to_string())
        } else if let Some(textgrid_file_path) = &self.textgrid_path {
            Some(textgrid_file_path.display().to_string())
        } else {
            None
        }
    }

    fn update_title_name(&mut self, ui: &mut egui::Ui, l: &L10N) {
        let new_title_name = self.title_name();
        if self
            .last_frame_title_name
            .as_ref()
            .is_none_or(|old_title_name| old_title_name != &new_title_name)
        {
            ui.send_viewport_cmd(egui::ViewportCommand::Title(l.tl(&Term::GridderProject {
                name: new_title_name.clone(),
            })));
            self.last_frame_title_name = Some(new_title_name);
        }
    }

    fn load_audio(&mut self, path: &PathBuf) {
        let (tx, rx) = mpsc::channel();
        self.audio_path = Some(path.clone());
        self.audio = ProjectAudioLifeCycle::Loading(Mutex::new(rx));

        thread::spawn({
            let path = path.clone();
            move || {
                let file = std::fs::File::open(&path).map_err(|e| e.to_string());
                let loaded_data = file.and_then(|f| {
                    let source = rodio::Decoder::new(std::io::BufReader::new(f))
                        .map_err(|e| e.to_string())?;
                    let sample_rate = source.sample_rate();
                    let channels = source.channels();
                    let samples_interleaved = source.collect::<Vec<_>>();

                    let samples_sha256 =
                        sha2::Sha256::digest(bytemuck::cast_slice::<f32, u8>(&samples_interleaved));
                    let id = egui::Id::new(&samples_sha256);

                    Ok(ProjectAudio {
                        id,
                        sample_rate,
                        channels,
                        samples_interleaved: Arc::from(samples_interleaved),
                    })
                });

                tx.send(loaded_data).ok();
            }
        });
    }

    fn load_textgrid(&mut self, path: &PathBuf) {
        let (tx, rx) = mpsc::channel();
        self.textgrid_path = Some(path.clone());
        self.textgrid = ProjectTextGridLifeCycle::Loading(Mutex::new(rx));

        thread::spawn({
            let path = path.clone();
            move || {
                let file = std::fs::File::open(&path).map_err(|e| e.to_string());
                let loaded_data = file.and_then(|mut f| {
                    let mut content = Vec::new();
                    f.read_to_end(&mut content).map_err(|e| e.to_string())?;
                    TextGrid::parse_text_format(&content).map_err(|e| e.to_string())
                });

                tx.send(loaded_data).ok();
            }
        });
    }

    fn clear_audio(&mut self) {
        self.audio_path = None;
        self.audio = ProjectAudioLifeCycle::Absent;
    }

    fn clear_textgrid(&mut self) {
        self.textgrid_path = None;
        self.textgrid = ProjectTextGridLifeCycle::Absent;
    }

    fn update_audio(&mut self) {
        let mut new_audio = None;
        match self.audio {
            ProjectAudioLifeCycle::Loading(ref rx) => {
                let rx = rx
                    .lock()
                    .expect("Failed to acquire lock on audio loading rx.");
                if let Ok(result) = rx.try_recv() {
                    match result {
                        Ok(loaded_data) => {
                            new_audio = Some(ProjectAudioLifeCycle::Loaded(loaded_data));
                        }
                        Err(e) => {
                            new_audio = Some(ProjectAudioLifeCycle::Error(e));
                        }
                    }
                }
            }
            _ => {}
        }
        if let Some(new_audio) = new_audio {
            self.audio = new_audio;
        }
    }

    fn update_textgrid(&mut self) {
        let mut new_textgrid = None;
        match self.textgrid {
            ProjectTextGridLifeCycle::Loading(ref rx) => {
                let rx = rx
                    .lock()
                    .expect("Failed to acquire lock on textgrid loading rx.");
                if let Ok(result) = rx.try_recv() {
                    match result {
                        Ok(loaded_data) => {
                            new_textgrid = Some(ProjectTextGridLifeCycle::Loaded(loaded_data));
                        }
                        Err(e) => {
                            new_textgrid = Some(ProjectTextGridLifeCycle::Error(e));
                        }
                    }
                }
            }
            _ => {}
        }
        if let Some(new_textgrid) = new_textgrid {
            self.textgrid = new_textgrid;
        }
    }

    fn max_seconds(&self) -> f32 {
        let max_seconds_from_audio = if let ProjectAudioLifeCycle::Loaded(ref audio) = self.audio {
            audio.samples_interleaved.len() as f32
                / audio.channels.get() as f32
                / audio.sample_rate.get() as f32
        } else {
            0.0
        };
        let max_seconds_from_textgrid =
            if let ProjectTextGridLifeCycle::Loaded(ref textgrid) = self.textgrid {
                textgrid.xmax as f32
            } else {
                0.0
            };

        max_seconds_from_audio.max(max_seconds_from_textgrid)
    }

    fn start_seconds(&self) -> f32 {
        self.offset_points / self.points_per_second
    }
}

impl Project {
    pub fn ui(&mut self, ui: &mut egui::Ui, l: L10N) {
        let preview = ProjectPreview::extract_from_ui(ui);
        if let Some(preview) = &preview
            && preview.is_from_dropping
        {
            if let Some(audio_file_path) = &preview.audio_file_path {
                self.load_audio(audio_file_path);
            }
            if let Some(textgrid_file_path) = &preview.textgrid_file_path {
                self.load_textgrid(textgrid_file_path);
            }
        } else {
            self.update_audio();
            self.update_textgrid();
        }

        self.update_title_name(ui, &l);

        self.header_pane_ui(ui, &preview);

        self.main_pane_ui(ui, &l);
    }

    fn header_pane_ui(&mut self, ui: &mut egui::Ui, preview: &Option<ProjectPreview>) {
        let full_width = ui.available_width();

        egui::Grid::new(ui.next_auto_id()).show(ui, |ui| {
            fn truncate_label(ui: &mut egui::Ui, str: &str) {
                egui::Label::new(str)
                    .wrap_mode(egui::TextWrapMode::Truncate)
                    .ui(ui);
            }

            fn preview_label(ui: &mut egui::Ui, path: &PathBuf, has_already_loaded: bool) {
                let text = if has_already_loaded {
                    format!("<will load as replacement>: {}", path.display())
                } else {
                    format!("<will load>: {}", path.display())
                };

                truncate_label(ui, &text);
            }

            two_cells(
                ui,
                full_width,
                |ui| {
                    ui.label("Audio").rect.width();
                },
                |ui| {
                    if let Some(preview) = preview
                        && let Some(audio_file_path) = &preview.audio_file_path
                    {
                        preview_label(ui, audio_file_path, self.audio_path.is_some());
                    } else if let Some(audio_path) = self.audio_path.clone() {
                        ui.horizontal(|ui| {
                            if ui.button(egui_phosphor::regular::X).clicked() {
                                self.clear_audio();
                            }
                            truncate_label(ui, &audio_path.display().to_string());
                        });
                    } else {
                        ui.label("<absent>");
                    }
                },
            );
            ui.end_row();

            two_cells(
                ui,
                full_width,
                |ui| {
                    ui.label("TextGrid");
                },
                |ui| {
                    if let Some(preview) = &preview
                        && let Some(textgrid_file_path) = &preview.textgrid_file_path
                    {
                        preview_label(ui, textgrid_file_path, self.textgrid_path.is_some());
                    } else if let Some(textgrid_path) = self.textgrid_path.clone() {
                        ui.horizontal(|ui| {
                            if ui.button(egui_phosphor::regular::X).clicked() {
                                self.clear_textgrid();
                            }
                            truncate_label(ui, &textgrid_path.display().to_string());
                        });
                    } else {
                        ui.label("<absent>");
                    }
                },
            );
            ui.end_row();
        });

        let mut start_percent = (self.start_seconds() / self.max_seconds()).min(1.0);
        let old_start_percent = start_percent;
        let seconds_in_view = ui.available_width() / self.points_per_second;
        let size_percent = (seconds_in_view / self.max_seconds()).min(1.0);

        HorizontalScrollBar::new(&mut start_percent, size_percent).ui(ui);

        if old_start_percent != start_percent {
            self.offset_points = start_percent * self.max_seconds() * self.points_per_second;
            if self.offset_points + seconds_in_view * self.points_per_second
                > self.max_seconds() * self.points_per_second
            {
                self.offset_points = self.max_seconds() * self.points_per_second
                    - seconds_in_view * self.points_per_second;
            }
        }
    }

    fn main_pane_ui(&mut self, ui: &mut egui::Ui, l: &L10N) {
        ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();

        egui::ScrollArea::vertical().show(ui, |ui| {
            match &self.audio {
                ProjectAudioLifeCycle::Absent => {}
                ProjectAudioLifeCycle::Loading(_) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(l.tl(&Term::LoadingThing {
                            thing: "audio",
                            path: self.audio_path.as_ref().unwrap().display().to_string(),
                        }));
                    });
                }
                ProjectAudioLifeCycle::Loaded(audio) => {
                    self.waveforms_ui(ui, &audio.clone());
                }
                ProjectAudioLifeCycle::Error(e) => {
                    ui.label(l.tl(&Term::FailedToLoadThing {
                        thing: "audio",
                        path: self.audio_path.as_ref().unwrap().display().to_string(),
                        error: e.clone(),
                    }));
                }
            }

            match self.textgrid {
                ProjectTextGridLifeCycle::Absent => {}
                ProjectTextGridLifeCycle::Loading(_) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(l.tl(&Term::LoadingThing {
                            thing: "TextGrid",
                            path: self.textgrid_path.as_ref().unwrap().display().to_string(),
                        }));
                    });
                }
                ProjectTextGridLifeCycle::Loaded(_) => {
                    ui.label("TODO: TextGrid loaded.");
                }
                ProjectTextGridLifeCycle::Error(ref e) => {
                    ui.label(l.tl(&Term::FailedToLoadThing {
                        thing: "TextGrid",
                        path: self.textgrid_path.as_ref().unwrap().display().to_string(),
                        error: e.clone(),
                    }));
                }
            }
        });
    }

    fn waveforms_ui(&mut self, ui: &mut egui::Ui, audio: &ProjectAudio) {
        const TMP_HEIGHT: f32 = 100.0;

        let wave_data = Arc::new(WaveData {
            id: audio.id,
            sample_rate: audio.sample_rate,
            channels: audio.channels,
            samples_interleaved: audio.samples_interleaved.clone(),
        });

        let size = egui::Vec2::new(ui.available_width(), TMP_HEIGHT);
        let max_seconds = self.max_seconds();

        ui.scope(|ui| {
            ui.style_mut().spacing.item_spacing.y = -1.0;

            for channel in 0..audio.channels.get() {
                egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(egui::Stroke::new(
                        1.0,
                        match ui.theme() {
                            egui::Theme::Dark => egui::Color32::WHITE,
                            egui::Theme::Light => egui::Color32::BLACK,
                        },
                    ))
                    .show(ui, |ui| {
                        HorizontalScrollAndZoomArea::new(
                            &mut self.points_per_second,
                            &mut self.offset_points,
                            max_seconds,
                        )
                        .show(
                            ui,
                            |ui, points_per_second, offset_points| {
                                Waveform::new(size, wave_data.clone(), channel)
                                    .points_per_second(points_per_second)
                                    .offset_points(offset_points)
                                    .ui(ui)
                            },
                        );
                    });
            }
        });
    }
}

fn two_cells(
    ui: &mut egui::Ui,
    full_width: f32,
    left: impl FnOnce(&mut egui::Ui),
    right: impl FnOnce(&mut egui::Ui),
) {
    let left_width = ui
        .horizontal(|ui| {
            left(ui);
        })
        .response
        .rect
        .width();

    // TODO: find out what this magic number is.
    const MAGIC: f32 = 16.0;

    let right_width = full_width - left_width - ui.style().spacing.item_spacing.x - MAGIC;
    egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::exact(right_width))
        .horizontal(|mut strip| {
            strip.cell(|ui| right(ui));
        });
}
