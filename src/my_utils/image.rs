use std::io::Cursor;

use image::{DynamicImage, ImageBuffer, ImageFormat, Luma, Rgb, RgbImage};
use imageproc::distance_transform::Norm;
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use imageproc::morphology::{dilate, erode};
use imageproc::{filter::gaussian_blur_f32, point::Point};
use imageproc::integral_image::{integral_image, sum_image_pixels};

use crate::models::engine_rec::{ProcessedImages, ReferenceModelPoints};
use crate::models::scan_json::ModelSize;
use crate::{config::CONFIG, models::{card::MyPoint, scan_json::Coordinate}};
use super::math::*;
use image_base64_wasm::from_base64;
use image_base64_wasm::vec_to_base64;

pub trait HasCoordinates<T> {
    fn get_coordinates(&self) -> (&T, &T);
}
// 定义一个宏来为多个类型实现 HasCoordinates trait
macro_rules! impl_has_coordinates {
    ($($type:ty),*) => {
        $(impl<T> HasCoordinates<T> for $type {
            fn get_coordinates(&self) -> (&T, &T) {
                (&self.x, &self.y)
            }
        })*
    };
}
impl_has_coordinates!(Point<T>);

/// 计算一组点的中心点
pub fn calculate_points_center<T, K>(points: &[T]) -> Option<(i32, i32)>
where
    T: HasCoordinates<K>,
    K: Default + Copy + Into<i32> + std::ops::Add<Output = K> + std::ops::Div<Output = K>,
{
    if points.is_empty() {
        return None;
    }

    // 初始化中心点的坐标
    let mut center_x = K::default();
    let mut center_y = K::default();

    // 计算所有点的坐标总和
    for point in points {
        let (x, y) = point.get_coordinates();
        center_x = center_x + *x;
        center_y = center_y + *y;
    }

    // 将坐标总和除以点的数量，得到中心点的坐标
    let num_points = points.len() as i32;
    // let num_points_k = K::from(num_points);
    let center_x: i32 = center_x.into();
    let center_y: i32 = center_y.into();
    let center_x = center_x / num_points;
    let center_y = center_y / num_points;

    Some((center_x, center_y))
}


/// 计算一组点的左上,右上,左下
pub fn calculate_points_lt_rt_ld<T, K>(points: &[T]) -> Option<[MyPoint;3]>
where
    T: HasCoordinates<K>,
    K: Default + Copy + Into<i32> + From<i32>,
{
    if points.is_empty() {
        return None;
    }

    // 初始化三点的坐标
    let mut lt_x = i32::default();
    let mut lt_y = i32::default();

    let mut rt_x = i32::default();
    let mut rt_y = i32::default();

    let mut ld_x = i32::default();
    let mut ld_y = i32::default();

    let mut min_x_add_y = 111111 as i32;
    let mut max_x_sub_y = -111111 as i32;
    let mut min_x_sub_y = 111111 as i32;

    // 找到三个最值点
    for point in points {
        let (x, y) = point.get_coordinates();
        let x:i32 = (*x).into();
        let y:i32 = (*y).into();
        if x + y < min_x_add_y {
            lt_x = x;
            lt_y = y;
            min_x_add_y = x+y;
        }
        if x - y > max_x_sub_y {
            rt_x = x;
            rt_y = y;
            max_x_sub_y = x-y;
        }
        if x - y < min_x_sub_y {
            ld_x = x;
            ld_y = y;
            min_x_sub_y = x-y;
        }
    }

    Some(
        [
            MyPoint{x:lt_x,y:lt_y},
            MyPoint{x:rt_x,y:rt_y},
            MyPoint{x:ld_x,y:ld_y},
        ]
    )
}

/// 计算一组矩形边缘点的宽高
pub fn calculate_points_wh<T, K>(points: &[T]) -> Option<(i32, i32)>
where
    T: HasCoordinates<K>,
    K: Default + Copy + Into<i32> + From<i32> + std::cmp::PartialOrd,
{
    if points.is_empty() {
        return None;
    }

    let mut minx:K = K::from(11111);
    let mut maxx:K = K::from(0);
    let mut miny:K = K::from(11111);
    let mut maxy:K = K::from(0);
    for point in points{
        let (x, y) = point.get_coordinates();
        if *x>maxx{
            maxx = *x;
        }
        if *x<minx{
            minx = *x;
        }
        if *y>maxy{
            maxy = *y;
        }
        if *y<miny{
            miny = *y;
        }

    }
    let maxx:i32= maxx.into();
    let maxy:i32= maxy.into();
    let minx:i32= minx.into();
    let miny:i32= miny.into();

    Some((maxx-minx, maxy-miny))
}

/// 根据给定的中心点center按角度angle_rad顺时针旋转
pub fn rotate_point(point: &MyPoint, center: &MyPoint, angle_rad: f32) -> (i32, i32)
{
    let cos_theta:f32 = angle_rad.cos();
    let sin_theta:f32 = angle_rad.sin();

    let x_diff = point.x - center.x;
    let y_diff = point.y - center.y;
    let rotated_x = (center.x as f32) + (x_diff as f32) * cos_theta - (y_diff as f32) * sin_theta;
    let rotated_y = (center.y as f32) + (x_diff as f32) * sin_theta + (y_diff as f32) * cos_theta;

    (rotated_x as i32, rotated_y as i32)
}

