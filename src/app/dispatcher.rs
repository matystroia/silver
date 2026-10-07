use std::sync::{Arc, atomic::Ordering};

use crossterm::event::KeyEvent;

use crate::{
    app::{App, NEED_RENDER, Router},
    core::notify::Message,
    events::{Event, Navigate},
    traits::{Menu, OkOrNotify},
    ui::{DirectoriesMenu, MoviesMenu, SeriesList, SeriesMenu},
};

pub struct Dispatcher<'a> {
    app: &'a mut App,
}

impl<'a> Dispatcher<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self { app }
    }

    pub async fn dispatch(&mut self, event: Event) {
        match event {
            Event::Navigate(nav) => self.dispatch_navigate(nav).await,
            Event::Key(key) => self.dispatch_key(key).await,
            Event::Render => self.dispatch_render(),
            Event::Notify(msg) => self.dispatch_notify(msg),

            Event::Suspend => {
                self.app.suspend().await.ok_or_notify();
            }
            Event::Terminate(sig) => self.app.terminate(sig),
            Event::Quit => self.app.quit(),
        }
    }

    async fn dispatch_navigate(&mut self, nav: Navigate) {
        let core = Arc::clone(&self.app.core);
        let next: Option<Box<dyn Menu>> = match nav {
            Navigate::Movies => Some(Box::new(MoviesMenu::new(core).await)),
            Navigate::SeriesList => Some(Box::new(SeriesList::new(core).await)),
            Navigate::Series(series_id) => Some(Box::new(SeriesMenu::new(core, series_id).await)),
            Navigate::Directories => Some(Box::new(DirectoriesMenu::new(core).await)),
            Navigate::Back => None,
        };

        match next {
            None => self.app.manager.pop_menu(),
            Some(menu) => self.app.manager.push_menu(menu),
        }

        self.dispatch_render();
    }

    fn dispatch_render(&mut self) {
        NEED_RENDER.store(true, Ordering::Relaxed);
    }

    async fn dispatch_key(&mut self, key: KeyEvent) {
        Router::new(self.app).route(key).await;
        self.dispatch_render();
    }

    fn dispatch_notify(&mut self, msg: Message) {
        let notify = &self.app.core.notify;
        notify.write().unwrap().push_notification(msg);

        self.dispatch_render();
    }
}
