use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    thread,
};

use eframe::egui::{self, Widget as _};
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

    next_task_id: usize,

    audio: ProjectAudioLifeCycle,
    textgrid: ProjectTextGridLifeCycle,
}

enum ProjectAudioLifeCycle {
    Absent,
    Loading {
        path: PathBuf,
        data: Arc<RwLock<LoadingData<ProjectAudio>>>,
    },
    Loaded(ProjectAudio),
    Error(String),
}

pub struct ProjectAudio {}

enum ProjectTextGridLifeCycle {
    Absent,
    Loading {
        path: PathBuf,
        data: Arc<RwLock<LoadingData<TextGrid>>>,
    },
    Loaded(TextGrid),
    Error(String),
}

struct LoadingData<T> {
    data: Option<Result<T, String>>,
    id: usize,
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
            next_task_id: 0,
            audio: ProjectAudioLifeCycle::Absent,
            textgrid: ProjectTextGridLifeCycle::Absent,
        }
    }

    pub fn id(&self) -> uuid::Uuid {
        self.uuid
    }

    fn load_audio(&mut self, path: &PathBuf) {
        let id = self.next_task_id;
        self.next_task_id += 1;

        let data = Arc::new(RwLock::new(LoadingData { data: None, id }));
        self.audio = ProjectAudioLifeCycle::Loading {
            path: path.clone(),
            data: data.clone(),
        };

        thread::spawn({
            let data = data.clone();
            move || {
                // TODO: actually load the audio data.
                let loaded_data = ProjectAudio {};

                let mut loading_data = data
                    .write()
                    .expect("Failed to acquire write lock on loading data.");
                if loading_data.id == id {
                    loading_data.data = Some(Ok(loaded_data));
                }
            }
        });
    }

    fn load_textgrid(&mut self, path: &PathBuf) {
        let id = self.next_task_id;
        self.next_task_id += 1;

        let data = Arc::new(RwLock::new(LoadingData { data: None, id }));
        self.textgrid = ProjectTextGridLifeCycle::Loading {
            path: path.clone(),
            data: data.clone(),
        };

        thread::spawn({
            let data = data.clone();
            let path = path.clone();
            move || {
                let file = std::fs::File::open(&path).map_err(|e| e.to_string());
                let loaded_data = file.and_then(|mut f| {
                    let mut content = Vec::new();
                    f.read_to_end(&mut content).map_err(|e| e.to_string())?;
                    TextGrid::parse_text_format(&content).map_err(|e| e.to_string())
                });

                let mut loading_data = data
                    .write()
                    .expect("Failed to acquire write lock on loading data.");
                if loading_data.id == id {
                    loading_data.data = Some(loaded_data);
                }
            }
        });
    }

    pub fn ui(&self, ui: &mut egui::Ui, l: L10N) {
        ui.label("TODO");
    }
}
