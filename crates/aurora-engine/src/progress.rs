/// Progress event emitted during pipeline execution (pipeline.md §5).
#[derive(Clone, Debug, PartialEq)]
pub struct StageProgress {
    pub stage_name: String,
    pub stage_index: u8,
    pub total_stages: u8,
    pub percent: f32,
    pub message: String,
}

impl StageProgress {
    pub fn new(
        stage_name: impl Into<String>,
        stage_index: u8,
        total_stages: u8,
        percent: f32,
        message: impl Into<String>,
    ) -> Self {
        Self {
            stage_name: stage_name.into(),
            stage_index,
            total_stages,
            percent: percent.clamp(0.0, 1.0),
            message: message.into(),
        }
    }
}

pub type ProgressCallback = Box<dyn Fn(StageProgress) + Send + Sync>;

pub fn report(
    callback: &Option<&ProgressCallback>,
    stage_name: &str,
    stage_index: u8,
    total_stages: u8,
    percent: f32,
    message: impl Into<String>,
) {
    if let Some(cb) = callback {
        cb(StageProgress::new(
            stage_name,
            stage_index,
            total_stages,
            percent,
            message,
        ));
    }
}
