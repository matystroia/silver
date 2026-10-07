use std::process::exit;

use crate::traits::Menu;

#[derive(Default)]
pub(super) struct Manager {
    menu_stack: Vec<Box<dyn Menu>>,
}

impl Manager {
    pub fn menu(&self) -> &dyn Menu {
        &**self.menu_stack.last().unwrap()
    }

    pub fn menu_mut(&mut self) -> &mut dyn Menu {
        &mut **self.menu_stack.last_mut().unwrap()
    }

    pub fn push_menu(&mut self, menu: Box<dyn Menu>) {
        self.menu_stack.push(menu);
    }

    pub fn pop_menu(&mut self) {
        if self.menu_stack.len() > 1 {
            self.menu_stack.pop();
        } else {
            exit(0)
        }
    }

    pub fn has_modal(&self) -> bool {
        self.menu().has_modal()
    }
}
