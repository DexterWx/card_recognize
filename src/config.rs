use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

/// 图片预处理处理参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageProcess {
    pub gaussian_blur_sigma: f32,
    pub retry_args: [ProcessedImagesArgs;5],
    pub empty_image_threshold: f64,
    pub fill_args: FillArgs,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FillArgs{
    pub all_fill_or_empty_min_var: f64,
    pub all_fill_var: f64,
    pub all_fill_otsu: u8,
    pub fill_same_max: u8,
    pub empty_same_max: u8,
    pub same_var: f64,
    pub same_var_exam_number: f64,
    pub otsu_black_fill_sep_weight: f64,
    pub text_a: TextBaseRate,
    pub text_b: TextBaseRate,
    pub text_c: TextBaseRate,
    pub text_d: TextBaseRate
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TextBaseRate{
    pub text: char,
    pub rate: f32,
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
    pub area_assist_point_nearby_length: i32,
    pub single_area_assist_delength: i32,
    pub area_assist_point_nearby_step: i32,
    pub area_assist_point_nearby_retry: u8,
    pub valid_coordinates4_cosine_similarity: f32,
    pub valid_coordinates_wh_sum_mean_dis: f32
}

/// 定位真实框参数
#[derive(Debug, Deserialize, Serialize)]
pub struct Location {
    pub select_model_point_cal_real_coor_y_boundary: f32
}

/// 判断填涂比参数
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageBlackFill {
    pub neighborhood_size: u8,
    pub neighborhood_size_exam_number: u8,
    pub image_type: u8,
    pub min_filled_ratio: f32,
    pub debug_rendering_show_rate_move: i32,
    pub debug_rendering_show_rate_scale: f32
}

/// 识别类型参数
#[derive(Debug, Deserialize, Serialize)]
pub struct RecognitionType {
    pub coordinate: u8,
    pub barcode: u8,
    pub black_fill: u8,
    pub number: u8,
    pub vx: u8,
    pub qrcode: u8,
    pub single_select: u8,
    pub multi_select: u8,
    pub exam_number: u8,
}

/// 配置参数
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub image_process: ImageProcess,
    pub image_baizheng: ImageBaizheng,
    pub image_blackfill: ImageBlackFill,
    pub recognize_type: RecognitionType,
    pub location: Location
    // 其他配置参数
}

// 全局配置单例
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    // 读取配置文件
    let file = File::open("config.yaml").expect("Failed to open config file");
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).expect("Failed to parse config")
});