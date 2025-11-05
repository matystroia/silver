use crate::model::{feature::Feature, file::FileData};

pub struct Video {
    file: FileData,
    feature: Option<Feature>,
}

impl Video {
    pub fn play(&self) {}
}
