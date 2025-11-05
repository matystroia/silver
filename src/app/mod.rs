use color_eyre::Result;
use ratatui::DefaultTerminal;
use ratatui::widgets::Widget;
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tokio::select;
use tokio::time::sleep;

use crate::app::dispatcher::Dispatcher;
use crate::app::signals::Signals;
use crate::core::Core;
use crate::core::events::NEED_RENDER;
use crate::core::events::event::Event;
use crate::core::manager::Manager;
use crate::traits::menu::Menu;
use crate::ui::home::Home;

mod dispatcher;
mod router;
mod signals;

pub struct App {
    pub manager: Manager,
    pub core: Arc<Core>,
    pub term: DefaultTerminal,
    pub signals: Signals,
}

impl App {
    pub fn bootstrap(&mut self) {
        let home = Home::new(&self.core);
        self.manager.push_menu(Box::new(home));
    }

    pub async fn serve(term: DefaultTerminal) -> Result<()> {
        let (mut rx, signals) = (Event::init(), Signals::start()?);

        let core = Arc::new(Core::make());
        let manager = Manager::default();

        let mut app = App {
            core,
            manager,
            term,
            signals,
        };
        app.bootstrap();

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

        loop {
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
            self.manager.menu().render(frame.area(), frame.buffer_mut());
            self.core.notify.read().unwrap().render(frame);
            self.core.tasks.read().unwrap().render(frame);
        })?;

        Ok(())
    }

    pub fn quit(&mut self) {
        ratatui::restore();
        exit(0)
    }
}
