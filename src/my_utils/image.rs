use std::ops::{Mul, Sub, Add};
use image::{DynamicImage, GrayImage, ImageBuffer, Luma, Rgb, RgbImage};
use imageproc::{filter::gaussian_blur_f32, point::Point};
use imageproc::integral_image::sum_image_pixels;

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

/// 根据给定的中心点center按角度angle_rad顺时针旋转
pub fn rotate_point(point: MyPoint, center: &MyPoint, angle_rad: f32) -> (i32, i32)
{
    let cos_theta:f32 = angle_rad.cos();
    let sin_theta:f32 = angle_rad.sin();

    let x_diff = point.x - center.x;
    let y_diff = point.y - center.y;
    let rotated_x = (center.x as f32) + (x_diff as f32) * cos_theta - (y_diff as f32) * sin_theta;
    let rotated_y = (center.y as f32) + (x_diff as f32) * sin_theta + (y_diff as f32) * cos_theta;

    (rotated_x as i32, rotated_y as i32)
}

pub fn generata_real_coordinate_with_model_points(model_points: &[ModelPoint;3], real_model_points: &[MyPoint;3], coordinate: &Coordinate) -> Coordinate{
    let x_rate = ((real_model_points[0].x - real_model_points[1].x) as f32) / ((model_points[0].coordinate.x - model_points[1].coordinate.x) as f32);
    let y_rate = ((real_model_points[0].y - real_model_points[2].y) as f32) / ((model_points[0].coordinate.y - model_points[2].coordinate.y) as f32);

    let real_w = x_rate * (coordinate.w as f32);
    let real_h = y_rate * (coordinate.h as f32);

    let real_w_model_point = x_rate * (model_points[0].coordinate.w as f32);
    let real_h_model_point = y_rate * (model_points[0].coordinate.h as f32);
    let real_x_model_point = real_model_points[0].x as f32 - real_w_model_point/2.0;
    let real_y_model_point = real_model_points[0].y as f32 - real_h_model_point/2.0;

    let real_x = x_rate * (coordinate.x - model_points[0].coordinate.x) as f32 + real_x_model_point;
    let real_y = y_rate * (coordinate.y - model_points[0].coordinate.y) as f32 + real_y_model_point;

    // let real_x = real_x_center - (model_points[0].coordinate.w as f32 * real_w)/2.0;
    // let real_y = real_y_center - (model_points[0].coordinate.h as f32 * real_h)/2.0;

    Coordinate{
        x: real_x as i32,
        y: real_y as i32,
        w: real_w as i32,
        h: real_h as i32
    }
    
}

pub fn calculate_page_number_match_rate(
    blurred_img: &GrayImage,
    coordinates: &Vec<Coordinate>,
    fill_rates: &Vec<f32>
) -> f32 {
    // sum_image_pixels();
    1.1
}