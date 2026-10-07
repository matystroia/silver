pub mod common;
mod directory;
mod home;
mod movie;
mod series;
pub mod util;

pub use directory::DirectoriesMenu;
pub use home::Home;
pub use movie::MoviesMenu;
pub use series::{SeriesList, single_series::SeriesMenu};
