use anyhow::{bail, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::fs;
use std::mem;
use std::error::Error;
use tauri::path::PathResolver;
use crate::MODEL_BASE_PATH;
use crate::cus_tts::helper::{
    load_text_to_speech, load_voice_style, timer, write_wav_file, sanitize_filename,
    // 假设这些 helper 函数已经被正确定义
};

const MODEL_RELATIVE_PATH: &str = "tts_models/supertonic";

// --- 命令行解析结构体 (保持不变，用于 CLI 入口) ---
#[derive(Parser, Debug)]
#[command(name = "TTS ONNX Inference")]
#[command(about = "TTS Inference with ONNX Runtime (Rust)", long_about = None)]
struct Args {
    /// Use GPU for inference (default: CPU)
    #[arg(long, default_value = "false")]
    use_gpu: bool,

    /// Path to ONNX model directory
    #[arg(long, default_value = "/Users/larry/github.com/vibe-app/pretrain_model/supertonic/onnx")]
    onnx_dir: String,
    
    // ... 其他 Args 字段保持不变 ...

    /// Number of denoising steps
    #[arg(long, default_value = "5")]
    total_step: usize,

    /// Number of times to generate
    #[arg(long, default_value = "4")]
    n_test: usize,

    /// Voice style file path(s)
    #[arg(long, value_delimiter = ',', default_values_t = vec!["/Users/larry/github.com/vibe-app/pretrain_model/supertonic/voice_styles/M1.json".to_string()])]
    voice_style: Vec<String>,

    /// Text(s) to synthesize
    #[arg(long, value_delimiter = '|', default_values_t = vec!["This morning, I took a walk in the park, and the sound of the birds and the breeze was so pleasant that I stopped for a long time just to listen.".to_string()])]
    text: Vec<String>,

    /// Output directory
    #[arg(long, default_value = "/Users/larry/Documents/output")]
    save_dir: String,
}

// 🌟 1. 新增：定义一个用于代码调用的配置结构体
// 它只包含业务逻辑需要的数据，不包含命令行解析相关的属性。
#[derive(Debug)]
pub struct TtsConfig<'a> {
    pub use_gpu: bool,
    pub onnx_dir: &'a str,
    pub total_step: usize,
    pub n_test: usize,
    pub voice_style_paths: &'a [String],
    pub text_list: &'a [String],
    pub save_dir: &'a str,
}

pub fn default_tts_inference(text: &str) -> Result<(Vec<f32>, i32)> {

    let model_path: &PathBuf = MODEL_BASE_PATH.get()
        .ok_or_else(|| anyhow::anyhow!("ONNX 路径未初始化"))?;

    let onnx_dir_path: PathBuf = model_path.join("onnx");
    let onnx_dir_str = onnx_dir_path.to_str()
        .ok_or_else(|| anyhow::anyhow!("ONNX 路径包含无效字符"))?.to_string();

    let voice_style_path = model_path.join("voice_styles/M1.json");
    let voice_style_str = voice_style_path.to_str()
        .ok_or_else(|| anyhow::anyhow!("模型路径包含无效字符"))?.to_string();

    println!("=== TTS Inference with ONNX Runtime (Rust) ===\n");
    let test_config = TtsConfig {
            use_gpu: true,
            onnx_dir: &onnx_dir_str, // 使用 mock 或最小化模型路径
            total_step: 15, // 高质量
            n_test: 1, // 对比测试时使用, 即生成多次
            voice_style_paths: &[
                voice_style_str,
            ],
            text_list: &[
                text.to_string(),
            ],
            save_dir: "output",
        };
    let (wav, sample_rate) = run_default_tts_inference(&test_config)?;

    println!("generate voice successfully \n");
    Ok((wav, sample_rate))
}
fn run_default_tts_inference(config: &TtsConfig) -> Result<(Vec<f32>, i32)> {
    println!("=== TTS Inference with ONNX Runtime (Rust) ===\n");

    let total_step = config.total_step;
    let n_test = config.n_test;
    let voice_style_paths = config.voice_style_paths;
    let text_list = config.text_list;
    let save_dir = config.save_dir;

    assert_eq!( voice_style_paths.len(), text_list.len());
    assert_eq!( n_test, 1);

    let bsz = voice_style_paths.len();

    assert!( bsz == 1);

    let mut text_to_speech = load_text_to_speech(config.onnx_dir, config.use_gpu)?;
    let style = load_voice_style(voice_style_paths, true)?;
    let (wav, _) = timer("Generating speech from text", || {
        text_to_speech.call(text_list, &style, total_step)
    })?;
    println!("\n=== Synthesis completed successfully! ===");
    Ok((wav, text_to_speech.sample_rate))
}

