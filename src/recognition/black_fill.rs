use std::collections::HashMap;

use crate::config::CONFIG;
use crate::models::engine_rec::ReferenceModelPoints;
use crate::models::rec_result::{MoveOperation, OutputEnum, RecOption, Recognize};
use crate::models::scan_json::{PageEnum, Recognition, Value};
use crate::my_utils::math::get_otsu;
use crate::{models::{engine_rec::ProcessedImages, rec_result::OutputRec, scan_json::Coordinate}, recognition::engine::Engine};
use crate::my_utils::image::*;
use image::imageops::crop_imm;
use image::{ImageBuffer, Luma, Pixel};
use image::Rgb;
use imageproc::contrast::threshold;
use imageproc::rect::Rect;
use imageproc::integral_image::sum_image_pixels;
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use ab_glyph::FontArc;
use imageproc::stats::{histogram, ChannelHistogram};

use super::baizheng::fix_coordinate_use_assist_points;

pub trait RecBlackFill {
    /// 填涂识别
    fn rec_black_fill(img: &ProcessedImages, coordinate: &Coordinate, value: &Option<Value>) -> f32;
    fn rec_black_fill_options(img: &ProcessedImages, reference_model_points: &Option<&ReferenceModelPoints>, rec: &Recognition,out_rec: &mut Recognize, move_ops: &Option<&HashMap<i32, MoveOperation>>, total_otsu: u8, neighborhood_size: u8);
    fn binary_fill_rate(output: &mut OutputEnum);
    fn rendering_black_fill(output: &mut OutputRec);
    fn rendering_black_fill_show_rate(output: &mut OutputRec);
    fn otsu_from_total_fill(img: &ProcessedImages, reference_model_points: &Option<&ReferenceModelPoints>, page: PageEnum, move_ops: &Option<&HashMap<i32, MoveOperation>>) -> (u8, f64);
}

impl RecBlackFill for Engine {
    fn otsu_from_total_fill(img: &ProcessedImages, reference_model_points: &Option<&ReferenceModelPoints>, page: PageEnum, move_ops: &Option<&HashMap<i32, MoveOperation>>) -> (u8, f64){
        let mut combined_hist = ChannelHistogram {
            channels: vec![[0u32; 256]; Luma::<u8>::CHANNEL_COUNT as usize],
        };
        let recs;
        match page {
            PageEnum::Page(page) => {
                recs = &page.recognizes;
            },
            PageEnum::PageSecond(page_second) => {
                recs = &page_second.recognizes;
            },
        }
        for rec in recs.iter(){
            if rec.rec_type != CONFIG.recognize_type.single_select
                && rec.rec_type != CONFIG.recognize_type.multi_select
                && rec.rec_type != CONFIG.recognize_type.exam_number {continue}
            for option in rec.options.iter() {
                let mut real_coordinate = option.coordinate.clone();
                if !reference_model_points.is_none() && !move_ops.is_none() {
                    let reference_model_points = reference_model_points.unwrap();
                    let move_ops = move_ops.unwrap();
                    real_coordinate = generate_real_coordinate_with_model_points(
                        &reference_model_points, &option.coordinate, true, None
                    );
                    fix_coordinate_use_assist_points(&mut real_coordinate, &move_ops.get(&option.coordinate.y));
                }
                let crop_gray = crop_imm(
                    &img.blur,
                    real_coordinate.x as u32,
                    real_coordinate.y as u32,
                    real_coordinate.w as u32,
                    real_coordinate.h as u32,
                ).to_image();
                let hist = histogram(&crop_gray);
                combined_hist = add_histograms(&combined_hist, &hist);
            }
        }
        let (otsu,var) = otsu_level_and_var_from_hist(&combined_hist);
        (otsu,var)
    }
    
