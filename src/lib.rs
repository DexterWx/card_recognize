pub mod recognition;
pub mod models;
pub mod my_utils;
pub mod config;

use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use image::{ImageBuffer, Rgb, RgbImage};
use std::fs::File;
use std::io::Read;

use models::scan_json::Input1;
use recognition::baizheng::process_img;
use config::CONFIG;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_json() -> Result<()>{
        let scan_path = "dev/test_data/cards/193265/scan.json";
        let mut file = File::open(scan_path).expect("Failed to open file");

        // 读取文件内容
        let mut json_str = String::new();
        file.read_to_string(&mut json_str)
            .expect("Failed to read file");

        // 将 JSON 解析为 MyStruct 结构体
        let parsed_struct: Input1 = serde_json::from_str(&json_str).unwrap();
        println!("{:?}", parsed_struct);
        Ok(())
    }
        
    #[test]
    fn test_image(){
        use imageproc::drawing::{draw_filled_circle_mut};
        let (mut img,[lt,rt,ld,rd]) = process_img("dev/test_data/test_3.jpg");
        for point in [lt,rt,ld,rd]{
            draw_filled_circle_mut(&mut img, (point.x as i32, point.y as i32), 10, Rgb([0, 0, 255]));
        }
        img.save("dev/test_data/output_location.jpg").expect("Failed to save image");
    }

    #[test]
    fn test_config(){
        println!("{:?}", CONFIG);
    }

}
