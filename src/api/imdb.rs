use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct IMDBId(pub u64);
