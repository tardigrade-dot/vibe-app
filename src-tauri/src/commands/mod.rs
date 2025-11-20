use tauri::command;
use anyhow::Result;
use tauri::State;

use crate::cus_tts::example_onnx::default_tts_inference;
#[command]
pub fn add_method(a: i32, b: i32) -> i32 {
    a + b
}

pub type CommandResult<T> = std::result::Result<T, String>;

#[command]
pub fn generate_voice(text: &str) -> CommandResult<(Vec<f32>, i32)> {
    let (wav, sample_rate) = default_tts_inference(text)
        .map_err(|e| e.to_string())?;

    Ok((wav, sample_rate))
}

// 在这个文件末尾编写针对 add_method 的单元测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_method_basic() {
        assert_eq!(add_method(5, 3), 8);
    }

    #[test]
    fn test_generate_voice() {
        generate_voice(&"Had the Russian intelligentsia been politically more mature—more patient, that is, and more understanding of the mentality of the monarchic establishment—Russia might perhaps have succeeded in making an orderly transition from a semi-constitutional to a genuinely constitutional regime.");
    }

}