use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use eframe::egui::{self, Widget as _};

use gridder_egui::{
    horizontal_scroll_and_zoom_area::HorizontalScrollAndZoomArea,
    horizontal_scroll_bar::HorizontalScrollBar,
    view_range::ViewRange,
    waveform::{WaveData, Waveform},
};

use crate::{
    l10n::{L10N, Term},
    states::projects::{
        ProjectAudio, ProjectAudioLifeCycle, ProjectPreviewState, ProjectState,
        ProjectTextGridLifeCycle,
    },
    views::utils::two_cells_h,
};

pub struct OpenedProject {
    uuid: uuid::Uuid,
    window: Mutex<ProjectWindow>,
}

pub struct OpenedProjects(Vec<Arc<OpenedProject>>);

impl OpenedProjects {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, project: ProjectWindow) {
        self.0.push(Arc::new(OpenedProject {
            uuid: *project.id(),
            window: Mutex::new(project),
        }));
    }

    pub fn remove(&mut self, uuid: &uuid::Uuid) {
        self.0.retain(|p| p.uuid != *uuid);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<OpenedProject>> {
        self.0.iter()
    }
}

pub struct ProjectWindow {
    l: L10N,

    state: ProjectState,

    last_frame_title_name: Option<Option<String>>,
    last_waveform_width: Option<f32>,

    view_range: ViewRange,
}

impl ProjectWindow {
    pub fn try_from_preview_from_dropping_files(
        l10n: L10N,
        preview: &ProjectPreviewState,
    ) -> Option<Self> {
        if !preview.is_from_dropping() {
            return None;
        }

        let mut zelf = Self::new(l10n);

        if let Some(audio_file_path) = preview.audio_file_path() {
            zelf.state.load_audio(audio_file_path);
        }
        if let Some(textgrid_file_path) = preview.textgrid_file_path() {
            zelf.state.load_textgrid(textgrid_file_path);
        }

        Some(zelf)
    }

    pub fn id(&self) -> &uuid::Uuid {
        self.state.uuid()
    }

    fn new(l: L10N) -> Self {
        Self {
            l,
            state: ProjectState::new(uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext))),
            last_frame_title_name: None,
            last_waveform_width: None,
            view_range: ViewRange::default(),
        }
    }

    fn title_name(&self) -> Option<String> {
        if let Some(audio_file_path) = self.state.audio_path() {
            Some(audio_file_path.display().to_string())
        } else if let Some(textgrid_file_path) = self.state.textgrid_path() {
            Some(textgrid_file_path.display().to_string())
        } else {
            None
        }
    }

    fn update_title_name(&mut self, ui: &mut egui::Ui) {
        let new_title_name = self.title_name();
        if self
            .last_frame_title_name
            .as_ref()
            .is_none_or(|old_title_name| old_title_name != &new_title_name)
        {
            ui.send_viewport_cmd(egui::ViewportCommand::Title(self.l.tl(
                &Term::GridderProject {
                    name: new_title_name.clone(),
                },
            )));
            self.last_frame_title_name = Some(new_title_name);
        }
    }

    fn reset_scroll_and_zoom(&mut self) {
        self.view_range = ViewRange::default();
    }
}

impl ProjectWindow {
    pub fn window(
        ui: &mut egui::Ui,
        l: L10N,
        project: Arc<OpenedProject>,
        projects: Arc<RwLock<OpenedProjects>>,
    ) {
        ui.ctx().show_viewport_deferred(
            egui::ViewportId::from_hash_of(project.uuid),
            egui::ViewportBuilder::default().with_title(l.tl(&Term::GridderProject { name: None })),
            move |ui, class| {
                let mut project = project
                    .window
                    .lock()
                    .expect("Failed to acquire lock on project.");
                if class == egui::ViewportClass::EmbeddedWindow {
                    // currently, the project UI is based on the
                    // assumption that it has its own viewport.
                    unimplemented!("Embedded viewports are not supported yet.");
                    // project.ui(ui);
                } else {
                    egui::CentralPanel::default().show(ui, |ui| {
                        if ui.input(|i| i.viewport().close_requested()) {
                            projects
                                .write()
                                .expect("Failed to acquire write lock on projects.")
                                .remove(project.id());
                            return;
                        }

                        project.ui(ui);
                    });
                }
            },
        );
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let mut is_updated = false;

        let preview = ProjectPreviewState::extract_from_ui(ui);
        if let Some(preview) = &preview
            && preview.is_from_dropping()
        {
            if let Some(audio_file_path) = preview.audio_file_path() {
                self.state.load_audio(audio_file_path);
            }
            if let Some(textgrid_file_path) = preview.textgrid_file_path() {
                self.state.load_textgrid(textgrid_file_path);
            }
        } else {
            is_updated |= self.state.update_audio();
            is_updated |= self.state.update_textgrid();
        }

        if is_updated {
            self.reset_scroll_and_zoom();
        }

        self.update_title_name(ui);

        self.header_pane_ui(ui, &preview);

        self.main_pane_ui(ui);
    }

