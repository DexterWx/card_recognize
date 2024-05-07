use crate::{config::CONFIG, models::card::MyPoint};

/// 余弦相似度
pub fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
    let dot_product = vec1.iter().zip(vec2.iter()).map(|(&a, &b)| a * b).sum::<f32>();
    let magnitude1 = (vec1.iter().map(|&x| x * x).sum::<f32>()).sqrt();
    let magnitude2 = (vec2.iter().map(|&x| x * x).sum::<f32>()).sqrt();

    dot_product / (magnitude1 * magnitude2)
}

/// 两个向量差的绝对值的均值
/// 衡量两个向量的数值差异
pub fn mean_absolute_difference(vec1: &[f32], vec2: &[f32]) -> f32 {
    let n = vec1.len() as f32;
    let sum_absolute_difference: f32 = vec1.iter().zip(vec2.iter()).map(|(&a, &b)| (a - b).abs()).sum();
    sum_absolute_difference / n
}

/// 欧氏距离
pub fn euclidean_distance(point1: (f32, f32), point2: (f32, f32)) -> f32 {
    let dx = point2.0 - point1.0;
    let dy = point2.1 - point1.1;

    (dx.powi(2) + dy.powi(2)).sqrt()
}

pub fn points4_is_valid(points: [(i32, i32); 4]) -> bool{
    let diff_x = ((points[2].0 - points[0].0) - (points[3].0 - points[1].0)).abs();
    let diff_y = ((points[2].1 - points[0].1) - (points[3].1 - points[1].1)).abs();
    if diff_x > CONFIG.image_baizheng.model_point_diff{return false;}
    if diff_y > CONFIG.image_baizheng.model_point_diff{return false;}
    true
}

// 计算线段夹角
pub fn cal_segment_angle(p1: MyPoint, p2: MyPoint, q1: MyPoint, q2: MyPoint) -> f32 {
    let v1 = MyPoint::new(p2.x - p1.x, p2.y - p1.y);
    let v2 = MyPoint::new(q2.x - q1.x, q2.y - q1.y);

    let dot_product = (v1.x * v2.x + v1.y * v2.y) as f32;
    let cross_product = (v1.x * v2.y - v1.y * v2.x) as f32;

    // 计算夹角的弧度
    let angle_rad = cross_product.atan2(dot_product);

    angle_rad
}