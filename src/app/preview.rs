use crate::{
    app::{Action, App},
    preview::{ContentSize, Sample, custom::CustomPreview},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewField {
    Sample,
    Content,
}

#[derive(Debug, Clone, Copy)]
pub struct PreviewSelection {
    pub sample: Sample,
    pub content: ContentSize,
    pub field: PreviewField,
}

impl App {
    pub fn use_custom_preview(&mut self, preview: CustomPreview) {
        self.custom_preview = Some(preview);
        self.preview_sample = Sample::Custom;
    }

    pub fn select_sample(&mut self, sample: Sample, content: ContentSize) {
        self.preview_sample = sample;
        if let Sample::BuiltIn(preset) = sample {
            self.preview_content = content;
            if !self.read_only {
                self.session.preset = preset;
            }
            self.record_session(false);
        }
    }

    pub fn open_preview(&mut self) {
        self.preview_selection = Some(PreviewSelection {
            sample: self.preview_sample,
            content: self.preview_content,
            field: PreviewField::Sample,
        });
    }

    pub fn update_preview(&mut self, action: Action) {
        let selection = self
            .preview_selection
            .as_mut()
            .expect("preview selector is open");
        match action {
            Action::FieldNext | Action::FieldPrevious if selection.sample != Sample::Custom => {
                selection.field = match selection.field {
                    PreviewField::Sample => PreviewField::Content,
                    PreviewField::Content => PreviewField::Sample,
                };
            }
            Action::ValueNext | Action::ValuePrevious => match selection.field {
                PreviewField::Sample => {
                    selection.sample = selection
                        .sample
                        .adjacent(action == Action::ValueNext, self.custom_preview.is_some());
                }
                PreviewField::Content => {
                    let index = ContentSize::ALL
                        .iter()
                        .position(|content| *content == selection.content)
                        .unwrap();
                    let length = ContentSize::ALL.len();
                    let offset = if action == Action::ValueNext {
                        1
                    } else {
                        length - 1
                    };
                    selection.content = ContentSize::ALL[(index + offset) % length];
                }
            },
            Action::Confirm => {
                let selection = self.preview_selection.take().unwrap();
                if self.preview_sample != selection.sample
                    || self.preview_content != selection.content
                {
                    self.select_sample(selection.sample, selection.content);
                }
            }
            Action::Cancel => self.preview_selection = None,
            _ => {}
        }
    }
}
