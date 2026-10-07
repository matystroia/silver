use color_eyre::eyre::Result;
use crossterm::event::{
    Event as CrosstermEvent, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use futures::StreamExt;
use tokio::{
    select,
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
        oneshot,
    },
};

use crate::events::Event;

pub struct Signals {
    tx: UnboundedSender<(bool, Option<oneshot::Sender<()>>)>,
}

impl Signals {
    pub fn start() -> Result<Self> {
        let (tx, rx) = unbounded_channel();
        Self::spawn(rx)?;

        Ok(Self { tx })
    }

    pub fn stop(&mut self, cb: Option<oneshot::Sender<()>>) -> Result<()> {
        Ok(self.tx.send((false, cb))?)
    }

    pub fn resume(&mut self, cb: Option<oneshot::Sender<()>>) -> Result<()> {
        Ok(self.tx.send((true, cb))?)
    }

    fn handle_sys(n: libc::c_int) -> bool {
        use libc::{SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGTSTP};
        match n {
            SIGINT => {}
            SIGQUIT | SIGHUP | SIGTERM => {
                Event::Terminate(n).emit();
            }
            SIGTSTP => {
                Event::Suspend.emit();
            }
            _ => {}
        }
        true
    }

    fn handle_term(event: CrosstermEvent) {
        match event {
            CrosstermEvent::Key(KeyEvent {
                code: KeyCode::Char('z'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) => Event::Suspend.emit(),
            CrosstermEvent::Key(
                key @ KeyEvent {
                    kind: KeyEventKind::Press,
                    ..
                },
            ) => Event::Key(key).emit(),
            CrosstermEvent::Resize(..) => Event::Render.emit(),
            _ => {}
        }
    }

    fn spawn(mut rx: UnboundedReceiver<(bool, Option<oneshot::Sender<()>>)>) -> Result<()> {
        use libc::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGTSTP};
        use signal_hook_tokio::Signals;

        let mut sys = Signals::new([SIGINT, SIGQUIT, SIGHUP, SIGTERM, SIGTSTP, SIGCONT])?;
        let mut term = Some(EventStream::new());

        tokio::spawn(async move {
            loop {
                if let Some(t) = &mut term {
                    select! {
                        biased;
                        Some((state, mut callback)) = rx.recv() => {
                            term = term.filter(|_| state);
                            callback.take().map(|cb|cb.send(()));
                        },
                        Some(n) = sys.next() => if !Self::handle_sys(n) { return },
                        Some(Ok(e)) = t.next() => Self::handle_term(e),
                    }
                } else {
                    select! {
                        biased;
                        Some((state, mut callback)) = rx.recv() => {
                            term = state.then(EventStream::new);
                            callback.take().map(|cb| cb.send(()));
                        },
                        Some(n) = sys.next() => if !Self::handle_sys(n) {return}
                    }
                }
            }
        });

        Ok(())
    }
}
