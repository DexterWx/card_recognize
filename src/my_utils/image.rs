use std::ops::{Add, Div, Mul, MulAssign, Sub};
use image::{DynamicImage, GrayImage, ImageBuffer, Luma, Rgb, RgbImage};
use imageproc::distance_transform::Norm;
use imageproc::geometric_transformations::{rotate_about_center, Interpolation};
use imageproc::local_binary_patterns::count_transitions;
use imageproc::morphology::{dilate, erode};
use imageproc::{filter::gaussian_blur_f32, point::Point};
use imageproc::integral_image::{integral_image, sum_image_pixels};

use crate::models::engine_rec::{ProcessedImages, ReferenceModelPoints};
use crate::{config::CONFIG, models::{card::MyPoint, scan_json::{Coordinate, ModelPoint}}};

trait HasCoordinates<T> {
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
    let mut center_x: i32 = center_x.into();
    let mut center_y: i32 = center_y.into();
    let center_x = center_x / num_points;
    let center_y = center_y / num_points;

    Some((center_x, center_y))
}


/// 计算一组点的左上
pub fn calculate_points_lt<T, K>(points: &[T]) -> Option<(i32, i32)>
where
    T: HasCoordinates<K>,
    K: Default + Copy + Into<i32> + From<i32>,
{
    if points.is_empty() {
        return None;
    }

    // 初始化左上的坐标
    let mut lt_x = i32::default();
    let mut lt_y = i32::default();

    let mut minxy = 111111 as i32;

    // 找到最小点
    for point in points {
        let (x, y) = point.get_coordinates();
        let x:i32 = (*x).into();
        let y:i32 = (*y).into();
        if x + y < minxy {
            lt_x = x;
            lt_y = y;
            minxy = x+y;
        }
    }

    Some((lt_x, lt_y))
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

pub fn process_image(img_path: String) -> ProcessedImages {
    let img = image::open(img_path).expect("Failed to open image file");
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
        rgb: rgb_img,
        gray: gray_img,
        morphology: eroded_img,
        integral_gray: integral_gray,
        integral_morphology: integral_morphology,
    }
}

pub fn rotate_processed_image(imgs: &mut ProcessedImages, angle_radians: f32){
    imgs.rgb = rotate_about_center(&imgs.rgb, angle_radians, Interpolation::Bilinear, Rgb([255,255,255]));
    imgs.gray = rotate_about_center(&imgs.gray, angle_radians, Interpolation::Bilinear, Luma([255]));
    imgs.morphology = rotate_about_center(&imgs.morphology, angle_radians, Interpolation::Bilinear, Luma([255]));
    imgs.integral_gray = integral_image(&imgs.gray);
    imgs.integral_morphology = integral_image(&imgs.morphology);
}


pub fn calculate_page_number_difference(
    integral_img: &ImageBuffer<Luma<i64>, Vec<i64>>,
    coordinates: &Vec<Coordinate>,
    fill_rates: &Vec<f32>
) -> f32 {
    let mut real_fill_rates: Vec<f32> = Vec::new();
    for (coordinate,fill_rate) in coordinates.iter().zip(fill_rates.iter()){
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

pub fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
    let dot_product = vec1.iter().zip(vec2.iter()).map(|(&a, &b)| a * b).sum::<f32>();
    let magnitude1 = (vec1.iter().map(|&x| x * x).sum::<f32>()).sqrt();
    let magnitude2 = (vec2.iter().map(|&x| x * x).sum::<f32>()).sqrt();

    dot_product / (magnitude1 * magnitude2)
}

pub fn mean_absolute_difference(vec1: &[f32], vec2: &[f32]) -> f32 {
    let n = vec1.len() as f32;
    let sum_absolute_difference: f32 = vec1.iter().zip(vec2.iter()).map(|(&a, &b)| (a - b).abs()).sum();
    sum_absolute_difference / n
}
