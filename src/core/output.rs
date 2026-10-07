use ratatui::{backend::IntoCrossterm, text::Line};

use crate::{cli_mode, core::notify::Message, events::Event};

pub struct Output {}

impl Output {
    pub fn notify(message: Message) {
        if cli_mode() {
            print_line(&message.message);
        } else {
            Event::Notify(message).emit();
        }
    }
}

fn print_line(line: &Line) {
    for span in &line.spans {
        let style = line.style.patch(span.style).into_crossterm();
        print!("{}", style.apply(&*span.content));
    }
    println!();
}
