pub mod event;
pub mod navigate;

pub static NEED_RENDER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[derive(Default)]
pub struct Events {}

impl Events {}
