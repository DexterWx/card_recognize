//! 定义输入输出和公用结构体

pub mod scan_json;
mod rec_result;

pub mod card{
    #[derive(Debug,Copy, Clone)]
    pub struct MyPoint{
        pub x: i32, // 引擎所有坐标点均使用i32
        pub y: i32,
    }
}

/// 定义引擎各种识别方法所需的结构体
/// 将每个方法所需要用到的字段整理成对应的结构体
pub mod engine_rec{
    use image::{GrayImage, ImageBuffer, Luma, RgbImage};

    use super::scan_json::{ModelSize, PageNumberPoint, ModelPoint};
    use super::card::MyPoint;

    pub struct ReferenceModelPoints{
        pub model_points: [ModelPoint;4],
        pub real_model_points: [MyPoint;4]
    }
    /// 大摆正所需要的信息
    pub struct RecInfoBaizheng{
        pub model_size: ModelSize,
        pub page_number_points: Vec<PageNumberPoint>,
        pub reference_model_points: ReferenceModelPoints,
    }
    /// 识别需要用到的各种图片
    pub struct ProcessedImages{
        /// 原始rgb图
        pub rgb: RgbImage,
        /// 二值灰度图
        pub gray: GrayImage,
        /// 形态学处理
        pub morphology: GrayImage,
        /// 二值灰度积分图，用来求区域像素值总和
        pub integral_gray: ImageBuffer<Luma<i64>, Vec<i64>>,
        /// 形态学处理积分图，用来求区域像素值总和
        pub integral_morphology: ImageBuffer<Luma<i64>, Vec<i64>>
    }
}

