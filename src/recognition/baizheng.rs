/// 完成图片和scanjson的页匹配+图片摆正
use std::cmp::max;
use std::cmp::min;
use std::f32::consts::PI;
use std::collections::HashMap;

use anyhow::Result;
use image::ImageBuffer;
use image::Luma;
use image::Rgb;
use imageproc::contours::find_contours;
use imageproc::contours::Contour;
use imageproc::drawing::draw_filled_circle_mut;
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::integral_image::integral_image;
use imageproc::integral_image::sum_image_pixels;
use imageproc::rect::Rect;


use crate::models::engine_rec::LocationInfo;
use crate::models::engine_rec::ProcessedImages;
use crate::models::engine_rec::ReferenceModelPoints;
use crate::models::engine_rec::{ProcessedImagesAndModelPoints, RecInfoBaizheng};
use crate::models::my_error::MyError;
use crate::models::rec_result::ImageStatus;
use crate::models::rec_result::MoveOperation;
use crate::models::rec_result::OutputRec;
use crate::models::rec_result::PageSize;
use crate::models::scan_json::AssistPoint;
use crate::models::scan_json::InputImage;
use crate::models::scan_json::PageNumberPoint;
use crate::models::scan_json::Coordinate;
use crate::my_utils::image::*;
use crate::models::card::MyPoint;
use crate::my_utils::math::cal_segment_angle;
use crate::my_utils::math::points4_is_valid;
use crate::my_utils::math::{cosine_similarity, euclidean_distance};
use crate::config::CONFIG;

use super::engine::Engine;

pub trait Baizheng{
    fn baizheng_and_match_page(&self, input_images: &InputImage, output: &mut OutputRec) -> Vec<Option<ProcessedImagesAndModelPoints>>;
    fn set_assist_points(&self, imgs_and_model_points: &Vec<Option<ProcessedImagesAndModelPoints>>, output: &mut OutputRec);
    fn rendering_model_points(&self, imgs_and_model_points: &mut Vec<Option<ProcessedImagesAndModelPoints>>, output: &mut OutputRec);
    fn rendering_assist_points(&self, imgs_and_model_points: &mut Vec<Option<ProcessedImagesAndModelPoints>>, output: &mut OutputRec);
}


