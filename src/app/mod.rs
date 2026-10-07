use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use color_eyre::Result;
use ratatui::{DefaultTerminal, layout::Size, widgets::WidgetRef};
use tokio::{select, sync::oneshot, time::sleep};

use crate::{
    core::Core,
    events::Event,
    ui::{Home, common::KeymapHelp},
};

mod dispatcher;
mod manager;
mod router;
mod signals;

pub use dispatcher::Dispatcher;
use manager::Manager;
pub use router::Router;
pub use signals::Signals;

pub static NEED_RENDER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub struct App {
    manager: Manager,
    core: Arc<Core>,
    term: DefaultTerminal,
    signals: Signals,
    should_exit: bool,
    showing_help: bool,
}

impl App {
    pub fn bootstrap(&mut self) {
        let home = Home::new(&self.core);
        self.manager.push_menu(Box::new(home));
    }

    pub async fn serve(term: DefaultTerminal) -> Result<()> {
        let (mut rx, signals) = (Event::init(), Signals::start()?);

        let core = Arc::new(Core::make().await);
        let manager = Manager::default();

        let mut app = App {
            core,
            manager,
            term,
            signals,
            should_exit: false,
            showing_help: false,
        };

        app.bootstrap();
        Event::Render.emit();

        let mut events = Vec::with_capacity(50);
        let (mut timeout, mut last_render) = (None, Instant::now());
        macro_rules! drain_events {
            () => {
                for event in events.drain(..) {
                    Dispatcher::new(&mut app).dispatch(event).await;
                    if !NEED_RENDER.load(Ordering::Relaxed) {
                        continue;
                    }

                    timeout = Duration::from_millis(10).checked_sub(last_render.elapsed());
                    if timeout.is_none() {
                        app.render()?;
                        last_render = Instant::now();
                    }
                }
            };
        }

        while !app.should_exit {
            if let Some(t) = timeout.take() {
                select! {
                    _ = sleep(t) => {
                        app.render()?;
                        last_render = Instant::now();
                    }
                    n = rx.recv_many(&mut events, 50) => {
                            if n == 0 {break}
                            drain_events!();
                        }
                }
            } else if rx.recv_many(&mut events, 50).await != 0 {
                drain_events!();
            } else {
                break;
            }
        }

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        NEED_RENDER.store(false, Ordering::Relaxed);

        self.term.draw(|frame| {
            let mut area = frame.area();

            let tasks = self.core.tasks.read().unwrap();
            tasks.render(area, frame.buffer_mut());

            let active_tasks = tasks.active_len();
            if active_tasks > 0 {
                area = area.resize(Size::new(area.width, area.height - 3 * active_tasks as u16));
            }

            self.manager.menu().render_ref(area, frame.buffer_mut());

            let notify = self.core.notify.read().unwrap();
            notify.render(area, frame.buffer_mut());

            if self.showing_help {
                KeymapHelp::new(self.manager.menu().keymaps())
                    .render_ref(frame.area(), frame.buffer_mut());
            }
        })?;

        Ok(())
    }

    pub async fn suspend(&mut self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.signals.stop(Some(tx))?;
        let _ = rx.await;

        ratatui::restore();
        signal_hook::low_level::emulate_default_handler(libc::SIGTSTP)?;
        self.term = ratatui::init();

        self.signals.resume(None)?;
        Event::Render.emit();
        Ok(())
    }

    pub fn terminate(&mut self, sig: i32) {
        ratatui::restore();
        let _ = signal_hook::low_level::emulate_default_handler(sig);
        self.should_exit = true;
    }

    pub fn quit(&mut self) {
        self.should_exit = true;
    }
}