/// 参照定位点得到标注coodinate对应的真实coordinate
pub fn generate_real_coordinate_with_model_points(reference_model_points: &ReferenceModelPoints, coordinate: &Coordinate) -> Coordinate{
    let model_points = &reference_model_points.model_points;
    let real_model_points = &reference_model_points.real_model_points;
    let x_rate = ((real_model_points[0].x - real_model_points[1].x) as f32) / ((model_points[0].coordinate.x - model_points[1].coordinate.x) as f32);
    let y_rate = ((real_model_points[0].y - real_model_points[2].y) as f32) / ((model_points[0].coordinate.y - model_points[2].coordinate.y) as f32);

    let real_w = x_rate * (coordinate.w as f32);
    let real_h = y_rate * (coordinate.h as f32);

    let real_x = x_rate * (coordinate.x - model_points[0].coordinate.x) as f32 + real_model_points[0].x as f32;
    let real_y = y_rate * (coordinate.y - model_points[0].coordinate.y) as f32 + real_model_points[0].y as f32;

    // let real_x = real_x_center - (model_points[0].coordinate.w as f32 * real_w)/2.0;
    // let real_y = real_y_center - (model_points[0].coordinate.h as f32 * real_h)/2.0;

    Coordinate{
        x: real_x as i32,
        y: real_y as i32,
        w: real_w as i32,
        h: real_h as i32
    }
    
}

pub fn trans_base64_to_image(base64_image: &String) -> DynamicImage {
    let base64_data = from_base64(base64_image.clone());
    // 将解码后的数据加载为图像
    let image = image::load_from_memory(&base64_data)
        .expect("Failed to load image from memory");
    image
}

/// 处理图片，返回图片预处理过程每一步中间图
/// 并根据长宽比例完成图片的90度翻转
pub fn process_image(model_size: &ModelSize, base64_image: &String) -> ProcessedImages {
    let mut img = trans_base64_to_image(base64_image);
    // 如果标注的长宽大小和图片的长宽大小关系不同，说明图片需要90度偏转
    let flag_need_90 = (model_size.h > model_size.w) != (img.height() > img.width());
    if flag_need_90{
        img = img.rotate90();
    };
    
    let rgb_img = img.to_rgb8();
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
    
    let integral_gray:ImageBuffer<Luma<i64>, Vec<i64>> = integral_image(&blurred_img);
    let integral_morphology:ImageBuffer<Luma<i64>, Vec<i64>> = integral_image(&eroded_img);

    ProcessedImages{
        org: Some(base64_image.clone()),
        rgb: rgb_img,
        gray: gray_img,
        morphology: eroded_img,
        integral_gray: integral_gray,
        integral_morphology: integral_morphology,
    }
}

/// 旋转ProcessedImages
pub fn rotate_processed_image(img: &mut ProcessedImages, angle_radians: f32){
    img.rgb = rotate_about_center(&img.rgb, angle_radians, Interpolation::Bilinear, Rgb([255,255,255]));
    img.gray = rotate_about_center(&img.gray, angle_radians, Interpolation::Bilinear, Luma([255]));
    img.morphology = rotate_about_center(&img.morphology, angle_radians, Interpolation::Bilinear, Luma([255]));
    img.integral_gray = integral_image(&img.gray);
    img.integral_morphology = integral_image(&img.morphology);
}

/// 计算页码点标注填涂率和真实填涂率的距离
pub fn calculate_page_number_difference(
    integral_img: &ImageBuffer<Luma<i64>, Vec<i64>>,
    coordinates: &Vec<Coordinate>,
    fill_rates: &Vec<f32>
) -> f32 {
    let mut real_fill_rates: Vec<f32> = Vec::new();
    for coordinate in coordinates.iter(){
        let sum_pixel = sum_image_pixels(
            integral_img,
            coordinate.x as u32,
            coordinate.y as u32,
            coordinate.x as u32 + coordinate.w as u32,
            coordinate.y as u32 + coordinate.h as u32
        )[0];
        let mean_pixel = sum_pixel / (coordinate.w * coordinate.h) as i64;
        let rate_pixel = 1.0 - mean_pixel as f32 / 255f32;
        real_fill_rates.push(rate_pixel);
    }

    mean_absolute_difference(&fill_rates, &real_fill_rates)
}

pub fn image_to_base64(img: &RgbImage) -> String {
    let mut image_data: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut image_data), ImageFormat::Jpeg).expect("Encode Image to Base64 Failed");
    vec_to_base64(image_data)
}


/**
 * 截取图像
 */
pub fn crop_image(input_image: &RgbImage, coordinate: &Coordinate) -> RgbImage {  
    let (width, height) = (coordinate.w as u32, coordinate.h as u32);  
    let mut cropped_image = ImageBuffer::from_fn(width, height, |_, _| Rgb([255u8; 3]));  
  
    for y in 0..height {  
        for x in 0..width {  
            let src_x = coordinate.x as u32 + x;  
            let src_y = coordinate.y as u32 + y;  
            if src_x < input_image.width() && src_y < input_image.height() {  
                let pixel: &Rgb<u8> = input_image.get_pixel(src_x, src_y);  
                cropped_image.put_pixel(x, y, *pixel);  
            } else {  
                //异常为白色
                cropped_image.put_pixel(x, y, image::Rgb([255, 255, 255]));  
            }  
        }  
    }  
  
    cropped_image
}