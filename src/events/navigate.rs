use crate::api::TMDBId;

#[derive(Hash, PartialEq, Eq)]
pub enum Navigate {
    Movies,
    SeriesList,
    Series(TMDBId),
    Directories,
    Back,
}
