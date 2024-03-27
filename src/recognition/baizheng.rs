use std::f32::consts::PI;

use image::{DynamicImage, GenericImageView, GrayImage, ImageBuffer, Luma, Rgb, RgbImage, Rgba};
use imageproc::drawing::{draw_filled_circle, draw_filled_circle_mut};
use imageproc::filter::{gaussian_blur_f32,median_filter};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use imageproc::morphology::{erode, dilate};
use imageproc::distance_transform::{distance_transform, Norm};
use imageproc::contours::find_contours;
use imageproc::contours::Contour;
use imageproc::rect::Rect;

use crate::models::engine_rec::{ProcessedImages, RecInfoBaizheng};
use crate::models::scan_json::{Coordinate, ModelSize};
use crate::my_utils::image::*;
use crate::models::card::MyPoint;
use crate::my_utils::io::compatible_path_format;
use crate::recognition::engine::Engine;
use crate::config::CONFIG;


// pub trait Baizheng{
//     /// 输入的图片已经是经过小角度摆正的图片
//     /// 该函数根据页码点进行大角度摆正
//     fn rotate_with_page_number<T, D>(&self, toinfo: T, img: &DynamicImage) -> D;
// }

// impl Baizheng for Engine {
//     fn rotate_with_page_number<T, D>(&self, toinfo: T, img: &DynamicImage) -> D {
        
//         unimplemented!()
//     }
// }



/// 输入DynamicImage图片
/// 不需要scan信息，纯靠图片寻找定位点并进行小角度摆正
/// 输出四个定位点并小角度摆正输入的图片
pub fn rotate_with_location(imgs: &mut ProcessedImages) -> [MyPoint;4]{
   
    // 查找图像中的轮廓
    let contours: Vec<Contour<i32>> = find_contours(&imgs.morphology);

    #[cfg(debug_assertions)]
    println!("找到的框的数量：{}", contours.len());

    // 寻找四个定位点 x+y足够小、x-y足够大、x-y足够小、x+y足够大
    let mut lt = MyPoint{x:111111,y:111111};
    let mut rt = MyPoint{x:-111111,y:111111};
    let mut ld = MyPoint{x:111111,y:-111111};
    let mut rd = MyPoint{x:-111111,y:-111111};

    for contour in contours.iter(){
        let center = calculate_points_lt(&contour.points);
        match center {
            Some((x,y)) => {
                if x+y<lt.x+lt.y {
                    lt.x = x;
                    lt.y = y;
                }
                if x-y>rt.x-rt.y {
                    rt.x = x;
                    rt.y = y;
                }
                if x-y<ld.x-ld.y {
                    ld.x = x;
                    ld.y = y;
                }
                if x+y>rd.x+rd.y {
                    rd.x = x;
                    rd.y = y;
                }
            }
            None => {
                continue;
            }
        }
    }

    // 根据定位点计算偏转角度
    let angle_radians1 = (rt.y as f32 - lt.y as f32).atan2(rt.x as f32 - lt.x as f32);
    let angle_radians2 = (ld.y as f32 - lt.y as f32).atan2(ld.x as f32 - lt.x as f32);

    // 旋转之前保存中心点
    let center = MyPoint{x:(imgs.rgb.width()/2) as i32, y:(imgs.rgb.height()/2) as i32};

    // 对图像进行旋转
    rotate_processed_image(imgs, -angle_radians1);

    #[cfg(debug_assertions)]
    imgs.morphology.save("dev/test_data/output_mor.jpg").expect("Failed to save image");

    // 对定位点进行旋转
    let mut points: [MyPoint;4] = [MyPoint{x:0,y:0};4];
    for (i,point) in [lt, rt, ld, rd].iter().enumerate(){
        let (new_x, new_y) = rotate_point(point, &center, -angle_radians1);
        points[i] = MyPoint{x:new_x,y:new_y};
    }
    points

}


/// 根据wh比例绝对是否对图片进行90度旋转
pub fn rotate_processed_image_90(model_size: &ModelSize, imgs: &mut ProcessedImages){
    // 如果标注的长宽大小和图片的长宽大小关系不同，说明图片需要90度偏转
    let flag_need_90 = (model_size.h > model_size.w) != (imgs.rgb.height() > imgs.rgb.width());
    if flag_need_90{
        rotate_processed_image(imgs, PI/2.0);
    }
}

/// 输入的图片已经是经过小角度摆正+90度摆正的图片
/// 该函数根据页码点进行180大角度摆正
pub fn rotate_with_page_number(baizheng_info: &mut RecInfoBaizheng, imgs: &mut ProcessedImages){
    // 对比当前图片页码点匹配率和旋转180后页码点匹配率，选择更大匹配率作为图片的最终摆正
    // 第一次获取真实页码点框
    let mut real_page_number_coordinates: Vec<Coordinate> = Vec::new();
    let mut page_number_fill_rates = Vec::new();
    for page_number in &baizheng_info.page_number_points{
        let real_coordinate = generate_real_coordinate_with_model_points(
            &baizheng_info.reference_model_points, &page_number.coordinate
        );
        page_number_fill_rates.push(page_number.fill_rate);
        real_page_number_coordinates.push(real_coordinate);
        
    }
    let first_difference = calculate_page_number_difference(&imgs.integral_morphology, &real_page_number_coordinates, &page_number_fill_rates);
    println!("{first_difference}");

    if first_difference <= 0.2{
        return;
    }

    let center = MyPoint{
        x: (imgs.rgb.width() / 2) as i32,
        y: (imgs.rgb.height() / 2) as i32,
    };

    let (x0,y0) = rotate_point(
        &baizheng_info.reference_model_points.real_model_points[3], &center, PI,
    );
    let (x1,y1) = rotate_point(
        &baizheng_info.reference_model_points.real_model_points[2], &center, PI,
    );
    let (x2,y2) = rotate_point(
        &baizheng_info.reference_model_points.real_model_points[1], &center, PI,
    );
    let (x3,y3) = rotate_point(
        &baizheng_info.reference_model_points.real_model_points[0], &center, PI,
    );
    baizheng_info.reference_model_points.real_model_points[0] = MyPoint{x:x0,y:y0};
    baizheng_info.reference_model_points.real_model_points[1] = MyPoint{x:x1,y:y1};
    baizheng_info.reference_model_points.real_model_points[2] = MyPoint{x:x2,y:y2};
    baizheng_info.reference_model_points.real_model_points[3] = MyPoint{x:x3,y:y3};

    rotate_processed_image(imgs, PI);


    // let real_model_points: Vec<MyPoint> = Vec::new();
    // rotate_processed_image(imgs, PI);

}