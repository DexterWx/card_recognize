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