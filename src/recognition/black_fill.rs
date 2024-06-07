use crate::config::CONFIG;
use crate::{models::{engine_rec::ProcessedImages, rec_result::{OutputRec, Value}, scan_json::Coordinate}, recognition::engine::Engine};
use crate::my_utils::image::*;
use image::{ImageBuffer, Luma};
use image::Rgb;
use imageproc::rect::Rect;
use imageproc::integral_image::sum_image_pixels;
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use ab_glyph::FontArc;

pub trait RecBlackFill {
    /// 填涂识别
    fn rec_black_fill(img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
    fn rendering_black_fill(output: &mut OutputRec);
    fn rendering_black_fill_show_rate(output: &mut OutputRec);
}

impl RecBlackFill for Engine {
    fn rec_black_fill(img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value> {
        let rect = Rect::at(coordinate.x, coordinate.y).of_size(coordinate.w as u32, coordinate.h as u32);
        let integral_image;
        if CONFIG.image_blackfill.image_type == 0 {
            integral_image = &img.integral_gray;
        } else {
            integral_image = &img.integral_morphology;
        }
        //计算摆正后原始区域填涂率
        let filled_ratio = calculate_fill_ratio(integral_image, rect);
        //计算制定区域最大值，默认搜索5*5范围内最大
        let neighborhood_radio = find_max_fillrate_in_neighborhood(integral_image, coordinate, filled_ratio);
        // #[cfg(debug_assertions)]
        // {
        //     println!("==={:?}所在区域填涂比{},5*5区域最值大{}===", coordinate, filled_ratio, neighborhood_radio);
        // }
        return Some(Value::Float(neighborhood_radio));
    }

    fn rendering_black_fill(output: &mut OutputRec) {
        for (_page_index, page) in output.pages.iter_mut().enumerate() {
            if matches!(page.image_rendering, None) {
                continue;
            }
            let rendering = trans_base64_to_image(&page.image_rendering.as_ref().expect("image_rendering is None"));
            if rendering.is_err() {
                continue;
            }
            let rendering = rendering.unwrap();
            let mut rendering = rendering.to_rgb8();
            for recognize in &page.recognizes {
                if recognize.rec_type == CONFIG.recognize_type.black_fill || recognize.rec_type == CONFIG.recognize_type.multi_select ||recognize.rec_type == CONFIG.recognize_type.single_select{
                    let mut max_filled_ratio_index = None;
                    let mut max_filled_ratio_value = None;
                    for (_index, rec_option) in recognize.rec_options.iter().enumerate() {
                        if let Some(Value::Float(value)) = rec_option.value {
                            if max_filled_ratio_value.map_or(true, |max_value| value > max_value) {
                                max_filled_ratio_index = Some(_index);
                                max_filled_ratio_value = Some(value);
                            }
                        }
                    }
                    // 填涂比rec_options中最大，且大于阈值min_filled_ratio的区域
                    if let Some(max_value) = max_filled_ratio_value {
                        if max_value as f32 > CONFIG.image_blackfill.min_filled_ratio {
                            let max_index = max_filled_ratio_index.expect("No max_filled_ratio_index provided");
                            // #[cfg(debug_assertions)]
                            // {
                            //     println!("===选项组最大填涂比{:?}===", recognize.rec_options[max_index].value);
                            // }
                            if let Some(coordinate) = recognize.rec_options[max_index].coordinate {
                                draw_filled_rect_mut(
                                    &mut rendering,
                                    Rect::at(coordinate.x, coordinate.y).of_size(coordinate.w as u32, coordinate.h as u32),
                                    Rgb([255u8, 0u8, 0u8]),
                                );
                            }
                        }
                    }
                }
            }

            let img_base64 = image_to_base64(&rendering);
            page.image_rendering = Some(img_base64);
        }

    }
    fn rendering_black_fill_show_rate(output: &mut OutputRec){
        let font_data = include_bytes!("../../Roboto-Thin.ttf") as &[u8];
        let font = FontArc::try_from_slice(font_data).expect("Error loading font");
        let color = Rgb([0, 0, 255]);

        for page in output.pages.iter_mut(){
            if !page.has_page {continue}
            let mut img = trans_base64_to_image(page.image_rendering.as_ref().unwrap()).unwrap().to_rgb8();

            for rec in page.recognizes.iter(){
                if rec.rec_type != CONFIG.recognize_type.black_fill && rec.rec_type != CONFIG.recognize_type.multi_select && rec.rec_type != CONFIG.recognize_type.single_select{continue}
                for option in rec.rec_options.iter(){
                    let rate = option.value.as_ref().unwrap();
                    let coor = option.coordinate.as_ref().unwrap();
                    draw_text_mut(
                        &mut img, color,
                        coor.x-CONFIG.image_blackfill.debug_rendering_show_rate_move,
                        coor.y-CONFIG.image_blackfill.debug_rendering_show_rate_move,
                        CONFIG.image_blackfill.debug_rendering_show_rate_scale,
                        &font, &format!("{rate:?}")[6..10]
                    );
                }
            }
            page.image_rendering = Some(image_to_base64(&img));
        }
    }
}

/// 计算填涂比
fn calculate_fill_ratio(image: &ImageBuffer<Luma<i64>, Vec<i64>>, rect: Rect) -> f32 {
    //计算图像中区域所有像素值的和
    let sum_pixels = sum_image_pixels(
        image,
        rect.left() as u32,
        rect.top() as u32,
        (rect.right() - 1) as u32,
        (rect.bottom() - 1) as u32
    )[0];
    let mean_pixel = sum_pixels / (rect.width() * rect.height()) as i64;
    let filled_ratio = 1.0 - mean_pixel as f32 / 255f32;
    return filled_ratio;
}

/// 以左上角（x,y,w,h）为基准，遍历所给区域附近范围，默认5*5
/// 查找最大填涂率
fn find_max_fillrate_in_neighborhood(integral_image: &ImageBuffer<Luma<i64>, Vec<i64>>, coordinate: &Coordinate, original_fillrate: f32) -> f32 {
    let mut new_fillrate = original_fillrate;
    let x = coordinate.x;
    let y = coordinate.y;
    let w = coordinate.w as u32;
    let h = coordinate.h as u32;
    //跨度向下取整
    let space = CONFIG.image_blackfill.neighborhood_size / 2;
    // 遍历x轴方向从x-2到x+2
    for i in (x - space as i32)..=(x + space as i32) {
        // 遍历y轴方向从y-2到y+2
        for j in (y - space as i32)..=(y + space as i32) {
            let rect = Rect::at(i, j).of_size(w, h);
            let fillrate = calculate_fill_ratio(integral_image, rect);
            if new_fillrate < fillrate {
                new_fillrate = fillrate;
            }
        }
    }
    return new_fillrate;
}