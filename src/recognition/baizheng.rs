/// 完成图片和scanjson的页匹配+图片摆正

use std::f32::consts::PI;

use image::ImageBuffer;
use image::Luma;
use imageproc::contours::find_contours;
use imageproc::contours::Contour;

use crate::models::engine_rec::ProcessedImages;
use crate::models::engine_rec::ReferenceModelPoints;
use crate::models::engine_rec::{ProcessedImagesAndModelPoints, RecInfoBaizheng};
use crate::models::scan_json::InputImage;
use crate::models::scan_json::PageNumberPoint;
use crate::models::scan_json::{Coordinate, ModelSize};
use crate::my_utils::image::*;
use crate::models::card::MyPoint;
use crate::my_utils::math::{cosine_similarity, euclidean_distance};
use crate::config::CONFIG;
use crate::my_utils::node::print2node;

use super::engine::Engine;

pub trait Baizheng{
    fn baizheng_and_match_page(&self, input_images: &InputImage) -> Vec<Option<ProcessedImagesAndModelPoints>>;
}


impl Baizheng for Engine {
    /// 输入的图片已经是经过小角度摆正的图片
    /// 输出对应page位置的图片，未匹配的使用None
    fn baizheng_and_match_page(&self, input_images: &InputImage) -> Vec<Option<ProcessedImagesAndModelPoints>>{
        // 读图+处理成ProcessedImages，包含各种预处理的图片
        let mut imgs: Vec<ProcessedImages> = Vec::new();
        for base64_image in &input_images.images{
            let img = process_image(&self.get_scan_data().pages[0].model_size, base64_image);
            imgs.push(img);
        }
        // 获取定位点wh，用于筛选定位点
        // todo: 后期可以抽象一下，目前只想到这一个
        let location_wh = (
            self.get_scan_data().pages[0].model_points[0].coordinate.w,
            self.get_scan_data().pages[0].model_points[0].coordinate.h,
        );
        // 计算每张图片真实定位点
        // 并根据定位点进行小角度摆正
        // 将img和定位点组成后续公用的图结构ProcessedImagesAndModelPoints
        let mut imgs_and_model_points = Vec::new();
        for mut img in imgs{
            let coordinates = generate_location_and_rotate(&mut img, location_wh);
            imgs_and_model_points.push(
                ProcessedImagesAndModelPoints{
                    img: img,
                    real_model_points: coordinates,
                }
            );
        }
        // 生成每个图结构的旋转180副本
        let mut imgs_and_model_points_contains_180 = Vec::new();
        for img in imgs_and_model_points{
            let mut img_180 = img.clone();
            rotate_img_and_model_points_180(&mut img_180);
            imgs_and_model_points_contains_180.push(img);
            imgs_and_model_points_contains_180.push(img_180);
        }

        // 遍历scan的每个page，从所有图结构里匹配页码差异符合要求的
        // todo：目前设定了一个差异度阈值，符合后就不做后续匹配了，后期可以加入差异排名做进一步判断
        let scan_size = self.get_scan_data().pages.len();
        let mut processed_images_res: Vec<Option<ProcessedImagesAndModelPoints>> = vec![None;scan_size];

        for (index,page) in self.get_scan_data().pages.iter().enumerate(){
            for img_and_model_points in &imgs_and_model_points_contains_180{
                let match_info = RecInfoBaizheng{
                    model_size: &page.model_size,
                    page_number_points: &page.page_number_points,
                    model_points: page.model_points_4.as_ref().expect("model_points_4 is None")
                };
                let flag = match_page_and_img(&match_info, &img_and_model_points);
                if flag{
                    processed_images_res[index] = Some(img_and_model_points.clone());
                    break
                }
            }
        }
        processed_images_res
    }
}

/// 根据wh比例决定是否对图片进行90度旋转
pub fn rotate_processed_image_90(model_size: &ModelSize, img: &mut ProcessedImages){
    // 如果标注的长宽大小和图片的长宽大小关系不同，说明图片需要90度偏转
    let flag_need_90 = (model_size.h > model_size.w) != (img.rgb.height() > img.rgb.width());
    if flag_need_90{
        rotate_processed_image(img, PI/2.0);
    }
}



