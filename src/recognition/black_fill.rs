use imageproc::integral_image::sum_image_pixels;
use imageproc::rect::Rect;
use crate::{models::{engine_rec::ProcessedImages, rec_result::Value, scan_json::Coordinate}, recognition::engine::Engine};

// use std::env;
// use imageproc::drawing::draw_filled_rect_mut;
// use image::Rgb;

pub trait RecBlackFill{
    /// 填涂识别
    fn rec_black_fill(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
}

impl RecBlackFill for Engine {
    fn rec_black_fill(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>{
        
        let rect = Rect::at(coordinate.x, coordinate.y).of_size(coordinate.w as u32, coordinate.h as u32);
        //计算图像中区域所有像素值的和
        let sum_pixels = sum_image_pixels(
          &img.integral_gray,
          rect.left() as u32,
          rect.top() as u32,
          (rect.right()-1) as u32,
          (rect.bottom()-1) as u32
        )[0];
        let mean_pixel = sum_pixels / (rect.width() * rect.height()) as i64;
        let filled_ratio = 1.0 - mean_pixel as f32 / 255f32;
        // println!("====={:?}所在区域填涂比{}=====", coordinate,filled_ratio);     

        // 绘制要识别的填涂区域
        // let mut img_rgb_painting = img.rgb.clone();
        // draw_filled_rect_mut(
        //     &mut img_rgb_painting,   
        //     Rect::at(coordinate.x, coordinate.y).of_size(coordinate.w as u32, coordinate.h as u32),    
        //     Rgb([255u8, 0u8, 0u8]),  
        // );

        // let cwd = env::current_dir().unwrap();
        // let str = format!("dev/test_data/rec_black_fill/{}_{}_{}_{}.jpg", coordinate.x,coordinate.y,coordinate.w,coordinate.h);
        // let path = cwd.join(str);
        // img_rgb_painting.save(path).unwrap();
        return Some(Value::Float(filled_ratio));
    }
}