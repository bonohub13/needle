use super::Text;
use serde::Deserialize;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct FpsConfig {
    pub enable: bool,
    pub text: Text,
}