/// 靠图片寻找定位点并进行小角度摆正
/// 输出四个定位点并小角度摆正输入的图片
fn generate_location_and_rotate(img: &mut ProcessedImages, location_wh: (i32, i32)) -> [Coordinate;4]{
    // todo: 定位点过滤补丁，后面需要优化
    let w = img.rgb.width();
    let lt_x_must_less = ((w as f32) / (4 as f32)) as i32;
    let rd_x_must_more = ((w as f32) / (4 as f32) * 3.0) as i32;

    // 查找图像中的轮廓
    let contours: Vec<Contour<i32>> = find_contours(&img.morphology);

    // 寻找四个定位点 x+y足够小、x-y足够大、x-y足够小、x+y足够大
    // todo：有的答题卡的考号在左上角会影响寻找左上角定位点
    let mut lt = Coordinate{x:111111,y:111111,w:0,h:0};
    let mut rt = Coordinate{x:-111111,y:111111,w:0,h:0};
    let mut ld = Coordinate{x:111111,y:-111111,w:0,h:0};
    let mut rd = Coordinate{x:-111111,y:-111111,w:0,h:0};

    for contour in contours.iter(){
        let [lt_box, rt_box, ld_box] = calculate_points_lt_rt_ld(&contour.points).unwrap();
        let w = euclidean_distance((lt_box.x as f32,lt_box.y as f32), (rt_box.x as f32,rt_box.y as f32)) as i32;
        let h = euclidean_distance((lt_box.x as f32,lt_box.y as f32), (ld_box.x as f32,ld_box.y as f32)) as i32;
        // 过滤影响定位点选择的框框，余弦相似度如果不够大说明不是定位点。
        if CONFIG.image_baizheng.model_point_wh_cosine_similarity > cosine_similarity(&vec![w as f32,h as f32], &vec![location_wh.0 as f32, location_wh.1 as f32]) {
            continue
        }
        let x = lt_box.x;
        let y = lt_box.y;
        
        // 因为左上和右下定位点会受到考号影响，所以加一些限制
        // 左上的y一定要是全局最小可以过滤掉考号
        // x在1/4内可以过滤第一行中其他定位点的干扰
        // 因为图片有可能是180旋转的，所以右下同理
        if y<lt.y && x<lt_x_must_less{
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
        if y>rd.y && x>rd_x_must_more {
            rd.x = x;
            rd.y = y;
            rd.w = w;
            rd.h = h;
        }
    }

    // println!("{lt:?}");
    // println!("{rt:?}");
    // println!("{ld:?}");
    // println!("{rd:?}");
    // let mut image = img.rgb.clone();
    // draw_filled_circle_mut(&mut image, (lt.x,lt.y), 5, Rgb([0,0,255]));
    // draw_filled_circle_mut(&mut image, (rt.x,rt.y), 5, Rgb([0,0,255]));
    // draw_filled_circle_mut(&mut image, (ld.x,ld.y), 5, Rgb([0,0,255]));
    // draw_filled_circle_mut(&mut image, (rd.x,rd.y), 5, Rgb([0,0,255]));
    // image.save("dev/test_data/output_view_location.jpg");

    // 根据定位点计算偏转角度
    // todo: 如果答题卡被折过，这种方法会有误差。
    // 后面可以增加一种对定位点位置的判断，猜测纸张是否可能被折过
    // 如果被折过，使用左侧两点后右侧两点分别对办张图片摆正，两边的框分开定位。
    let angle_radians1 = (rt.y as f32 - lt.y as f32).atan2(rt.x as f32 - lt.x as f32);
    // let angle_radians2 = (ld.y as f32 - lt.y as f32).atan2(ld.x as f32 - lt.x as f32);

    // 旋转之前保存中心点
    let center = MyPoint{x:(img.rgb.width()/2) as i32, y:(img.rgb.height()/2) as i32};

    // 对图像进行旋转
    rotate_processed_image(img, -angle_radians1);

    // 对定位点进行旋转
    let mut points: [Coordinate;4] = [Coordinate{x:0,y:0,w:0,h:0};4];
    for (i,point) in [lt, rt, ld, rd].iter().enumerate(){
        let (new_x, new_y) = rotate_point(&MyPoint{x:point.x,y:point.y}, &center, -angle_radians1);
        points[i] = Coordinate{x:new_x,y:new_y,w:point.w,h:point.h};
    }
    points
}


/// 输入的图片已经是经过小角度摆正+90度摆正的图片
/// 该函数根据页面点的向量距离对page和image进行匹配
/// 匹配成功的img直接进行180大角度摆正
fn match_page_and_img(
    baizheng_info: &RecInfoBaizheng, img_and_model_points: &ProcessedImagesAndModelPoints
) -> bool {
    
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
    // todo：匹配率作为报错信息返回
    if diff <= CONFIG.image_baizheng.page_number_diff{
        return true;
    }

    return false;
}

/// 翻转180,会修改原图片,需要提前clone
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

    // 图片旋转180
    rotate_processed_image(&mut img_and_model_points.img, PI);

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
