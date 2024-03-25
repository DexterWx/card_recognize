use std::f32::consts::PI;

use image::{DynamicImage, GenericImageView, GrayImage, Luma};
use imageproc::filter::{gaussian_blur_f32,median_filter};
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use imageproc::morphology::{erode, dilate};
use imageproc::distance_transform::{distance_transform, Norm};
use imageproc::contours::find_contours;
use imageproc::contours::Contour;
use image::{ImageBuffer, Rgb, RgbImage};

use crate::models::engine_rec::RecInfoBaizheng;
use crate::my_utils::image::*;
use crate::models::card::MyPoint;
use crate::config::CONFIG;

/// 输入图片路径，输出小角度摆正图和四个定位点
pub fn process_img(inimg: &str) -> (RgbImage, [MyPoint;4]){
    // 读取图像文件
    let img = image::open(inimg).expect("Failed to open image file");
    // 将图像转换为灰度图像
    let gray_img = img.to_luma8();

    // 对灰度图像进行高斯模糊
    let mut blurred_img = gaussian_blur_f32(&gray_img, CONFIG.image_process.gaussian_blur_sigma);

    // 对模糊后的图像进行二值化
    blurred_img.enumerate_pixels_mut().for_each(|(_, _, pixel)| {
        if pixel[0] > CONFIG.image_process.binarization_threshold {
            *pixel = Luma([255u8]);
        } else {
            *pixel = Luma([0u8]);
        }
    });

    // 膨胀操作
    let dilated_img = dilate(&blurred_img, Norm::LInf, CONFIG.image_process.morphology_kernel);

    // 腐蚀操作
    let eroded_img = erode(&dilated_img, Norm::LInf, CONFIG.image_process.morphology_kernel);

    // 保存结果
    #[cfg(debug_assertions)]
    eroded_img.save("dev/test_data/output.jpg").expect("Failed to save image");

    // 查找图像中的轮廓
    let contours: Vec<Contour<i32>> = find_contours(&eroded_img);

    #[cfg(debug_assertions)]
    println!("找到的框的数量：{}", contours.len());

    // 寻找四个定位点 x+y足够小、x-y足够大、x-y足够小、x+y足够大
    let mut lt = MyPoint{x:111111,y:111111};
    let mut rt = MyPoint{x:-111111,y:111111};
    let mut ld = MyPoint{x:111111,y:-111111};
    let mut rd = MyPoint{x:-111111,y:-111111};

    for contour in contours.iter(){
        let center = calculate_points_center(&contour.points);
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

    let angle_radians1 = (rt.y as f32 - lt.y as f32).atan2(rt.x as f32 - lt.x as f32);
    let angle_radians2 = (ld.y as f32 - lt.y as f32).atan2(ld.x as f32 - lt.x as f32);

    // 对图像进行旋转
    let rotated_img = rotate_about_center(&img.to_rgb8(), -angle_radians1, Interpolation::Bilinear, Rgb([255,255,255]));
    // 保存结果
    #[cfg(debug_assertions)]
    rotated_img.save("dev/test_data/output_rotate.jpg").expect("Failed to save image");
    // 旋转定位点
    let center = MyPoint{x:(img.width()/2) as i32, y:(img.height()/2) as i32};

    let (new_x, new_y) = rotate_point(lt, &center, -angle_radians1);
    let lt = MyPoint{x:new_x, y:new_y};

    let (new_x, new_y) = rotate_point(rt, &center, -angle_radians1);
    let rt = MyPoint{x:new_x, y:new_y};

    let (new_x, new_y) = rotate_point(ld, &center, -angle_radians1);
    let ld = MyPoint{x:new_x, y:new_y};

    let (new_x, new_y) = rotate_point(rd, &center, -angle_radians1);
    let rd = MyPoint{x:new_x, y:new_y};

    (rotated_img,[lt,rt,ld,rd])
}

/// 输入的图片已经是经过小角度摆正的图片
/// 该函数根据页码点进行大角度摆正
pub fn rotate_with_page_number(baizheng_info: &RecInfoBaizheng, img: &RgbImage){
    // 如果标注的长宽大小和图片的长宽大小关系不同，说明图片需要90度偏转
    let flag_need_90 = (baizheng_info.model_size.h > baizheng_info.model_size.w) != (img.height() > img.width());
    if flag_need_90{
        let rotated_img = rotate_about_center(img, PI/2.0, Interpolation::Bilinear, Rgb([255,255,255]));
    
    
    }
    

}