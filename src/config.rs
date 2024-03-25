use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

/// 图片预处理处理参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageProcess {
    pub gaussian_blur_sigma: f32,
    pub binarization_threshold: u8,
    pub morphology_kernel: u8
}

/// 配置参数
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub image_process: ImageProcess,
    // 其他配置参数
}

// 全局配置单例
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    // 读取配置文件
    let file = File::open("src/config.yaml").expect("Failed to open config file");
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).expect("Failed to parse config")
});