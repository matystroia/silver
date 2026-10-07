use std::sync::{Arc, Mutex};

use image::{DynamicImage, RgbImage};
use ratatui::{
    prelude::*,
    widgets::{StatefulWidget, WidgetRef},
};
use ratatui_image::{
    FilterType, Resize, StatefulImage,
    thread::{ResizeRequest, ThreadProtocol},
};
use tokio::sync::mpsc;

use crate::{PICKER, events::Event, traits::HeapSize};

pub struct Image {
    protocol: Arc<Mutex<ThreadProtocol>>,
    byte_size: usize,
    _worker: tokio::task::JoinHandle<()>,
}

impl Image {
    pub fn new(image: impl Into<RgbImage>) -> Self {
        let dyn_img = DynamicImage::ImageRgb8(image.into());
        let byte_size = dyn_img.as_bytes().len();

        let inner = PICKER.get().unwrap().new_resize_protocol(dyn_img);

        let (tx, mut rx) = mpsc::unbounded_channel::<ResizeRequest>();
        let protocol = Arc::new(Mutex::new(ThreadProtocol::new(tx, Some(inner))));
        let protocol_ref = Arc::clone(&protocol);

        let worker = tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                if let Ok(response) = request.resize_encode() {
                    protocol_ref
                        .lock()
                        .unwrap()
                        .update_resized_protocol(response);
                    Event::Render.emit();
                }
            }
        });

        Self {
            protocol,
            byte_size,
            _worker: worker,
        }
    }
}

impl WidgetRef for Image {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        StatefulImage::default()
            .resize(Resize::Scale(Some(FilterType::CatmullRom)))
            .render(area, buf, &mut *self.protocol.lock().unwrap());
    }
}

impl HeapSize for Image {
    fn byte_size(&self) -> usize {
        self.byte_size
    }
}
