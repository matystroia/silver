use std::sync::{Arc, atomic::Ordering};

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;

use crate::{
    app::{App, router::Router},
    core::{
        CoreTasks,
        events::{NEED_RENDER, event::Event, navigate::Navigate},
        notify::message::Message,
    },
    model::feature::Feature,
    traits::menu::Menu,
    ui::{directories::Directories, movies::Movies},
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
            Event::Navigate(nav) => self.dispatch_navigate(nav),
            Event::Key(key) => self.dispatch_key(key),
            Event::Render => self.dispatch_render(),
            Event::Notify(msg) => self.dispatch_notify(msg),
            Event::Scan | Event::RmDir(_) | Event::AddDir(_) => self.dispatch_app(event),
            Event::Quit => self.app.quit(),
            _ => {}
        }
    }

    fn dispatch_navigate(&mut self, nav: Navigate) {
        let next: Option<Box<dyn Menu>> = match nav {
            Navigate::Movies => Some(Box::new(Movies::new(&self.app.core))),
            Navigate::Directories => Some(Box::new(Directories::new(&self.app.core))),
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

    fn dispatch_key(&mut self, key: KeyEvent) {
        Router::new(self.app).route(key);
        self.dispatch_render();
    }

    fn dispatch_app(&mut self, event: Event) {
        let core = &self.app.core;
        match event {
            Event::Scan => {
                core.scan_directories();
                // if let Some(features) = self.app.core.scan_directories().{
                // let mut guard = self.app.core.data.write().unwrap();
                // guard.movies = features
                //     .iter()
                //     .filter_map(|feat| match feat {
                //         Feature::Movie(movie) => Some(movie),
                //         _ => None,
                //     })
                //     .cloned()
                //     .collect();
                // guard.episodes = features
                //     .iter()
                //     .filter_map(|feat| match feat {
                //         Feature::Episode(episode) => Some(episode),
                //         _ => None,
                //     })
                //     .cloned()
                //     .collect();
                // }
            }
            Event::RmDir(dir) => {
                core.data.write().unwrap().rm_dir(&dir).unwrap_or_notify();
            }
            Event::AddDir(dir) => {
                core.data.write().unwrap().add_dir(dir).unwrap_or_notify();
            }
            _ => {}
        };
    }

    fn dispatch_notify(&mut self, msg: Message) {
        let notify = &self.app.core.notify;
        notify.write().unwrap().push_notification(msg);
    }
}

trait UnwrapOrNotify<T> {
    fn unwrap_or_notify(self) -> Option<T>;
}

impl<T> UnwrapOrNotify<T> for Result<T> {
    fn unwrap_or_notify(self) -> Option<T> {
        match self {
            Ok(val) => Some(val),
            Err(err) => {
                Event::Notify(err.to_string().into()).emit();
                None
            }
        }
    }
}
