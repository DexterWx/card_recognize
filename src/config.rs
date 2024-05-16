use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

/// 图片预处理处理参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageProcess {
    pub gaussian_blur_sigma: f32,
    pub retry_args: [ProcessedImagesArgs;5],
    pub empty_image_threshold: u8
}
/// 需要多次尝试的参数
#[derive(Debug, Deserialize, Serialize)]
    pub struct ProcessedImagesArgs{
        pub binarization_threshold: u8,
        pub erode_kernel: u8,
        pub morphology_kernel: u8
    }
    impl ProcessedImagesArgs {
        // 构造函数
        pub fn new(binarization_threshold: u8, erode_kernel: u8, morphology_kernel: u8) -> Self {
            Self {
                binarization_threshold,
                erode_kernel,
                morphology_kernel
            }
        }
    }

/// 图片摆正处理参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageBaizheng {
    pub page_number_diff: f32,
    pub model_point_wh_cosine_similarity: f32,
    pub model_points_3_angle_threshold: f32,
    pub model_point_min_wh: i32,
    pub model_point_max_wh: i32,
    pub model_point_diff: i32,
    pub model_point_scan_range: i32,
    pub assist_point_scan_range: i32,
    pub assist_point_min_distance: i32,
    pub assist_point_max_distance: i32,
    pub model_point_min_distance: i32,
    pub model_point_max_distance: i32,
    pub assist_point_nearby_length: i32,
    pub valid_coordinates4_cosine_similarity: f32,
    pub valid_coordinates_wh_sum_mean_dis: f32
}

/// 判断填涂比参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageBlackFill {
    pub neighborhood_size: u8,
    pub image_type: u8,
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

// 定义硬编码的配置参数
#[cfg(not(debug_assertions))]
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let image_process = ImageProcess {
        gaussian_blur_sigma: 1.0,
        empty_image_threshold: 253,
        retry_args: [
            ProcessedImagesArgs::new(172, 1, 5),
            ProcessedImagesArgs::new(172, 2, 5),
            ProcessedImagesArgs::new(172, 2, 6),
            ProcessedImagesArgs::new(172, 3, 6),
            ProcessedImagesArgs::new(172, 0, 6),
        ],
    };
    
    let image_baizheng = ImageBaizheng {
        page_number_diff: 0.23,
        model_point_wh_cosine_similarity: 0.985,
        model_points_3_angle_threshold: 1.046,
        model_point_min_wh: 10,
        model_point_max_wh: 70,
        model_point_diff: 45,
        model_point_scan_range: 6,
        assist_point_scan_range: 8,
        assist_point_min_distance: 6,
        assist_point_max_distance: 25,
        model_point_min_distance: 6,
        model_point_max_distance: 60,
        assist_point_nearby_length: 4,
        valid_coordinates4_cosine_similarity: 0.996,
        valid_coordinates_wh_sum_mean_dis: 8.0
    };
    
    let image_blackfill = ImageBlackFill {
        neighborhood_size: 5,
        image_type: 0,
        min_filled_ratio: 0.5,
    };
    
    let recognize_type = RecognitionType {
        coordinate: 6,
        barcode: 5,
        black_fill: 1,
        number: 3,
        vx: 2,
        qrcode: 4,
    };

    Config {
        image_process,
        image_baizheng,
        image_blackfill,
        recognize_type,
        // 其他配置参数
    }
});