    fn header_pane_ui(&mut self, ui: &mut egui::Ui, preview: &Option<ProjectPreviewState>) {
        let full_width = ui.available_width();

        egui::Grid::new(ui.next_auto_id()).show(ui, |ui| {
            fn truncate_label(ui: &mut egui::Ui, str: &str) {
                egui::Label::new(str)
                    .wrap_mode(egui::TextWrapMode::Truncate)
                    .ui(ui);
            }

            fn preview_label(ui: &mut egui::Ui, path: impl AsRef<Path>, has_already_loaded: bool) {
                let text = if has_already_loaded {
                    format!("<will load as replacement>: {}", path.as_ref().display())
                } else {
                    format!("<will load>: {}", path.as_ref().display())
                };

                truncate_label(ui, &text);
            }

            two_cells_h(
                ui,
                full_width,
                |ui| {
                    ui.label("Audio").rect.width();
                },
                |ui| {
                    if let Some(preview) = preview
                        && let Some(audio_file_path) = preview.audio_file_path()
                    {
                        preview_label(ui, audio_file_path, self.state.audio_path().is_some());
                    } else if let Some(audio_path) = self.state.audio_path().map(PathBuf::from) {
                        ui.horizontal(|ui| {
                            if ui.button(egui_phosphor::regular::X).clicked() {
                                self.state.clear_audio();
                            }
                            truncate_label(ui, &audio_path.display().to_string());
                        });
                    } else {
                        ui.label("<absent>");
                    }
                },
            );
            ui.end_row();

            two_cells_h(
                ui,
                full_width,
                |ui| {
                    ui.label("TextGrid");
                },
                |ui| {
                    if let Some(preview) = &preview
                        && let Some(textgrid_file_path) = preview.textgrid_file_path()
                    {
                        preview_label(ui, textgrid_file_path, self.state.textgrid_path().is_some());
                    } else if let Some(textgrid_path) =
                        self.state.textgrid_path().map(PathBuf::from)
                    {
                        ui.horizontal(|ui| {
                            if ui.button(egui_phosphor::regular::X).clicked() {
                                self.state.clear_textgrid();
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

        HorizontalScrollBar::new(&mut self.view_range).ui(ui);
    }

    fn main_pane_ui(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();

        egui::ScrollArea::vertical().show(ui, |ui| {
            match &self.state.audio() {
                ProjectAudioLifeCycle::Absent => {}
                ProjectAudioLifeCycle::Loading(_) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            self.l.tl(&Term::LoadingThing {
                                thing: "audio",
                                path: self
                                    .state
                                    .audio_path()
                                    .as_ref()
                                    .unwrap()
                                    .display()
                                    .to_string(),
                            }),
                        );
                    });
                }
                ProjectAudioLifeCycle::Loaded(audio) => {
                    self.waveforms_ui(ui, &audio.clone());
                }
                ProjectAudioLifeCycle::Error(e) => {
                    ui.label(
                        self.l.tl(&Term::FailedToLoadThing {
                            thing: "audio",
                            path: self
                                .state
                                .audio_path()
                                .as_ref()
                                .unwrap()
                                .display()
                                .to_string(),
                            error: e.clone(),
                        }),
                    );
                }
            }

            match self.state.textgrid() {
                ProjectTextGridLifeCycle::Absent => {}
                ProjectTextGridLifeCycle::Loading(_) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            self.l.tl(&Term::LoadingThing {
                                thing: "TextGrid",
                                path: self
                                    .state
                                    .textgrid_path()
                                    .as_ref()
                                    .unwrap()
                                    .display()
                                    .to_string(),
                            }),
                        );
                    });
                }
                ProjectTextGridLifeCycle::Loaded(_) => {
                    ui.label("TODO: TextGrid loaded.");
                }
                ProjectTextGridLifeCycle::Error(e) => {
                    ui.label(
                        self.l.tl(&Term::FailedToLoadThing {
                            thing: "TextGrid",
                            path: self
                                .state
                                .textgrid_path()
                                .as_ref()
                                .unwrap()
                                .display()
                                .to_string(),
                            error: e.clone(),
                        }),
                    );
                }
            }
        });
    }

