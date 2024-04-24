use imageproc::integral_image::sum_image_pixels;
use imageproc::rect::Rect;
use crate::{models::{engine_rec::ProcessedImages, rec_result::{OutputRec, Value}, scan_json::Coordinate}, recognition::engine::Engine};
use imageproc::drawing::draw_filled_rect_mut;
use image::Rgb;

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
    // println!("====={:?}所在区域填涂比{}=====", coordinate,filled_ratio);     

    return Some(Value::Float(filled_ratio));
  }

  fn rendering_black_fill(&self, output: &mut OutputRec) {

    // 绘制填涂区域
    // let mut img_rgb_rendering = img.rgb.clone();
    // draw_filled_rect_mut(
    //     &mut img_rgb_rendering,   
    //     Rect::at(coordinate.x, coordinate.y).of_size(coordinate.w as u32, coordinate.h as u32),    
    //     Rgb([255u8, 0u8, 0u8]),  
    // );

      
      // let rendering = trans_base64_to_image(&page.image_rotated.as_ref().expect("image_rendering is None"));
      // let mut rendering = rendering.to_rgb8();
      // for point in img_and_model_points.as_ref().unwrap().real_model_points.iter(){
      //     draw_filled_circle_mut(&mut rendering,(point.x,point.y),3, Rgb([0,0,255]));
      //     draw_filled_circle_mut(&mut rendering,(point.x+point.w,point.y+point.h),3, Rgb([0,0,255]));
      // }
      // let img_base64 = image_to_base64(&rendering);
      // page.image_rendering = Some(img_base64);   

  }
}