impl Baizheng for Engine {
    /// 输出对应page位置的图片并摆正，未匹配的使用None
    fn baizheng_and_match_page(&self, input_images: &InputImage, output: &mut OutputRec) -> Vec<Option<ProcessedImagesAndModelPoints>>{
        // 读图+处理成ProcessedImages，包含各种预处理的图片
        let mut imgs: Vec<ProcessedImages> = Vec::new();
        for base64_image in &input_images.images{
            let img = process_image(&self.get_scan_data().pages[0].model_size, base64_image);
            let mean_pixel = sum_image_pixels(
                &img.integral_gray, 0, 0, img.morphology.width()-1, img.morphology.height()-1
            )[0]/((img.morphology.width() * img.morphology.height()) as i64);
            if mean_pixel > CONFIG.image_process.empty_image_threshold as i64{continue;}
            imgs.push(img);
        }
        // 计算每张图片真实定位点
        // 并根据定位点进行小角度摆正
        // 将img和定位点组成后续公用的图结构ProcessedImagesAndModelPoints
        let mut imgs_and_model_points = Vec::new();

        // 定位点信息
        let location_wh = (
            self.get_scan_data().pages[0].model_points[0].coordinate.w,
            self.get_scan_data().pages[0].model_points[0].coordinate.h,
        );
        let location_info = LocationInfo::new(location_wh, self.get_scan_data().is_in_seal);
        for img in imgs.iter_mut(){
            let coordinates = generate_location_and_rotate(img, &location_info);
            match coordinates{
                // 找到定位点
                Some(coordinates) => {
                    imgs_and_model_points.push(
                        ProcessedImagesAndModelPoints{
                            img: img.clone(),
                            real_model_points: coordinates,
                        }
                    );
                }
                // 如果寻找定位点失败，直接把图片置为失败状态
                None => {
                    let _base64 = img.org.as_ref().expect("org is None");
                    let image_status = ImageStatus {
                        image_source: _base64.clone(),
                        code: 1
                    };
                    output.images.push(image_status);
                }
            };
        }

        // 初始化匹配成功的标记
        let mut is_match_dict: HashMap<usize, bool> = HashMap::new();
        for i in 0..imgs_and_model_points.len() {
            is_match_dict.insert(i, false);
        }

        // 遍历scan的每个page，从所有图结构里匹配页码差异符合要求的
        // todo：目前设定了一个差异度阈值，符合后就不做后续匹配了，后期可以加入差异排名做进一步判断
        let scan_size = self.get_scan_data().pages.len();
        let mut processed_images_res: Vec<Option<ProcessedImagesAndModelPoints>> = vec![None;scan_size];

        // 遍历第一遍原方向的图片
        for (index_scan,page) in self.get_scan_data().pages.iter().enumerate(){
            for (index_image, img_and_model_points) in imgs_and_model_points.iter().enumerate(){
                let match_info = RecInfoBaizheng{
                    model_size: &page.model_size,
                    page_number_points: &page.page_number_points,
                    model_points: page.model_points_4.as_ref().expect("model_points_4 is None")
                };
                let diff = match_page_and_img(&match_info, &img_and_model_points);

                if diff <= CONFIG.image_baizheng.page_number_diff{
                    let img_and_model_points = img_and_model_points.clone();
                    processed_images_res[index_scan] = Some(img_and_model_points.clone());
                    is_match_dict.insert(index_image, true);
                    break
                }
            }
        }

        // 挑选第一遍中没有匹配上的图翻转180
        // 并把第一遍成功匹配的图片写到输出中
        let mut imgs_and_model_points_180 = Vec::new();
        for (index, flag) in is_match_dict.iter() {
            if *flag {
                let _base64 = imgs_and_model_points[*index].img.org.as_ref().expect("org is None");
                let image_status = ImageStatus {
                    image_source: _base64.clone(),
                    code: 0
                };
                output.images.push(image_status);
                continue
            }
            let mut img_180 = imgs_and_model_points[*index].clone();
            rotate_img_and_model_points_180(&mut img_180);
            imgs_and_model_points_180.push(img_180);
        }

        // 初始化匹配成功的标记
        let mut is_match_dict: HashMap<usize, bool> = HashMap::new();
        for i in 0..imgs_and_model_points_180.len() {
            is_match_dict.insert(i, false);
        }

        // 第二次遍历翻转180的剩余图片
        for (index_scan,page) in self.get_scan_data().pages.iter().enumerate(){
            if !matches!(processed_images_res[index_scan],None){continue;}
            for (index_image, img_and_model_points) in imgs_and_model_points_180.iter().enumerate(){
                let match_info = RecInfoBaizheng{
                    model_size: &page.model_size,
                    page_number_points: &page.page_number_points,
                    model_points: page.model_points_4.as_ref().expect("model_points_4 is None")
                };
                let diff = match_page_and_img(&match_info, &img_and_model_points);
                if diff <= CONFIG.image_baizheng.page_number_diff{
                    let img_and_model_points = img_and_model_points.clone();
                    processed_images_res[index_scan] = Some(img_and_model_points.clone());
                    is_match_dict.insert(index_image, true);
                    break
                }
            }
        }
        // 把剩余的180翻转图片匹配情况写入结果
        for (index, flag) in is_match_dict.iter() {
            let _base64 = imgs_and_model_points_180[*index].img.org.as_ref().expect("org is None");
            let image_status = ImageStatus {
                image_source: _base64.clone(),
                code: if *flag { 0 } else { 1 }
            };
            output.images.push(image_status);
        }

        // 精细修复定位点轮廓
        for processed_image_and_points in processed_images_res.iter_mut() {
            if matches!(processed_image_and_points,None){continue;}
            let processed_image_and_points = processed_image_and_points.as_mut().unwrap();
            let img = &mut processed_image_and_points.img;
            let coors = &mut processed_image_and_points.real_model_points;
            fix_model_points_coordinate(img, coors, CONFIG.image_baizheng.model_point_scan_range);
            rotate_img_and_model_points(img, coors);
        }

        // 查看精细修复后的匹配率
        #[cfg(debug_assertions)]
        {
            for (index_scan,page) in self.get_scan_data().pages.iter().enumerate(){
                if matches!(processed_images_res[index_scan],None){continue;}
                let img_and_model_points = processed_images_res[index_scan].as_ref().unwrap();
                let match_info = RecInfoBaizheng{
                    model_size: &page.model_size,
                    page_number_points: &page.page_number_points,
                    model_points: page.model_points_4.as_ref().expect("model_points_4 is None")
                };
                let _ = match_page_and_img(&match_info, img_and_model_points);
            }
        }


        processed_images_res
    }

