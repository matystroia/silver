use std::sync::Arc;

use ratatui::widgets::Row;

#[derive(Clone, Copy, strum::Display)]
pub enum Sort {
    Title,
    Year,
}

/// An item that can be listed in a filterable, sortable [`View`].
pub trait Listable {
    fn title(&self) -> &str;
    fn year(&self) -> Option<i32>;
    fn row(&self) -> Row<'static>;

    /// Whether the item matches a (case-insensitive) filter needle.
    fn matches(&self, needle: &str) -> bool {
        self.title().to_lowercase().contains(&needle.to_lowercase())
    }
}

/// Holds a backing list of items plus the sorted/filtered view derived from it.
pub struct View<T> {
    all: Vec<Arc<T>>,
    filtered: Vec<Arc<T>>,
    filter: String,
    sort: Sort,
    reverse: bool,
}

impl<T: Listable> View<T> {
    pub fn new(items: Vec<Arc<T>>) -> Self {
        let mut ret = Self {
            all: items.clone(),
            filtered: items,
            filter: String::new(),
            sort: Sort::Title,
            reverse: false,
        };
        ret.apply();
        ret
    }

    pub fn rows(&self) -> Vec<Row<'static>> {
        self.filtered.iter().map(|item| item.row()).collect()
    }

    pub fn get(&self, i: usize) -> Option<Arc<T>> {
        self.filtered.get(i).cloned()
    }

    /// Re-sort the backing list and rebuild the filtered view from it.
    fn apply(&mut self) {
        match self.sort {
            Sort::Title => self.all.sort_by(|a, b| a.title().cmp(b.title())),
            Sort::Year => self.all.sort_by_key(|item| item.year()),
        }
        if self.reverse {
            self.all.reverse();
        }

        self.filtered = if self.filter.is_empty() {
            self.all.clone()
        } else {
            self.all
                .iter()
                .filter(|item| item.matches(&self.filter))
                .cloned()
                .collect()
        };
    }

    pub fn set_items(&mut self, items: Vec<Arc<T>>) {
        self.all = items;
        self.apply();
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.apply();
    }

    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Cycle through ascending/descending for each sort key in turn.
    pub fn cycle_sort(&mut self) {
        match (self.sort, self.reverse) {
            (_, false) => self.reverse = true,
            (sort, true) => {
                self.sort = match sort {
                    Sort::Title => Sort::Year,
                    Sort::Year => Sort::Title,
                };
                self.reverse = false;
            }
        }
        self.apply();
    }

    pub fn filter_display(&self) -> String {
        if self.filter.is_empty() {
            String::new()
        } else {
            format!("Filter: {}", self.filter)
        }
    }

    pub fn sort_display(&self) -> String {
        format!(
            "Sort: {} {}",
            self.sort,
            if self.reverse { "↓" } else { "↑" }
        )
    }
}