// 🌟 2. 新增：可被代码调用的核心函数
// 它接受 TtsConfig 结构体作为参数，彻底解耦 CLI。
pub fn run_tts_inference(config: &TtsConfig) -> Result<()> {
    println!("=== TTS Inference with ONNX Runtime (Rust) ===\n");

    // --- 1. 参数验证与解构 --- //
    let total_step = config.total_step;
    let n_test = config.n_test;
    let voice_style_paths = config.voice_style_paths;
    let text_list = config.text_list;
    let save_dir = config.save_dir;

    if voice_style_paths.len() != text_list.len() {
        bail!(
            "Number of voice styles ({}) must match number of texts ({})",
            voice_style_paths.len(),
            text_list.len()
        );
    }

    let bsz = voice_style_paths.len();

    // --- 2. Load TTS components --- //
    // 使用 config 中的 onnx_dir 和 use_gpu
    let mut text_to_speech = load_text_to_speech(config.onnx_dir, config.use_gpu)?;

    // --- 3. Load voice styles --- //
    let style = load_voice_style(voice_style_paths, true)?;

    // --- 4. Synthesize speech --- //
    fs::create_dir_all(save_dir)?;

    for n in 0..n_test {
        println!("\n[{}/{}] Starting synthesis...", n + 1, n_test);

        // 调用逻辑保持不变
        let (wav, duration) = timer("Generating speech from text", || {
            text_to_speech.call(text_list, &style, total_step)
        })?;

        // Save outputs
        let wav_len = wav.len() / bsz;
        for i in 0..bsz {
            let fname = format!("{}_{}.wav", sanitize_filename(&text_list[i], 20), n + 1);
            let actual_len = (text_to_speech.sample_rate as f32 * duration[i]) as usize;

            let wav_start = i * wav_len;
            let wav_end = wav_start + actual_len.min(wav_len);
            let wav_slice = &wav[wav_start..wav_end];

            let output_path = PathBuf::from(save_dir).join(&fname);
            write_wav_file(&output_path, wav_slice, text_to_speech.sample_rate)?;
            println!("Saved: {}", output_path.display());
        }
    }

    println!("\n=== Synthesis completed successfully! ===");
    
    // 🌟 关键修改：移除 mem::forget 和 libc::_exit(0)
    // 在库函数中，绝不能手动释放内存或强制退出进程。
    // 让 text_to_speech 在函数结束时正常 drop 即可。
    
    Ok(())
}


// 🌟 3. 重命名并修改旧的 main99 函数，作为 CLI 的入口
// 它的唯一职责就是解析命令行参数，然后调用核心逻辑。
pub fn cli_entrypoint() -> Result<()> {
    // --- 1. Parse arguments --- //
    let args = Args::parse();
    
    // 2. 将 Args 转换为 TtsConfig
    let config = TtsConfig {
        use_gpu: args.use_gpu,
        onnx_dir: &args.onnx_dir,
        total_step: args.total_step,
        n_test: args.n_test,
        voice_style_paths: &args.voice_style,
        text_list: &args.text,
        save_dir: &args.save_dir,
    };
    
    // 3. 调用核心逻辑
    run_tts_inference(&config)?;
    
    // 4. 处理 CLI 退出时的特殊清理（如果确实需要）
    // 只有在 CLI/main 函数中才需要执行这些危险操作
    // 如果 ONNX 仍然导致问题，这里可以考虑保留清理代码，但要确保它不会在测试中运行。
    
    // 假设 text_to_speech 是在 run_tts_inference 内部被 drop 的，这里不再需要清理。
    
    Ok(())
}


// --- 4. 修改测试函数 ---
#[cfg(test)]
mod tests {
    use super::*; 
    use anyhow::Result; // 确保导入 Result

    #[test]
    fn test_default_tts_inference() -> Result<()> {

        bad_text = "The electoral law was worked out at meetings of officials and public representatives. The principal question was whether to provide for an equal and direct vote or a vote organized by estates and cast indirectly, through electoral chambers.14 Following the recommendation of the bureaucracy, it was decided to adopt a system of indirect voting by estates in order to reduce the weight of constituencies regarded as more likely to elect radical deputies. There were to be four electoral curiae: for the gentry (dvoriane), for burghers (meshchane), for peasants, and for workers, the last-named group now given the vote which the Bulygin project had denied it. The franchise was so contrived that one gentry vote carried the weight of three burgher, fifteen peasant, and forty-five worker votes.15 Except in the large cities, the voters cast their ballots for electors who, in turn, selected either other electors or the deputies themselves. These electoral provisions rejected the democratic franchise advocated by Russian liberal and socialist parties which called for the “four-tail” vote—universal, direct, equal, and secret. It was the government’s hope that by reducing the urban vote it would ensure a tractable Duma.";
        let (wav, sample_rate) = default_tts_inference("This is a test text for TTS inference.")?;
        // assert!(result.is_ok());
        write_wav_file(&"/Users/larry/Documents/output/b.wav", &wav, sample_rate)?;
        Ok(())
    }

    #[test] 
    fn test_run_tts_simple() -> Result<()> {
        // 🌟 关键修改：手动构造 TtsConfig 用于测试
        let test_config = TtsConfig {
            use_gpu: true,
            onnx_dir: "/Users/larry/github.com/vibe-app/pretrain_model/supertonic/onnx", // 使用 mock 或最小化模型路径
            total_step: 15,
            n_test: 1,
            voice_style_paths: &[
                "/Users/larry/github.com/vibe-app/pretrain_model/supertonic/voice_styles/F1.json".to_string(),
                "/Users/larry/github.com/vibe-app/pretrain_model/supertonic/voice_styles/M1.json".to_string(),
            ],
            text_list: &[
                "This text-to-speech system runs entirely in your browser, providing fast and private operation without sending any data to external servers.".to_string(),
                "Had the Russian intelligentsia been politically more mature—more patient, that is, and more understanding of the mentality of the monarchic establishment—Russia might perhaps have succeeded in making an orderly transition from a semi-constitutional to a genuinely constitutional regime.".to_string(),
            ],
            save_dir: "/Users/larry/Documents/output",
        };
        let result = run_tts_inference(&test_config);
        
        assert!(result.is_ok()); 
        Ok(())
    }
}