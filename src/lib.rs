pub mod recognition;
pub mod models;
pub mod my_utils;
pub mod config;

use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use image::{ImageBuffer, Rgb, RgbImage};
use std::fs::File;
use std::io::Read;

use models::scan_json::InputScan;
use recognition::baizheng::rotate_with_location;
use config::CONFIG;

#[cfg(test)]
mod tests {

    use std::path::PathBuf;
    use crate::{models::{engine_rec::{RecInfoBaizheng, ReferenceModelPoints}, scan_json::{ModelPoint, ModelSize}}, my_utils::{image::process_image, io::compatible_path_format}, recognition::baizheng::rotate_processed_image_90};

    use image::{Luma, Rgba};
    use imageproc::{distance_transform::Norm, filter::gaussian_blur_f32, morphology::{dilate, erode}};

    use super::*;

    #[test]
    fn test_json() -> Result<()>{
        let scan_path = compatible_path_format("dev/test_data/cards/193265/scan.json");
        let mut file = File::open(scan_path).expect("Failed to open file");

        // 读取文件内容
        let mut json_str = String::new();
        file.read_to_string(&mut json_str)
            .expect("Failed to read file");

        // 将 JSON 解析为 MyStruct 结构体
        let parsed_struct: InputScan = serde_json::from_str(&json_str).unwrap();
        let input1 = InputScan::renew(parsed_struct);
        println!("{:?}", input1);
        Ok(())
    }
        
    #[test]
    fn test_image(){
        use imageproc::drawing::{draw_filled_circle_mut};

        let img_path = compatible_path_format("dev/test_data/cards/194144/images/03e4bfb222d86ae8501b6eaf544947c0.jpg");
        let model_size = ModelSize{w:50,h:100};
        let mut processed_imgs = process_image(&model_size, img_path);
        processed_imgs.rgb.save(compatible_path_format("dev/test_data/output_location.jpg")).expect("Failed to save image");
        // let [lt,rt,ld,rd] = rotate_with_location(&mut processed_imgs);
        // for point in [lt,rt,ld,rd]{
        //     draw_filled_circle_mut(&mut processed_imgs.rgb, (point.x as i32, point.y as i32), 10, Rgb([0, 0, 255]));
        // }
        
    }

    #[test]
    fn test_config(){
        println!("{:?}", CONFIG);
    }

    #[test]
    fn test_sum_pix(){
        use crate::recognition::baizheng::{rotate_with_location,rotate_with_page_number};
        use crate::my_utils::image::process_image;

        let scan_path = compatible_path_format("dev/test_data/cards/194144/scan.json");
        let mut file = File::open(scan_path).expect("Failed to open file");

        // 读取文件内容
        let mut json_str = String::new();
        file.read_to_string(&mut json_str)
            .expect("Failed to read file");

        // 将 JSON 解析为 MyStruct 结构体
        let parsed_struct: InputScan = serde_json::from_str(&json_str).unwrap();
        let input_scan = InputScan::renew(parsed_struct);

        let img_path = compatible_path_format("dev/test_data/cards/194144/images/03e4bfb222d86ae8501b6eaf544947c0.jpg");
        let mut processed_imgs = process_image(&input_scan.pages[0].model_size,img_path);
        let wh = (
            input_scan.pages[0].model_points[0].coordinate.w,
            input_scan.pages[0].model_points[0].coordinate.h
        );
        let real_model_points = rotate_with_location(&mut processed_imgs, wh);
        processed_imgs.rgb.save(compatible_path_format("dev/test_data/output1.jpg")).expect("Failed to save image");
        let mut baizheng_info = RecInfoBaizheng{
            model_size: input_scan.pages[0].model_size,
            page_number_points: input_scan.pages[0].page_number_points.clone(),
            reference_model_points: ReferenceModelPoints{
                model_points: <std::option::Option<[ModelPoint; 4]> as Clone>::clone(&input_scan.pages[0].model_points_4).unwrap(),
                real_model_points: real_model_points,
            }
        };
        rotate_with_page_number(&mut baizheng_info, &mut processed_imgs);
        processed_imgs.rgb.save(compatible_path_format("dev/test_data/output2.jpg")).expect("Failed to save image");
    }

}