    fn rendering_model_points(&self, imgs_and_model_points: &mut Vec<Option<ProcessedImagesAndModelPoints>>, output: &mut OutputRec){
        for (_index,(img_and_model_points, page)) in imgs_and_model_points.iter().zip(output.pages.iter_mut()).enumerate(){
            if matches!(img_and_model_points, None){continue;}
            let rendering = trans_base64_to_image(&page.image_rotated.as_ref().expect("image_rendering is None"));
            let mut rendering = rendering.to_rgb8();
            for point in img_and_model_points.as_ref().unwrap().real_model_points.iter(){
                draw_filled_circle_mut(&mut rendering,(point.x,point.y),3, Rgb([0,0,255]));
                draw_filled_circle_mut(&mut rendering,(point.x+point.w,point.y+point.h),3, Rgb([0,0,255]));
            }
            let img_base64 = image_to_base64(&rendering);
            page.image_rendering = Some(img_base64);
        }
    }

    fn rendering_assist_points(&self, imgs_and_model_points: &mut Vec<Option<ProcessedImagesAndModelPoints>>, output: &mut OutputRec){
        for (page,(img_and_model_points,page_out)) in self.get_scan_data().pages.iter().zip(imgs_and_model_points.iter().zip(output.pages.iter_mut())){
            if matches!(img_and_model_points,None) {continue;}
            if matches!(page.assist_points, None) {continue;}
            if matches!(page_out.image_rendering, None){continue;}
            let rendering = trans_base64_to_image(&page_out.image_rendering.as_ref().expect("image_rendering is None"));
            let mut rendering = rendering.to_rgb8();
            let img_and_model_points = img_and_model_points.as_ref().expect("img_and_model_points is None");

            let reference_model_points = ReferenceModelPoints{
                model_points: &page.model_points_4.expect("model_points_4 is None"),
                real_model_points: &img_and_model_points.real_model_points
            };
            let assist_points = page.assist_points.as_ref().unwrap();
            let fix_assist_points = page_out.assist_points.as_ref().unwrap();
            for (point, fix_point) in assist_points.iter().zip(fix_assist_points.iter()){
                let left_coor = generate_real_coordinate_with_model_points(&reference_model_points, &point.left);
                let right_coor = generate_real_coordinate_with_model_points(&reference_model_points, &point.right);
                draw_filled_rect_mut(
                    &mut rendering,
                    Rect::at(left_coor.x, left_coor.y).of_size(left_coor.w as u32, left_coor.h as u32),
                    Rgb([255u8, 0u8, 0u8]),
                );

                draw_filled_rect_mut(
                    &mut rendering,
                    Rect::at(right_coor.x, right_coor.y).of_size(right_coor.w as u32, right_coor.h as u32),
                    Rgb([255u8, 0u8, 0u8]),
                );

                draw_filled_rect_mut(
                    &mut rendering,
                    Rect::at(fix_point.left.x, fix_point.left.y).of_size(fix_point.left.w as u32, fix_point.left.h as u32),
                    Rgb([0u8, 0u8, 255u8]),
                );

                draw_filled_rect_mut(
                    &mut rendering,
                    Rect::at(fix_point.right.x, fix_point.right.y).of_size(fix_point.right.w as u32, fix_point.right.h as u32),
                    Rgb([0u8, 0u8, 255u8]),
                );
            }
            let img_base64 = image_to_base64(&rendering);
            page_out.image_rendering = Some(img_base64);
        }
    }

