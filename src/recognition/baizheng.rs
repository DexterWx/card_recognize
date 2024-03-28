use std::f32::consts::PI;

use imageproc::contours::find_contours;
use imageproc::contours::Contour;

use crate::models::engine_rec::{ProcessedImages, RecInfoBaizheng};
use crate::models::scan_json::{Coordinate, ModelSize};
use crate::my_utils::image::*;
use crate::models::card::MyPoint;
use crate::my_utils::math::{cosine_similarity, euclidean_distance};
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




/// 靠图片寻找定位点并进行小角度摆正
/// 输出四个定位点并小角度摆正输入的图片
pub fn rotate_with_location(imgs: &mut ProcessedImages, location_wh: (i32, i32)) -> [Coordinate;4]{
   
    // 查找图像中的轮廓
    let contours: Vec<Contour<i32>> = find_contours(&imgs.morphology);

    // 寻找四个定位点 x+y足够小、x-y足够大、x-y足够小、x+y足够大
    let mut lt = Coordinate{x:111111,y:111111,w:0,h:0};
    let mut rt = Coordinate{x:-111111,y:111111,w:0,h:0};
    let mut ld = Coordinate{x:111111,y:-111111,w:0,h:0};
    let mut rd = Coordinate{x:-111111,y:-111111,w:0,h:0};

    for contour in contours.iter(){
        let [lt_box, rt_box, ld_box] = calculate_points_lt_rt_ld(&contour.points).unwrap();
        let w = euclidean_distance((lt_box.x as f32,lt_box.y as f32), (rt_box.x as f32,rt_box.y as f32)) as i32;
        let h = euclidean_distance((lt_box.x as f32,lt_box.y as f32), (ld_box.x as f32,ld_box.y as f32)) as i32;
        if CONFIG.image_baizheng.model_point_wh_cosine_similarity > cosine_similarity(&vec![w as f32,h as f32], &vec![location_wh.0 as f32, location_wh.1 as f32]) {
            continue
        }
        let x = lt_box.x;
        let y = lt_box.y;
        
        if x+y<lt.x+lt.y {
            lt.x = x;
            lt.y = y;
            lt.w = w;
            lt.h = h;
        }
        if x-y>rt.x-rt.y {
            rt.x = x;
            rt.y = y;
            rt.w = w;
            rt.h = h;
        }
        if x-y<ld.x-ld.y {
            ld.x = x;
            ld.y = y;
            ld.w = w;
            ld.h = h;
        }
        if x+y>rd.x+rd.y {
            rd.x = x;
            rd.y = y;
            rd.w = w;
            rd.h = h;
        }
    }

    // 根据定位点计算偏转角度
    let angle_radians1 = (rt.y as f32 - lt.y as f32).atan2(rt.x as f32 - lt.x as f32);
    // let angle_radians2 = (ld.y as f32 - lt.y as f32).atan2(ld.x as f32 - lt.x as f32);

    // 旋转之前保存中心点
    let center = MyPoint{x:(imgs.rgb.width()/2) as i32, y:(imgs.rgb.height()/2) as i32};

    // 对图像进行旋转
    rotate_processed_image(imgs, -angle_radians1);

    // 对定位点进行旋转
    let mut points: [Coordinate;4] = [Coordinate{x:0,y:0,w:0,h:0};4];
    for (i,point) in [lt, rt, ld, rd].iter().enumerate(){
        let (new_x, new_y) = rotate_point(&MyPoint{x:point.x,y:point.y}, &center, -angle_radians1);
        points[i] = Coordinate{x:new_x,y:new_y,w:point.w,h:point.h};
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
    // 输入图片可能是需要180翻转的，根据真实页码点填涂率和标注页码点填涂率的距离确定
    // 获取真实页码点框
    let mut real_page_number_coordinates: Vec<Coordinate> = Vec::new();
    let mut page_number_fill_rates = Vec::new();
    for page_number in &baizheng_info.page_number_points{
        let real_coordinate = generate_real_coordinate_with_model_points(
            &baizheng_info.reference_model_points, &page_number.coordinate
        );
        page_number_fill_rates.push(page_number.fill_rate);
        real_page_number_coordinates.push(real_coordinate);
        
    }
    // 计算距离
    let difference = calculate_page_number_difference(&imgs.integral_morphology, &real_page_number_coordinates, &page_number_fill_rates);
    // 距离足够小说明不需要180翻转
    if difference <= CONFIG.image_baizheng.page_number_diff{
        return;
    }

    // 以下距离超过阈值的情况下
    // 需要把图片和之前找到的真实定位点都反转180

    // 翻转中心
    let center = MyPoint{
        x: (imgs.rgb.width() / 2) as i32,
        y: (imgs.rgb.height() / 2) as i32,
    };

    // 0，1，2，3对应左上，右上，左下，右下
    // 0旋转180翻到3，1旋转180放到2，2旋转180放到1，3旋转180放到0
    let real_point_0 = baizheng_info.reference_model_points.real_model_points[0];
    let real_point_1 = baizheng_info.reference_model_points.real_model_points[1];
    let real_point_2 = baizheng_info.reference_model_points.real_model_points[2];
    let real_point_3 = baizheng_info.reference_model_points.real_model_points[3];

    // 要翻转的不是每个定位点的左上xy坐标，而是右下的x+w,y+h
    // 因为180之后每个定位点的右下变成了左上
    let ((x0,y0), w0, h0) = (rotate_point(
        &MyPoint{
            x:real_point_3.x + real_point_3.w,
            y:real_point_3.y + real_point_3.h,
        },
        &center, PI,
    ),real_point_3.w, real_point_3.h);

    let ((x1,y1), w1, h1) = (rotate_point(
        &MyPoint{
            x:real_point_2.x + real_point_2.w,
            y:real_point_2.y + real_point_2.h,
        },
        &center, PI,
    ), real_point_2.w,real_point_2.h);

    let ((x2,y2), w2, h2) = (rotate_point(
        &MyPoint{
            x:real_point_1.x + real_point_1.w,
            y:real_point_1.y + real_point_1.h,
        },
        &center, PI,
    ), real_point_1.w, real_point_1.h);

    let ((x3,y3), w3, h3) = (rotate_point(
        &MyPoint{
            x:real_point_0.x + real_point_0.w,
            y:real_point_0.y + real_point_0.h,
        },
        &center, PI,
    ), real_point_0.w, real_point_0.h);

    baizheng_info.reference_model_points.real_model_points[0] = Coordinate{x:x0,y:y0,w:w0,h:h0};
    baizheng_info.reference_model_points.real_model_points[1] = Coordinate{x:x1,y:y1,w:w1,h:h1};
    baizheng_info.reference_model_points.real_model_points[2] = Coordinate{x:x2,y:y2,w:w2,h:h2};
    baizheng_info.reference_model_points.real_model_points[3] = Coordinate{x:x3,y:y3,w:w3,h:h3};

    // 图片旋转180
    rotate_processed_image(imgs, PI);
}