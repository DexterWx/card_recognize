//! 定义输入输出和公用结构体
//!
//!
//! # Examples
//!
//! ```rust
//! use my_module::some_function;
//!
//! let result = some_function(42);
//! assert_eq!(result, 42);
//! ```
//!

pub mod scan_json;
mod rec_result;

pub mod card{
    #[derive(Debug)]
    pub struct MyPoint{
        pub x: i32, // 引擎所有坐标点均使用i32
        pub y: i32,
    }
}

/// 定义引擎各种识别方法所需的结构体
/// 将每个方法所需要用到的字段整理成对应的结构体
pub mod engine_rec{
    use super::scan_json::{ModelSize, PageNumberPoint};
    /// 大摆正所需要的信息
    pub struct RecInfoBaizheng<'a>{
        pub model_size: &'a ModelSize,
        pub page_number_points: &'a Vec<PageNumberPoint>
    }
}

