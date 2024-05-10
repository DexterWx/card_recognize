//! 定义输入输出和公用结构体

pub mod scan_json;
pub mod rec_result;

/// 定义常用结构体
pub mod card{
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Copy, Clone)]
    pub struct MyPoint{
        pub x: i32, // 引擎所有坐标点均使用i32
        pub y: i32,
    }
    impl MyPoint {
        pub fn new(x: i32, y: i32) -> Self {
            MyPoint { x, y }
        }
    }
}

/// 定义引擎各种识别方法所需的结构体
/// 将每个方法所需要用到的字段整理成对应的结构体
pub mod engine_rec{
    use image::{GrayImage, ImageBuffer, Luma, RgbImage};

    use super::scan_json::{Coordinate, ModelPoint, ModelSize, PageNumberPoint};

    /// 大摆正所需要的信息
    pub struct RecInfoBaizheng<'a>{
        pub model_size: &'a ModelSize,
        pub model_points: &'a [ModelPoint;4],
        pub page_number_points: &'a Vec<PageNumberPoint>
    }

    /// 标注定位点和实际定位点，用来参照计算其他标注框的真实坐标
    pub struct ReferenceModelPoints<'a>{
        pub model_points: &'a [ModelPoint;4],
        pub real_model_points: &'a [Coordinate;4]
    }

    /// 识别需要用到的各种图片
    #[derive(Clone)]
    pub struct ProcessedImages{
        /// 未处理的原始图，为了兼容业务逻辑
        pub org: Option<String>,
        /// 原始rgb图
        pub rgb: RgbImage,
        /// 高斯模糊图，用于后续调参数生成不同的形态图
        pub blur: GrayImage,
        /// 形态学处理
        pub morphology: GrayImage,
        /// 二值灰度积分图，用来求区域像素值总和
        pub integral_gray: ImageBuffer<Luma<i64>, Vec<i64>>,
        /// 形态学处理积分图，用来求区域像素值总和
        pub integral_morphology: ImageBuffer<Luma<i64>, Vec<i64>>
    }

    #[derive(Clone)]
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

    #[derive(Clone)]
    pub struct ProcessedImagesAndModelPoints{
        pub img: ProcessedImages,
        pub real_model_points: [Coordinate;4]
    }
}


pub mod my_error{
    use thiserror::Error;
    #[derive(Error, Debug)]
    pub enum MyError {
        #[error("没有找到定位点")]
        ErrorModelPointNotFound,
    }
}