    fn waveforms_ui(&mut self, ui: &mut egui::Ui, audio: &ProjectAudio) {
        const TMP_HEIGHT: f32 = 100.0;

        let wave_data = Arc::new(WaveData {
            id: audio.id(),
            sample_rate: audio.sample_rate(),
            channels: audio.channels(),
            samples_interleaved: audio.samples_interleaved().clone(),
        });

        ui.scope(|ui| {
            #[derive(Clone)]
            struct Cache {
                width: f32,
                points_per_second: f64,
                offset_points: f64,
            }
            let mut cache: Option<Cache> = None;

            let border_stroke = egui::Stroke::new(
                1.0,
                match ui.theme() {
                    egui::Theme::Dark => egui::Color32::WHITE,
                    egui::Theme::Light => egui::Color32::BLACK,
                },
            );
            ui.style_mut().spacing.item_spacing.y = -border_stroke.width;

            for channel in 0..audio.channels().get() {
                egui::Frame::new()
                    .fill(ui.visuals().extreme_bg_color)
                    .stroke(egui::Stroke::new(
                        border_stroke.width,
                        egui::Color32::TRANSPARENT,
                    ))
                    .show(ui, |ui| {
                        let Cache {
                            width,
                            points_per_second,
                            offset_points,
                        } = if let Some(cache) = &cache {
                            cache.clone()
                        } else {
                            let width = ui.available_width();
                            if let Some(last_waveform_width) = self.last_waveform_width
                                && (last_waveform_width - width).abs() > f32::EPSILON
                            {
                                self.view_range
                                    .anti_stretch_after_resize(width / last_waveform_width);
                            }
                            self.last_waveform_width = Some(width);

                            let length_in_seconds = self.state.length_in_seconds();
                            let points_per_second =
                                (width as f64 / self.view_range.view_ratio()) / length_in_seconds;
                            let offset_points = self
                                .view_range
                                .start_points(length_in_seconds * points_per_second);

                            let some_cache = Cache {
                                width,
                                points_per_second,
                                offset_points,
                            };
                            cache = Some(some_cache.clone());
                            some_cache
                        };
                        let size = egui::vec2(width, TMP_HEIGHT);

                        draw_borders_in_frame(
                            ui,
                            &self.view_range,
                            egui::Rect::from_min_size(ui.min_rect().min, size),
                            border_stroke,
                        );

                        HorizontalScrollAndZoomArea::new(&mut self.view_range).show(
                            ui,
                            |ui, _view_range| {
                                Waveform::new(size, wave_data.clone(), channel)
                                    .points_per_second(points_per_second as f32)
                                    .offset_points(offset_points as f32)
                                    .color(match ui.theme() {
                                        egui::Theme::Dark => egui::Color32::LIGHT_GRAY,
                                        egui::Theme::Light => egui::Color32::DARK_GRAY,
                                    })
                                    .center_line_color(Some(
                                        egui::Color32::GRAY.linear_multiply(0.5),
                                    ))
                                    .ui(ui)
                            },
                        );
                    });
            }
        });
    }
}

fn draw_borders_in_frame(
    ui: &mut egui::Ui,
    view_range: &ViewRange,
    inner_rect: egui::Rect,
    stroke: egui::Stroke,
) {
    let stroke_more = egui::Stroke::new(stroke.width, stroke.color.linear_multiply(0.5));
    let width = stroke.width;

    let painter = ui.painter();
    painter.line_segment(
        [
            egui::pos2(inner_rect.left() - width, inner_rect.top() - width),
            egui::pos2(inner_rect.right() + width, inner_rect.top() - width),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(inner_rect.left() - width, inner_rect.bottom() + width),
            egui::pos2(inner_rect.right() + width, inner_rect.bottom() + width),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(inner_rect.left(), inner_rect.top() - width),
            egui::pos2(inner_rect.left(), inner_rect.bottom() + width),
        ],
        if view_range.start_ratio() < f64::EPSILON {
            stroke
        } else {
            stroke_more
        },
    );
    painter.line_segment(
        [
            egui::pos2(inner_rect.right(), inner_rect.top() - width),
            egui::pos2(inner_rect.right(), inner_rect.bottom() + width),
        ],
        if 1.0 - view_range.end_ratio() < f64::EPSILON {
            stroke
        } else {
            stroke_more
        },
    );
}
