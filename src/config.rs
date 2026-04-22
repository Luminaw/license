use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub author_name: String,
    pub author_email: Option<String>,
    pub osi_approved_only: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            author_name: "Your Name".into(),
            author_email: None,
            osi_approved_only: true,
        }
    }
}