    fn rec_black_fill_options(
        img: &ProcessedImages,
        reference_model_points: &Option<&ReferenceModelPoints>,
        rec: &Recognition,
        out_rec: &mut Recognize,
        move_ops: &Option<&HashMap<i32, MoveOperation>>,
        total_otsu: u8,
        neighborhood_size: u8,
    ) {

        let mut _best_otsu_255 = 0;
        let mut best_var = 0f64;
        let mut min_var = 1111111111f64;
        let mut min_otsu = 0u8;
        let mut best_back_weight = 0f64;
        let mut best_fill_rate_100_for_otsu = Vec::new();
        let mut best_coors = Vec::new();
        //跨度向下取整
        let space = (neighborhood_size / 2) as i32;
        // 遍历x轴方向从x-2到x+2
        for i in -space..=space {
            // 遍历y轴方向从y-2到y+2
            for j in -space..=space {
                let mut combined_hist = ChannelHistogram {
                    channels: vec![[0u32; 256]; Luma::<u8>::CHANNEL_COUNT as usize],
                };
                for option in rec.options.iter(){
                    let mut real_coordinate = option.coordinate.clone();
                    if !reference_model_points.is_none() && !move_ops.is_none() {
                        let reference_model_points = reference_model_points.unwrap();
                        let move_ops = move_ops.unwrap();
                        real_coordinate = generate_real_coordinate_with_model_points(
                            &reference_model_points, &option.coordinate, true, None
                        );
                        fix_coordinate_use_assist_points(&mut real_coordinate, &move_ops.get(&option.coordinate.y));
                    }
                    real_coordinate.x+=i;
                    real_coordinate.y+=j;
                    let _option_value = &option.value;
                    let crop_gray = crop_imm(
                        &img.blur,
                        real_coordinate.x as u32,
                        real_coordinate.y as u32,
                        real_coordinate.w as u32,
                        real_coordinate.h as u32,
                    ).to_image();
                    let hist = histogram(&crop_gray);
                    combined_hist = add_histograms(&combined_hist, &hist);
                }
                let (otsu_255,_) = otsu_level_and_var_from_hist(&combined_hist);

                let mut fill_rate_100_for_otsu = Vec::new();
                let mut coors = Vec::new();
                
                for option in rec.options.iter(){
                    let mut real_coordinate = option.coordinate.clone();
                    if !reference_model_points.is_none() && !move_ops.is_none() {
                        let reference_model_points = reference_model_points.unwrap();
                        let move_ops = move_ops.unwrap();
                        real_coordinate = generate_real_coordinate_with_model_points(
                            &reference_model_points, &option.coordinate, true, None
                        );
                        fix_coordinate_use_assist_points(&mut real_coordinate, &move_ops.get(&option.coordinate.y));
                    }
                    real_coordinate.x+=i;
                    real_coordinate.y+=j;
                    let crop_gray = crop_imm(
                        &img.blur,
                        real_coordinate.x as u32,
                        real_coordinate.y as u32,
                        real_coordinate.w as u32,
                        real_coordinate.h as u32,
                    ).to_image();
                    let crop_bi = threshold(&crop_gray, otsu_255);
                    let sum_pix: u32 = crop_bi.iter().map(|&x| x as u32).sum();
                    let pix_count = crop_bi.width() * crop_bi.height();
                    let fill_rate;
                    if pix_count == 0{
                        fill_rate = 0f32;
                    }
                    else{
                        fill_rate = 1f32 - (sum_pix / pix_count) as f32 / 255f32;
                    }
                    fill_rate_100_for_otsu.push(
                        (fill_rate * 100f32) as u8
                    );
                    coors.push(real_coordinate);
                }
                let (_, var, back_weight) = get_otsu(&fill_rate_100_for_otsu, CONFIG.image_process.fill_args.otsu_black_fill_sep_weight);
                if var >= best_var {
                    _best_otsu_255 = otsu_255;
                    best_var = var; 
                    best_fill_rate_100_for_otsu = fill_rate_100_for_otsu;
                    best_coors = coors;
                }
                if var < min_var{min_var = var;min_otsu=otsu_255;best_back_weight=back_weight}
            }
        }

        // 针对全涂和全不涂做一个特殊处理
        // 整体逻辑是根据9*9范围内获得的最小类间方差判定是否全涂或者全不涂，方差小说明差异小
        // 在差异很小的情况下判定是全涂或者全不涂
        // 然后使用全局otus重新计算填涂率

        // 对非常确定的全涂case做提前处理
        // 因为9*9的使用最大类间方差作为停止搜索的规则不利于全涂case
        // 最终得到的坐标框是基于最大类间方差的，即便使用了全局otsu重新计算
        // 仍然会造成全涂case的类间方差偏大，导致后续题内阈值卡不住最大类间方差。
        // 所以根据badcase获得的最小类间方差和对应的otsu提前卡住非常确定的全填图case，手动修改填涂率
        for ((option,rate),coor) in out_rec.rec_options.iter_mut().zip(best_fill_rate_100_for_otsu.iter()).zip(best_coors.iter()){
            
            // 步骤1：非常确认的全涂case手动修改填涂率为1
            if min_var < CONFIG.image_process.fill_args.all_fill_var
                && min_otsu < CONFIG.image_process.fill_args.all_fill_otsu {
                    option.value = Some(Value::Float(1f32));
                    option._value = Some(Value::Float(1f32));
            }
            // 步骤2: 对全涂或者全不涂使用全局otsu重新计算fillrate
            else if min_var < CONFIG.image_process.fill_args.all_fill_or_empty_min_var{
                let crop_gray = crop_imm(
                    &img.blur,
                    coor.x as u32,
                    coor.y as u32,
                    coor.w as u32,
                    coor.h as u32,
                ).to_image();
                let crop_bi = threshold(&crop_gray, total_otsu);
                let sum_pix: u32 = crop_bi.iter().map(|&x| x as u32).sum();
                let pix_count = crop_bi.width() * crop_bi.height();
                let fill_rate;
                if pix_count == 0{
                    fill_rate = 0f32;
                }
                else{
                    fill_rate = 1f32 - (sum_pix / pix_count) as f32 / 255f32;
                }
                option.value = Some(Value::Float(fill_rate));
                option._value = Some(Value::Float(fill_rate));
            }
            else {
                option.value = Some(Value::Float(*rate as f32/100f32));
                option._value = Some(Value::Float(*rate as f32/100f32));
            }
            #[cfg(debug_assertions)]
            {
                option.coordinate = Some(*coor);
            }
        }
        #[cfg(debug_assertions)]
        {
            println!("min_var: {min_var:?}\tmin_otsu: {min_otsu:?}\tback_weight: {best_back_weight:?}\ttotal_otsu: {total_otsu:?}");
        }

    }
   
