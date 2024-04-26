use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

/// 图片预处理处理参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageProcess {
    pub gaussian_blur_sigma: f32,
    pub binarization_threshold: u8,
    pub morphology_kernel: u8,
    pub empty_image_threshold: u8
}

/// 图片摆正处理参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageBaizheng {
    pub page_number_diff: f32,
    pub model_point_wh_cosine_similarity: f32,
    pub model_point_diff: i32,
}

/// 判断填涂比参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageBlackFill {
    pub image_type: String,
    pub min_filled_ratio: f32,
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
    pub image_blackfill: ImageBlackFill,
    pub recognize_type: RecognitionType,
    // 其他配置参数
}

// 全局配置单例
#[cfg(debug_assertions)]
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    // 读取配置文件
    let file = File::open("config.yaml").expect("Failed to open config file");
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).expect("Failed to parse config")
});


#[cfg(not(debug_assertions))]
pub static CONFIG: Config = Config{
    image_process: ImageProcess{
        gaussian_blur_sigma: 1.0,
        binarization_threshold: 180,
        morphology_kernel: 5,
        empty_image_threshold: 253,
    },
    image_baizheng: ImageBaizheng{
        page_number_diff: 0.21,
        model_point_wh_cosine_similarity: 0.985,
        model_point_diff:50
    },
    image_blackfill: ImageBlackFill{
        image_type: "integral_gray",
        min_filled_ratio: 0.7
    },
    recognize_type: RecognitionType{
        black_fill: 1,
        vx: 2,
        number: 3,
        qrcode: 4,
        barcode: 5,
        coordinate: 6,
    }
};