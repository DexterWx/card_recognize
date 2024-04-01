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

/// 图片摆正处理参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageBaizheng {
    pub page_number_diff: f32,
    pub model_point_wh_cosine_similarity: f32
}

/// 识别类型参数
#[derive(Debug, Deserialize, Serialize)]
pub struct RecognitionType {
    pub coordinate: u8,
    pub barcode: u8,
    pub black_fill: u8,
    pub number: u8,
    pub vx: u8,
    pub qrcode: u8
}

/// 配置参数
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub image_process: ImageProcess,
    pub image_baizheng: ImageBaizheng,
    pub recognize_type: RecognitionType,
    // 其他配置参数
}

// 全局配置单例
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    // 读取配置文件
    let file = File::open("src/config.yaml").expect("Failed to open config file");
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).expect("Failed to parse config")
});