    fn binary_fill_rate(output: &mut OutputEnum) {
        match output {
            OutputEnum::OutputRec(output) => {
                for page in output.pages.iter_mut(){
                    if !page.has_page {continue}
                    for rec in page.recognizes.iter_mut(){
                        if rec.rec_type==CONFIG.recognize_type.single_select {
                            set_single_fill_rate(&mut rec.rec_options, CONFIG.image_process.fill_args.same_var);
                        }
                        if rec.rec_type==CONFIG.recognize_type.exam_number {
                            set_single_fill_rate(&mut rec.rec_options, CONFIG.image_process.fill_args.same_var_exam_number);
                        }
                        if rec.rec_type==CONFIG.recognize_type.black_fill {
                            set_single_fill_rate(&mut rec.rec_options, CONFIG.image_process.fill_args.same_var);
                        }
                        if rec.rec_type==CONFIG.recognize_type.multi_select {
                            set_multi_fill_rate(&mut rec.rec_options)
                        }
                    }
                }
            },
            OutputEnum::OutputRecSecond(output_second) => {
                for page in output_second.pages.iter_mut(){
                    for rec in page.recognizes.iter_mut(){
                        if rec.rec_type==CONFIG.recognize_type.single_select {
                            set_single_fill_rate(&mut rec.rec_options, CONFIG.image_process.fill_args.same_var);
                        }
                        if rec.rec_type==CONFIG.recognize_type.exam_number {
                            set_single_fill_rate(&mut rec.rec_options, CONFIG.image_process.fill_args.same_var_exam_number);
                        }
                        if rec.rec_type==CONFIG.recognize_type.black_fill {
                            set_single_fill_rate(&mut rec.rec_options, CONFIG.image_process.fill_args.same_var);
                        }
                        if rec.rec_type==CONFIG.recognize_type.multi_select {
                            set_multi_fill_rate(&mut rec.rec_options)
                        }
                    }
                }
            },
        }
        
    }
    fn rec_black_fill(img: &ProcessedImages, coordinate: &Coordinate, value: &Option<Value>) -> f32 {
        let rect = Rect::at(coordinate.x, coordinate.y).of_size(coordinate.w as u32, coordinate.h as u32);
        let integral_image;
        if CONFIG.image_blackfill.image_type == 0 {
            integral_image = &img.integral_gray;
        } else {
            integral_image = &img.integral_morphology;
        }
        //计算摆正后原始区域填涂率
        let mut filled_ratio = calculate_fill_ratio(integral_image, rect);
        //计算制定区域最大值，默认搜索5*5范围内最大
        // let mut filled_ratio = find_max_fillrate_in_neighborhood(integral_image, coordinate, filled_ratio);
        //调整不同选项的选项的基础阈值，微调填涂率
        if !value.is_none(){
            filled_ratio = finetune_rate(filled_ratio, value.as_ref().unwrap());
        }

        filled_ratio = filled_ratio.min(1f32).max(0f32);  
        filled_ratio
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
                if recognize.rec_type == CONFIG.recognize_type.black_fill
                || recognize.rec_type == CONFIG.recognize_type.multi_select
                || recognize.rec_type == CONFIG.recognize_type.single_select
                || recognize.rec_type == CONFIG.recognize_type.exam_number
                {
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
                if rec.rec_type != CONFIG.recognize_type.black_fill
                && rec.rec_type != CONFIG.recognize_type.multi_select
                && rec.rec_type != CONFIG.recognize_type.single_select
                && rec.rec_type != CONFIG.recognize_type.exam_number{continue}
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

fn set_single_fill_rate(options: &mut Vec<RecOption>, same_var: f64){

    if options.len() == 0 {return}
    let fill_rates_u8 = get_array_values_for_otsu(options);
    let (mut best_threshold, var,_) = get_otsu(&fill_rates_u8,CONFIG.image_process.fill_args.otsu_black_fill_sep_weight);

    if var < same_var {
        
        if *fill_rates_u8.iter().max().unwrap() > CONFIG.image_process.fill_args.fill_same_max{
            best_threshold = 0;
        }
        if *fill_rates_u8.iter().max().unwrap() < CONFIG.image_process.fill_args.empty_same_max {
            best_threshold = 100;
        }
    }
    
    #[cfg(debug_assertions)]
    {
        println!("{fill_rates_u8:?}  {best_threshold:?} {var:?}");
    }
    set_filled_use_threshold(options, best_threshold);

}



fn set_multi_fill_rate(options: &mut Vec<RecOption>){

    if options.len() == 0 {return}
    let fill_rates_u8 = get_array_values_for_otsu(options);
    let (mut best_threshold, var,_) = get_otsu(&fill_rates_u8,CONFIG.image_process.fill_args.otsu_black_fill_sep_weight);
    
    if var < CONFIG.image_process.fill_args.same_var {
        
        if *fill_rates_u8.iter().max().unwrap() > CONFIG.image_process.fill_args.fill_same_max{
            best_threshold = 0;
        }
        if *fill_rates_u8.iter().max().unwrap() < CONFIG.image_process.fill_args.empty_same_max {
            best_threshold = 100;
        }
    }

    #[cfg(debug_assertions)]
    {
        println!("{fill_rates_u8:?}  {best_threshold:?} {var:?}");
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