    fn set_assist_points(&self, imgs_and_model_points: &Vec<Option<ProcessedImagesAndModelPoints>>, output: &mut OutputRec){
        for (page, (img_and_model_points, out_page)) in self.get_scan_data().pages.iter().zip(
            imgs_and_model_points.iter().zip(
                output.pages.iter_mut()
            )
        ){
            if page.assist_points.is_none(){continue;}
            if img_and_model_points.is_none(){continue;}
            let assist_points = page.assist_points.as_ref().unwrap();
            let img_and_model_points = img_and_model_points.as_ref().unwrap();
            let reference_model_points = ReferenceModelPoints{
                model_points: &page.model_points_4.expect("model_points_4 is None"),
                real_model_points: &img_and_model_points.real_model_points
            };
            let move_hash = &mut out_page.assist_points_move_op;
            let mut fix_assist_points:Vec<AssistPoint> = Vec::new();
            for point in assist_points.iter(){
                if point.left.y != point.right.y {continue;}
                let left_coor = generate_real_coordinate_with_model_points(&reference_model_points, &point.left);
                let right_coor = generate_real_coordinate_with_model_points(&reference_model_points, &point.right);
                let mut fix_left_coor = left_coor.clone();
                let mut fix_right_coor = right_coor.clone();
                fix_coordinate_by_search_nearby(&img_and_model_points.img, &mut fix_left_coor, CONFIG.image_baizheng.assist_point_nearby_length);
                fix_coordinate_by_search_nearby(&img_and_model_points.img, &mut fix_right_coor, CONFIG.image_baizheng.assist_point_nearby_length);
                fix_coordinate(&img_and_model_points.img, &mut fix_left_coor, CONFIG.image_baizheng.assist_point_scan_range, CONFIG.image_baizheng.assist_point_min_distance, CONFIG.image_baizheng.assist_point_max_distance);
                fix_coordinate(&img_and_model_points.img, &mut fix_right_coor, CONFIG.image_baizheng.assist_point_scan_range, CONFIG.image_baizheng.assist_point_min_distance, CONFIG.image_baizheng.assist_point_max_distance);
                fix_coordinate_by_search_nearby(&img_and_model_points.img, &mut fix_left_coor, CONFIG.image_baizheng.assist_point_nearby_length);
                fix_coordinate_by_search_nearby(&img_and_model_points.img, &mut fix_right_coor, CONFIG.image_baizheng.assist_point_nearby_length);
                fix_coordinate(&img_and_model_points.img, &mut fix_left_coor, CONFIG.image_baizheng.assist_point_scan_range, CONFIG.image_baizheng.assist_point_min_distance, CONFIG.image_baizheng.assist_point_max_distance);
                fix_coordinate(&img_and_model_points.img, &mut fix_right_coor, CONFIG.image_baizheng.assist_point_scan_range, CONFIG.image_baizheng.assist_point_min_distance, CONFIG.image_baizheng.assist_point_max_distance);

                let move_op = generate_move_op([left_coor,right_coor], [fix_left_coor, fix_right_coor]);
                move_hash.insert(point.left.y, move_op);
                fix_assist_points.push(
                    AssistPoint{
                        left: fix_left_coor.clone(),
                        right: fix_right_coor.clone(),
                    }
                );
            }
            out_page.assist_points = Some(fix_assist_points);
        }
    }
}


// 根据定位点计算偏转角度
// todo: 如果答题卡被折过，这种方法会有误差。
// 后面可以增加一种对定位点位置的判断，猜测纸张是否可能被折过
// 如果被折过，使用左侧两点后右侧两点分别对办张图片摆正，两边的框分开定位。
pub fn rotate_img_and_model_points(img: &mut ProcessedImages, mut coors: &mut [Coordinate;4]){

    let angle_radians1 = (coors[1].y as f32 - coors[0].y as f32).atan2(coors[1].x as f32 - coors[0].x as f32);
    let angle_radian = -angle_radians1;

    // 旋转之前保存中心点
    let center = MyPoint { x: coors[0].x as i32, y: coors[0].y as i32 };

    // 对图像进行旋转
    rotate_processed_image(img, &center, angle_radian);

    // 对定位点进行旋转
    rotate_model_points(&mut coors, &center, angle_radian);
}


