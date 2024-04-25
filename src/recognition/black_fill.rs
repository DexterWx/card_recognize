use crate::config::CONFIG;
use crate::{models::{engine_rec::ProcessedImages, rec_result::{OutputRec, Value}, scan_json::Coordinate}, recognition::engine::Engine};
use crate::my_utils::image::*;
use image::Rgb;
use imageproc::rect::Rect;
use imageproc::integral_image::sum_image_pixels;
use imageproc::drawing::draw_filled_rect_mut;
use std::env;
pub trait RecBlackFill{
  /// 填涂识别
  fn rec_black_fill(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
  fn rendering_black_fill(&self, output: &mut OutputRec);
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
    // println!("==={:?}所在区域填涂比{}===", coordinate,filled_ratio);

    return Some(Value::Float(filled_ratio));
  }

  fn rendering_black_fill(&self, output: &mut OutputRec) {
    for (i, page) in output.pages.iter().enumerate() {  
      let rendering = trans_base64_to_image(&page.image_rendering.as_ref().expect("image_rendering is None"));
      let mut rendering = rendering.to_rgb8();
      for recognize in &page.recognizes {  
        if recognize.rec_type == CONFIG.recognize_type.black_fill {
          let mut max_filled_ratio_index = None;
          let mut max_filled_ratio_value = None;
          for (_index, rec_option) in recognize.rec_options.iter().enumerate() {
          
            if let Some(Value::Float(value)) = rec_option.value {  
              if match max_filled_ratio_value {  
                  Some(max_value) => value > max_value,  
                  None => true  
              } {   
                  max_filled_ratio_index = Some(_index);
                  max_filled_ratio_value = Some(value);
              }
            }
          }
          // 填涂比rec_options中最大，且大于0.5的区域
          if let Some(max_index) = max_filled_ratio_index {  
            if max_index as f32 > 0.5 {
              let coordinate = recognize.rec_options[max_index].coordinate;
              match coordinate {
                Some(c) => {
                  draw_filled_rect_mut(
                    &mut rendering, 
                    Rect::at(c.x, c.y).of_size(c.w as u32, c.h as u32), 
                    Rgb([255u8, 0u8, 0u8]),
                  );
                },
                None => {  
                  // 异常处理  
                }  
              }
            } 
          }
        }
      }

      // let img_base64 = image_to_base64(&rendering);
      // page.image_rendering = Some(img_base64);

      let cwd = env::current_dir().unwrap();
      let str = format!("dev/test_data/{}.jpg",i);
      let path = cwd.join(str);
      rendering.save(path).unwrap();

    }

  }
}