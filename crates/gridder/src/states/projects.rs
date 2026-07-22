use std::{
    io::Read as _,
    num::{NonZeroU16, NonZeroU32},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread,
};

use eframe::egui;
use rodio::Source as _;
use sha2::Digest as _;

use textgrid_rs::TextGrid;

pub static SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &["wav", "mp3"];

pub struct ProjectPreviewState {
    audio_file_path: Option<PathBuf>,
    textgrid_file_path: Option<PathBuf>,

    is_from_dropping: bool,
}

struct HoveredOrDroppedFile<'a> {
    path: Option<&'a Path>,
}

impl ProjectPreviewState {
    pub fn audio_file_path(&self) -> Option<&Path> {
        self.audio_file_path.as_deref()
    }
    pub fn textgrid_file_path(&self) -> Option<&Path> {
        self.textgrid_file_path.as_deref()
    }
    pub fn is_from_dropping(&self) -> bool {
        self.is_from_dropping
    }

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
    pub fn extract_from_ui(ui: &mut egui::Ui) -> Option<ProjectPreviewState> {
        if true || ui.response().contains_pointer() {
            ui.input(|input| {
                ProjectPreviewState::try_from_dropped_files(input.raw.dropped_files.iter()).or_else(
                    || ProjectPreviewState::try_from_hovered_files(input.raw.hovered_files.iter()),
                )
            })
        } else {
            None
        }
    }

    fn try_from_files<'a>(
        files: impl Iterator<Item = HoveredOrDroppedFile<'a>>,
        is_from_dropping: bool,
    ) -> Option<ProjectPreviewState> {
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
}

pub struct ProjectState {
    uuid: uuid::Uuid,

    audio_path: Option<PathBuf>,
    audio: ProjectAudioLifeCycle,
    textgrid_path: Option<PathBuf>,
    textgrid: ProjectTextGridLifeCycle,
}

impl ProjectState {
    pub fn uuid(&self) -> &uuid::Uuid {
        &self.uuid
    }
    pub fn audio_path(&self) -> Option<&Path> {
        self.audio_path.as_deref()
    }
    pub fn audio(&self) -> &ProjectAudioLifeCycle {
        &self.audio
    }
    pub fn textgrid_path(&self) -> Option<&Path> {
        self.textgrid_path.as_deref()
    }
    pub fn textgrid(&self) -> &ProjectTextGridLifeCycle {
        &self.textgrid
    }

    pub fn new(uuid: uuid::Uuid) -> Self {
        Self {
            uuid,
            audio_path: None,
            audio: ProjectAudioLifeCycle::Absent,
            textgrid_path: None,
            textgrid: ProjectTextGridLifeCycle::Absent,
        }
    }

    pub fn load_audio(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();

        let (tx, rx) = mpsc::channel();
        self.audio_path = Some(path.clone());
        self.audio = ProjectAudioLifeCycle::Loading(Mutex::new(rx));

        thread::spawn({
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

                    Ok(ProjectAudio::new(
                        id,
                        sample_rate,
                        channels,
                        Arc::from(samples_interleaved),
                    ))
                });

                tx.send(loaded_data).ok();
            }
        });
    }

    pub fn load_textgrid(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();

        let (tx, rx) = mpsc::channel();
        self.textgrid_path = Some(path.clone());
        self.textgrid = ProjectTextGridLifeCycle::Loading(Mutex::new(rx));

        thread::spawn({
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

    pub fn clear_audio(&mut self) {
        self.audio_path = None;
        self.audio = ProjectAudioLifeCycle::Absent;
    }

    pub fn clear_textgrid(&mut self) {
        self.textgrid_path = None;
        self.textgrid = ProjectTextGridLifeCycle::Absent;
    }

    pub fn update_audio(&mut self) -> bool {
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
            true
        } else {
            false
        }
    }

    pub fn update_textgrid(&mut self) -> bool {
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
            true
        } else {
            false
        }
    }

    pub fn length_in_seconds(&self) -> f64 {
        let audio_length = if let ProjectAudioLifeCycle::Loaded(ref audio) = self.audio {
            audio.length_in_seconds()
        } else {
            0.0
        };
        let textgrid_length = if let ProjectTextGridLifeCycle::Loaded(ref textgrid) = self.textgrid
        {
            textgrid.xmax
        } else {
            0.0
        };

        audio_length.max(textgrid_length)
    }
}

pub enum ProjectAudioLifeCycle {
    Absent,
    Loading(Mutex<mpsc::Receiver<Result<ProjectAudio, String>>>),
    Loaded(ProjectAudio),
    Error(String),
}

#[derive(Clone)]
pub struct ProjectAudio {
    id: egui::Id,
    sample_rate: NonZeroU32,
    channels: NonZeroU16,
    samples_interleaved: Arc<[f32]>,
}

impl ProjectAudio {
    pub fn id(&self) -> egui::Id {
        self.id
    }
    pub fn sample_rate(&self) -> NonZeroU32 {
        self.sample_rate
    }
    pub fn channels(&self) -> NonZeroU16 {
        self.channels
    }
    pub fn samples_interleaved(&self) -> &Arc<[f32]> {
        &self.samples_interleaved
    }

    pub fn new(
        id: egui::Id,
        sample_rate: NonZeroU32,
        channels: NonZeroU16,
        samples_interleaved: Arc<[f32]>,
    ) -> Self {
        Self {
            id,
            sample_rate,
            channels,
            samples_interleaved,
        }
    }
}

impl ProjectAudio {
    pub fn length_in_seconds(&self) -> f64 {
        self.samples_interleaved.len() as f64
            / self.channels.get() as f64
            / self.sample_rate.get() as f64
    }
}

pub enum ProjectTextGridLifeCycle {
    Absent,
    Loading(Mutex<mpsc::Receiver<Result<TextGrid, String>>>),
    Loaded(TextGrid),
    Error(String),
}