fn generate_location_and_rotate(img: &mut ProcessedImages, location_info: &LocationInfo) -> Option<[Coordinate;4]>{
    for args in CONFIG.image_process.retry_args.iter(){
        let img_mor: ImageBuffer<Luma<u8>, Vec<u8>> = generate_mophology_from_blur(&img.blur, args);
        let model_points = generate_location(&img_mor, location_info);
        if model_points.is_none(){continue};
        let mut model_points = model_points.unwrap();
        rotate_img_and_model_points(img, &mut model_points);

        #[cfg(debug_assertions)]
        {
            debug_rendering_process_image(img, &model_points);
        }

        return Some(model_points);
    }
    None
}

/// 靠图片寻找定位点并进行小角度摆正
/// 输出四个定位点并小角度摆正输入的图片
fn generate_location(img: &ImageBuffer<Luma<u8>, Vec<u8>>, location_info: &LocationInfo) -> Option<[Coordinate;4]>{

    // 查找图像中的轮廓
    let contours: Vec<Contour<i32>> = find_contours(img);

    // 寻找四个定位点 x+y足够小、x-y足够大、x-y足够小、x+y足够大
    let mut lt = Coordinate{x:111111,y:111111,w:0,h:0};
    let mut rt = Coordinate{x:-111111,y:111111,w:0,h:0};
    let mut ld = Coordinate{x:111111,y:-111111,w:0,h:0};
    let mut rd = Coordinate{x:-111111,y:-111111,w:0,h:0};

    for contour in contours.iter(){
        let [lt_box, rt_box, ld_box] = calculate_points_lt_rt_ld(&contour.points).expect("Calculate 3 Points Failed");
        let w = euclidean_distance((lt_box.x as f32,lt_box.y as f32), (rt_box.x as f32,rt_box.y as f32)) as i32;
        let h = euclidean_distance((lt_box.x as f32,lt_box.y as f32), (ld_box.x as f32,ld_box.y as f32)) as i32;
        
        // 过滤定位点宽高大小不符合的
        if w<CONFIG.image_baizheng.model_point_min_wh || h<CONFIG.image_baizheng.model_point_min_wh{
            continue;
        }
        if w>CONFIG.image_baizheng.model_point_max_wh || h>CONFIG.image_baizheng.model_point_max_wh{
            continue;
        }

        // 过滤影响定位点选择的框框，余弦相似度如果不够大说明不是定位点。
        let cos = cosine_similarity(&vec![w as f32,h as f32], &vec![location_info.wh.0 as f32, location_info.wh.1 as f32]);
        if CONFIG.image_baizheng.model_point_wh_cosine_similarity > cos{
            continue
        }

        let x = lt_box.x;
        let y = lt_box.y;

        if x+y<lt.x+lt.y{
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
    
    if !points4_is_valid(
        [
            (lt.x,lt.y),
            (rt.x,rt.y),
            (ld.x,ld.y),
            (rd.x,rd.y),
        ]
    ) {
        println!("找到的4个定位点不符合要求 {:?}",[lt,rt,ld,rd]);
        return None;
    }

    Some([lt,rt,ld,rd])
}

fn rotate_model_points(points: &mut [Coordinate;4], center: &MyPoint, angle_radian: f32){
    for i in 0..points.len() {
        let point = &mut points[i];
        let (new_x, new_y) = rotate_point(&MyPoint { x: point.x, y: point.y }, &center, angle_radian);
        point.x = new_x;
        point.y = new_y;
    }
}

fn fix_model_points_coordinate(img: &ProcessedImages, coordinates: &mut [Coordinate; 4], scan_range: i32){
    for coor in coordinates.iter_mut(){
        fix_coordinate(img, coor, scan_range, CONFIG.image_baizheng.model_point_min_distance,CONFIG.image_baizheng.model_point_max_distance);
    }
}

fn fix_coordinate(img: &ProcessedImages, coordinate: &mut Coordinate, scan_range: i32, min_dis: i32, max_dis: i32){
    fix_boundary_top_down(img, coordinate, scan_range, min_dis, max_dis);
    fix_boundary_left_right(img, coordinate, scan_range, min_dis, max_dis);
}

fn fix_boundary_top_down(img: &ProcessedImages, coordinate: &mut Coordinate, scan_range: i32, min_dis: i32, max_dis: i32){
    let top = max(coordinate.y - scan_range,0) as u32;
    let bottom = min(coordinate.y + coordinate.h + scan_range, img.rgb.height() as i32) as u32;
    let left = coordinate.x as u32;
    let right = (coordinate.x + coordinate.w) as u32;
    let mut min_decrease = 0;
    let mut max_increase = 0;
    let mut _y = coordinate.y;
    let mut _yh = coordinate.y + coordinate.h;
    for i in top+1..bottom{
        let current = sum_image_pixels(
            &img.integral_gray, left, i, right, i
        )[0];
        let before = sum_image_pixels(
            &img.integral_gray, left, i-1, right, i-1
        )[0];
        let diff = current - before;
        if diff < min_decrease && i <= (top+bottom)*2/3 {min_decrease = diff;_y = i as i32;}
        if diff > max_increase && i >= (top+bottom)/3 {max_increase = diff;_yh = (i-1) as i32;}
    }
    if _yh - _y > min_dis && _yh - _y < max_dis
    {
        coordinate.y = _y;
        coordinate.h = _yh - _y
    };
}

fn fix_boundary_left_right(img: &ProcessedImages, coordinate: &mut Coordinate, scan_range: i32, min_dis: i32, max_dis: i32){
    let left = max(coordinate.x - scan_range,0) as u32;
    let right = min(coordinate.x + coordinate.w + scan_range, img.rgb.width() as i32) as u32;
    let top = coordinate.y as u32;
    let bottom = (coordinate.y + coordinate.h) as u32;
    let mut min_decrease = 0;
    let mut max_increase = 0;
    let mut _x = coordinate.x;
    let mut _xw = coordinate.x + coordinate.w;
    for i in left+1..right{
        let current = sum_image_pixels(
            &img.integral_gray, i, top, i, bottom
        )[0];
        let before = sum_image_pixels(
            &img.integral_gray, i-1, top, i-1, bottom
        )[0];
        let diff = current - before;
        if diff < min_decrease && i <= (left+right)*2/3 {min_decrease = diff;_x = i as i32;}
        if diff > max_increase && i >= (left+right)/3 {max_increase = diff;_xw = (i-1) as i32;}
    }
    if _xw - _x > min_dis && _xw - _x < max_dis
    {
        coordinate.x = _x;
        coordinate.w = _xw - _x
    };
}


fn fix_coordinate_by_search_nearby(img: &ProcessedImages, coordinate: &mut Coordinate, nearby_length: i32){
    let directions = [
        (0,0),
        (0,1),
        (1,1),
        (1,0),
        (1,-1),
        (0,-1),
        (-1,-1),
        (-1,0),
        (-1,1),
    ];
    let mut mean_pix = 255;
    let mut direction = (0,0);
    for dir in directions.iter(){
        let _mean_pix = sum_image_pixels(
            &img.integral_gray,
            (coordinate.x + dir.0 * nearby_length) as u32,
            (coordinate.y + dir.1 * nearby_length) as u32,
            (coordinate.x + dir.0 * nearby_length + coordinate.w - 1) as u32,
            (coordinate.y + dir.1 * nearby_length + coordinate.h - 1) as u32
        )[0] as i32 / (coordinate.w * coordinate.h);
        if _mean_pix < mean_pix{direction.0 = dir.0;direction.1 = dir.1;mean_pix = _mean_pix;}
    }
    coordinate.x += direction.0 * nearby_length;
    coordinate.y += direction.1 * nearby_length;
}

/// 输入的图片已经是经过小角度摆正+90度摆正的图片
/// 该函数根据页面点的向量距离对page和image进行匹配
/// 匹配成功的img直接进行180大角度摆正
fn match_page_and_img(
    baizheng_info: &RecInfoBaizheng, img_and_model_points: &ProcessedImagesAndModelPoints
) -> f32 {

    // 输入图片可能是需要180翻转的，根据真实页码点填涂率和标注页码点填涂率的距离确定
    let diff = calculate_page_img_diff(
        baizheng_info.page_number_points,
        &ReferenceModelPoints{
            model_points: baizheng_info.model_points,
            real_model_points: &img_and_model_points.real_model_points,
        },
        &img_and_model_points.img.integral_morphology
    );

    // 距离足够小说明匹配成功
    #[cfg(debug_assertions)]
    {
        println!("{diff}");
    }
    return diff;
}

/// 翻转180,和普通旋转逻辑不通，因为涉及定位点的位置调换
fn rotate_img_and_model_points_180(
    img_and_model_points: &mut ProcessedImagesAndModelPoints
){
    // 翻转中心
    let center = MyPoint{
        x: (img_and_model_points.img.rgb.width() / 2) as i32,
        y: (img_and_model_points.img.rgb.height() / 2) as i32,
    };

    // 0，1，2，3对应左上，右上，左下，右下
    // 0旋转180翻到3，1旋转180放到2，2旋转180放到1，3旋转180放到0
    let real_point_0 = img_and_model_points.real_model_points[0];
    let real_point_1 = img_and_model_points.real_model_points[1];
    let real_point_2 = img_and_model_points.real_model_points[2];
    let real_point_3 = img_and_model_points.real_model_points[3];

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

    img_and_model_points.real_model_points[0] = Coordinate{x:x0,y:y0,w:w0,h:h0};
    img_and_model_points.real_model_points[1] = Coordinate{x:x1,y:y1,w:w1,h:h1};
    img_and_model_points.real_model_points[2] = Coordinate{x:x2,y:y2,w:w2,h:h2};
    img_and_model_points.real_model_points[3] = Coordinate{x:x3,y:y3,w:w3,h:h3};

    rotate_processed_image(&mut img_and_model_points.img, &center, PI);

}



/// 计算page和img的差异
fn calculate_page_img_diff(
    page_number_points: &Vec<PageNumberPoint>,
    reference_model_points: &ReferenceModelPoints,
    img: &ImageBuffer<Luma<i64>, Vec<i64>>,
) -> f32 {
    // 获取标注的pagenumber填涂向量
    let mut page_number_fill_rates = Vec::new();
    let mut real_page_number_coordinates: Vec<Coordinate> = Vec::new();
    for page_number in page_number_points{
        let real_coordinate = generate_real_coordinate_with_model_points(
            reference_model_points,
            &page_number.coordinate
        );
        page_number_fill_rates.push(page_number.fill_rate);
        real_page_number_coordinates.push(real_coordinate);
    }
    // 计算距离
    let difference = calculate_page_number_difference(img, &real_page_number_coordinates, &page_number_fill_rates);
    difference
}

fn generate_move_op(old_points: [Coordinate;2], new_points: [Coordinate;2]) -> MoveOperation{
    let move_x = new_points[0].x - old_points[0].x;
    let move_y = new_points[0].y - old_points[0].y;
    let center = MyPoint{x: new_points[0].x, y: new_points[0].y};
    let angle = cal_segment_angle(
        MyPoint::new(old_points[0].x, old_points[0].y),
        MyPoint::new(old_points[1].x, old_points[1].y),
        MyPoint::new(new_points[0].x, new_points[0].y),
        MyPoint::new(new_points[1].x, new_points[1].y),
    );
    MoveOperation{
        move_x: move_x,
        move_y: move_y,
        center: center,
        angle: angle
    }
}

pub fn fix_coordinate_use_assist_points(coordinate: &mut Coordinate, move_op: &Option<&MoveOperation>){
    if move_op.is_none(){return;}
    let move_op = move_op.unwrap();
    coordinate.x += move_op.move_x;
    coordinate.y += move_op.move_y;
    let new_point = rotate_point(
        &MyPoint::new(coordinate.x, coordinate.y), &move_op.center, move_op.angle
    );
    coordinate.x = new_point.0;
    coordinate.y = new_point.1;
}


fn debug_rendering_process_image(img: &mut ProcessedImages, model_points: &[Coordinate;4]){
    let mut rendering = img.rgb.clone();
    for coor in model_points.iter(){
        draw_filled_circle_mut(&mut rendering,(coor.x,coor.y),3, Rgb([0,0,255]));
        draw_filled_circle_mut(&mut rendering,(coor.x + coor.w,coor.y+coor.h),3, Rgb([0,0,255]));
    }
    let path_model_point = format!("dev/test_data/debug_model_points.jpg");
    let _ = rendering.save(path_model_point);

    let path_morphology = format!("dev/test_data/debug_path_morphology.jpg");
    let _ = img.morphology.save(path_morphology);

    let path_gray = format!("dev/test_data/debug_path_gray.jpg");
    let _ = img.blur.save(path_gray);
}