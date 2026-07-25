use crate::l10n::{Language, Term};

pub struct ChineseMandarinSimplified;

impl Language for ChineseMandarinSimplified {
    fn code(&self) -> &'static str {
        "cmn-Hans"
    }

    fn display_name(&self) -> &'static str {
        "中文（汉语官话・简体）"
    }

    fn tl(&self, term: &Term) -> String {
        use Term::*;

        match term {
            About => "关于".to_string(),
            Settings => "设置".to_string(),
            Appearance => "外观".to_string(),
            DropHintText {
                supported_audio_extensions,
            } => {
                let supported_text = supported_audio_extensions
                    .iter()
                    .map(|ext| format!(".{}", ext))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "拖入一份音频文件（{}）和/或 TextGrid 文件到此处。",
                    supported_text
                )
            }
            ProjectPreviewText {
                has_audio,
                has_textgrid,
            } => {
                let desc = match (has_audio, has_textgrid) {
                    (true, true) => "一份音频文件和一份 TextGrid 文件",
                    (true, false) => "一份音频文件",
                    (false, true) => "一份 TextGrid 文件",
                    (false, false) => {
                        return "将打开一份空项目。".to_string();
                    }
                };

                format!("将打开一份以{}构成的项目。", desc)
            }
            LoadingThing { thing, path } => format!("正在加载「{}」：{}", thing, path),
            FailedToLoadThing { thing, path, error } => {
                format!("加载「{}」失败：{}\n{}", thing, path, error)
            }
            GridderProject { name } => match name {
                Some(name) => format!("Gridder 项目 - {}", name),
                None => "Gridder 项目".to_owned(),
            },
            Theme => "主题".to_owned(),
        }
    }
}
