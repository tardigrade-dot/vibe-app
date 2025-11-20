mod helper;
pub mod example_onnx;

use crate::cus_tts::helper::{
    load_text_to_speech, load_voice_style, timer, write_wav_file, sanitize_filename,
};

use crate::cus_tts::example_onnx::run_tts_inference;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts() {
        // let mut text_to_speech = load_text_to_speech("/Users/larry/github.com/vibe-app/pretrain_model/supertonic/onnx", true)?;
        // let style = load_voice_style(["assets/voice_styles/M1.json"], true)?;
        
    }

}
    