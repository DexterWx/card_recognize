use image::{DynamicImage, GenericImageView, GrayImage, Luma};
use imageproc::filter::{gaussian_blur_f32,median_filter};
use imageproc::morphology::{erode, dilate};
use imageproc::distance_transform::{distance_transform, Norm};
use imageproc::contours::find_contours;
use imageproc::contours::Contour;
use image::{ImageBuffer, Rgb, RgbImage};

use crate::my_utils::image::*;
use crate::models::card::MyPoint;

/// 输入图片路径，输出摆正图和四个定位点
pub fn process_img(inimg: &str) -> (RgbImage, [MyPoint;4]){
    // 读取图像文件
    let img = image::open(inimg).expect("Failed to open image file");
    // 将图像转换为灰度图像
    let gray_img = img.to_luma8();

    // 对灰度图像进行高斯模糊
    let mut blurred_img = gaussian_blur_f32(&gray_img, 1.0);

    // 对模糊后的图像进行二值化
    blurred_img.enumerate_pixels_mut().for_each(|(_, _, pixel)| {
        if pixel[0] > 180 {
            *pixel = Luma([255u8]);
        } else {
            *pixel = Luma([0u8]);
        }
    });

    // 膨胀操作
    let dilated_img = dilate(&blurred_img, Norm::LInf, 5);

    // 腐蚀操作
    let eroded_img = erode(&dilated_img, Norm::LInf, 5);

    // 保存结果
    eroded_img.save("output.jpg").expect("Failed to save image");

    // 查找图像中的轮廓
    let contours: Vec<Contour<i32>> = find_contours(&eroded_img);

    // 寻找四个定位点 x+y足够小、x-y足够大、x-y足够小、x+y足够大
    let mut lt = MyPoint{x:111111,y:111111};
    let mut rt = MyPoint{x:-111111,y:111111};
    let mut ld = MyPoint{x:111111,y:-111111};
    let mut rd = MyPoint{x:-111111,y:-111111};

    for contour in contours.iter(){
        let a = calculate_points_center(&contour.points);
        match a {
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
    
    use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
    use image::Rgb;

    let angle_radians1 = (rt.y as f32 - lt.y as f32).atan2(rt.x as f32 - lt.x as f32);
    let angle_radians2 = (ld.y as f32 - lt.y as f32).atan2(ld.x as f32 - lt.x as f32);

    // 对图像进行旋转
    let rotated_img = rotate_about_center(&img.to_rgb8(), -angle_radians1, Interpolation::Bilinear, Rgb([255,255,255]));
    // 保存结果
    rotated_img.save("output_rotate.jpg").expect("Failed to save image");
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