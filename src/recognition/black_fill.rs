use crate::config::CONFIG;
use crate::models::rec_result::RecOption;
use crate::models::scan_json::Value;
use crate::my_utils::math::get_otsu;
use crate::{models::{engine_rec::ProcessedImages, rec_result::OutputRec, scan_json::Coordinate}, recognition::engine::Engine};
use crate::my_utils::image::*;
use image::{ImageBuffer, Luma};
use image::Rgb;
use imageproc::rect::Rect;
use imageproc::integral_image::sum_image_pixels;
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use ab_glyph::FontArc;

pub trait RecBlackFill {
    /// 填涂识别
    fn rec_black_fill(img: &ProcessedImages, coordinate: &Coordinate, value: &Option<Value>) -> Option<Value>;
    fn binary_fill_rate(output: &mut OutputRec);
    fn rendering_black_fill(output: &mut OutputRec);
    fn rendering_black_fill_show_rate(output: &mut OutputRec);
}

impl RecBlackFill for Engine {
    fn binary_fill_rate(output: &mut OutputRec) {
        for page in output.pages.iter_mut(){
            if !page.has_page {continue}
            for rec in page.recognizes.iter_mut(){
                if rec.rec_type==CONFIG.recognize_type.single_select {
                    set_single_fill_rate(&mut rec.rec_options);
                }
                if rec.rec_type==CONFIG.recognize_type.multi_select {
                    set_multi_fill_rate(&mut rec.rec_options)
                }
            }
        }
    }
    fn rec_black_fill(img: &ProcessedImages, coordinate: &Coordinate, value: &Option<Value>) -> Option<Value> {
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
        let mut filled_ratio = find_max_fillrate_in_neighborhood(integral_image, coordinate, filled_ratio);
        //调整不同选项的选项的基础阈值，微调填涂率
        if !value.is_none(){
            filled_ratio = finetune_rate(filled_ratio, value.as_ref().unwrap());
        }

        filled_ratio = filled_ratio.min(1f32).max(0f32);  
        return Some(Value::Float(filled_ratio));
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
                    for (_index, rec_option) in recognize.rec_options.iter().enumerate() {
                        if let Some(Value::Float(value)) = rec_option.value {
                            if value != 1f32 {continue}
                            let coordinate = rec_option.coordinate.unwrap();
                            draw_filled_rect_mut(
                                &mut rendering,
                                Rect::at(coordinate.x, coordinate.y).of_size(coordinate.w as u32, coordinate.h as u32),
                                Rgb([255u8, 0u8, 0u8]),
                            );
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
                    let rate = option._value.as_ref().unwrap();
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

fn set_single_fill_rate(options: &mut Vec<RecOption>){

    if options.len() == 0 {return}
    let fill_rates_u8 = get_array_values_for_otsu(options);
    let mut best_threshold = get_otsu(&fill_rates_u8);
    if *fill_rates_u8.iter().max().unwrap() - *fill_rates_u8.iter().min().unwrap() < CONFIG.image_process.fill_args.fill_same_min_max_diff_for_all_empty
        && *fill_rates_u8.iter().max().unwrap() < CONFIG.image_process.fill_args.fill_same_max
    {
        best_threshold = 100;
    }
    if *fill_rates_u8.iter().max().unwrap() - *fill_rates_u8.iter().min().unwrap() <= CONFIG.image_process.fill_args.fill_same_min_max_diff_for_all_fill
        && *fill_rates_u8.iter().max().unwrap() >= CONFIG.image_process.fill_args.fill_same_max
    {
        best_threshold = 0;
    }
    #[cfg(debug_assertions)]
    {
        println!("{fill_rates_u8:?}  {best_threshold:?}");
    }
    set_filled_use_threshold(options, best_threshold);

}



fn set_multi_fill_rate(options: &mut Vec<RecOption>){

    if options.len() == 0 {return}
    let fill_rates_u8 = get_array_values_for_otsu(options);
    let mut best_threshold = get_otsu(&fill_rates_u8);
    if *fill_rates_u8.iter().max().unwrap() - *fill_rates_u8.iter().min().unwrap() < CONFIG.image_process.fill_args.fill_same_min_max_diff_for_all_empty
        && *fill_rates_u8.iter().max().unwrap() < CONFIG.image_process.fill_args.fill_same_max
    {
        best_threshold = 100;
    }
    if *fill_rates_u8.iter().max().unwrap() - *fill_rates_u8.iter().min().unwrap() <= CONFIG.image_process.fill_args.fill_same_min_max_diff_for_all_fill
        && *fill_rates_u8.iter().max().unwrap() >= CONFIG.image_process.fill_args.fill_same_max
    {
        best_threshold = 0;
    }
    #[cfg(debug_assertions)]
    {
        println!("{fill_rates_u8:?}  {best_threshold:?}");
    }
    set_filled_use_threshold(options, best_threshold);
    
}

fn get_array_values_for_otsu(options: &Vec<RecOption>) -> Vec<u8>{
    let mut array = Vec::new();
    for option in options.iter(){
        let value = option.value.as_ref().unwrap().to_float().unwrap();
        let mut _value = value.min(1f32).max(0f32) as u32;
        array.push((value * 100f32) as u8);
    }
    array
}

fn set_filled_use_threshold(options: &mut Vec<RecOption>, threshold: u8){
    for option in options.iter_mut(){
        let value = option.value.as_ref().unwrap().to_float().unwrap();
        let value = (value * 100f32) as u8;
        if value > threshold {
            option.value = Some(Value::Float(1f32));
        }
        else {
            option.value = Some(Value::Float(0f32));
        }
    }
}

fn finetune_rate(rate: f32, value: &Value) -> f32 {
    if value.to_string().is_none(){return rate}
    let value = value.to_string().unwrap().chars().next();
    if value.is_none(){return rate}
    let value = value.unwrap();
    if value == CONFIG.image_process.fill_args.text_a.text{
        return rate-CONFIG.image_process.fill_args.text_a.rate;
    }
    if value == CONFIG.image_process.fill_args.text_b.text{
        return rate-CONFIG.image_process.fill_args.text_b.rate;
    }
    if value == CONFIG.image_process.fill_args.text_c.text{
        return rate-CONFIG.image_process.fill_args.text_c.rate;
    }
    if value == CONFIG.image_process.fill_args.text_d.text{
        return rate-CONFIG.image_process.fill_args.text_d.rate;
    }
    return rate;
}