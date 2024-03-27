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
use recognition::baizheng::rotate_with_location;
use config::CONFIG;

#[cfg(test)]
mod tests {

    use image::{Luma, Rgba};
    use imageproc::{distance_transform::Norm, filter::gaussian_blur_f32, morphology::{dilate, erode}};

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
        let input1 = Input1::renew(parsed_struct);
        println!("{:?}", input1);
        Ok(())
    }
        
    #[test]
    fn test_image(){
        use imageproc::drawing::{draw_filled_circle_mut};

        let img = image::open("dev/test_data/cards/193265/images/3a7c7f4fff35cada1b9588133a943d1f.jpg").expect("Failed to open image file");
        let (mut img,[lt,rt,ld,rd]) = rotate_with_location(&img);
        for point in [lt,rt,ld,rd]{
            draw_filled_circle_mut(&mut img, (point.x as i32, point.y as i32), 10, Rgba([0, 0, 255, 0]));
        }
        img.save("dev/test_data/output_location.jpg").expect("Failed to save image");
    }

    #[test]
    fn test_config(){
        println!("{:?}", CONFIG);
    }

    #[test]
    fn test_baizheng(){
        use recognition::baizheng::rotate_with_page_number;
        use models::engine_rec::RecInfoBaizheng;

        let img = image::open("dev/test_data/cards/193265/images/3a7c7f4fff35cada1b9588133a943d1f.jpg").expect("Failed to open image file");
        let (img,[lt,rt,ld,rd]) = rotate_with_location(&img);

        let scan_path = "dev/test_data/cards/193265/scan.json";
        let mut file = File::open(scan_path).expect("Failed to open file");

        // 读取文件内容
        let mut json_str = String::new();
        file.read_to_string(&mut json_str)
            .expect("Failed to read file");

        // 将 JSON 解析为 MyStruct 结构体
        let parsed_struct: Input1 = serde_json::from_str(&json_str).unwrap();
        let input1 = Input1::renew(parsed_struct);

        let a = RecInfoBaizheng{
            model_size: &input1.pages[0].model_size,
            page_number_points: &input1.pages[0].page_number_points,
            model_points: &input1.pages[0].model_points_3.as_ref().unwrap(),
            real_model_points: &[lt,rt,ld]
        };
        // rotate_with_page_number(&a, &img);
    }

    #[test]
    fn test_sum_pix(){
        use imageproc::integral_image::sum_image_pixels;
        use imageproc::integral_image::{integral_image};
        let img = image::open("dev/test_data/cards/193265/images/3a7c7f4fff35cada1b9588133a943d1f.jpg").expect("Failed to open image file");
        let gray_img = img.to_luma8();
        // 对灰度图像进行高斯模糊
        let mut blurred_img = gaussian_blur_f32(&gray_img, CONFIG.image_process.gaussian_blur_sigma);

        // 对模糊后的图像进行二值化
        blurred_img.enumerate_pixels_mut().for_each(|(_, _, pixel)| {
            if pixel[0] > CONFIG.image_process.binarization_threshold {
                *pixel = Luma([255u8]);
            } else {
                *pixel = Luma([0u8]);
            }
        });

         // 膨胀操作
        let dilated_img = dilate(&blurred_img, Norm::LInf, CONFIG.image_process.morphology_kernel);

        // 腐蚀操作
        let eroded_img = erode(&dilated_img, Norm::LInf, CONFIG.image_process.morphology_kernel);
        
        let blurred_img = integral_image(&eroded_img);

        let a:[i64;1] = sum_image_pixels(&blurred_img, 104, 134, 115, 164);
        // let p = blurred_img.get_pixel(0, 0);
        println!("{a:?}");
    }

}
