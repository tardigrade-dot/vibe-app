pub mod commands;
pub mod cus_tts;
use once_cell::sync::OnceCell;
use std::path::PathBuf;

pub static MODEL_BASE_PATH: OnceCell<PathBuf> = OnceCell::new();