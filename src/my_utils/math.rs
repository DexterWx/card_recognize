use std::collections::HashSet;
use crate::{config::CONFIG, models::{card::MyPoint, engine_rec::LocationInfo, scan_json::Coordinate}};

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

pub fn normalize_vec(vec: &mut Vec<f32>) {
    // 找到向量中的最小值和最大值
    let min_val = *vec.iter().min_by(|x, y| x.partial_cmp(y).unwrap()).unwrap();
    let max_val = *vec.iter().max_by(|x, y| x.partial_cmp(y).unwrap()).unwrap();
    
    // 计算范围
    let range = max_val - min_val;
    if range == 0f32{return}
    
    // 归一化处理
    for val in vec {
        *val = (*val - min_val) / range;
    }
}

/// 欧氏距离
pub fn euclidean_distance(point1: (f32, f32), point2: (f32, f32)) -> f32 {
    let dx = point2.0 - point1.0;
    let dy = point2.1 - point1.1;

    (dx.powi(2) + dy.powi(2)).sqrt()
}

pub fn coordinates4_is_valid(coors: &[Coordinate; 4], location_info: &LocationInfo) -> bool{
    // 不能存在相同坐標
    let mut seen = HashSet::new();
    for &coor in coors.iter() {
        if !seen.insert(coor) {
            #[cfg(debug_assertions)]
            {
                println!("定位点存在相同坐标: {coors:?}");
            }
            return false; // 如果插入失败，说明已经存在相同的坐标，返回 false
        }
    }

    // 四个定位点的组成的矩形面积不能小于20000
    if (coors[0].x - coors[1].x).abs() * (coors[0].y - coors[2].y).abs() < 20000{return false;}
    if (coors[3].x - coors[2].x).abs() * (coors[3].y - coors[1].y).abs() < 20000{return false;}
    
    // 四点是否距离等差
    let diff_x = ((coors[2].x - coors[0].x) - (coors[3].x - coors[1].x)).abs();
    let diff_y = ((coors[2].y - coors[0].y) - (coors[3].y - coors[1].y)).abs();
    if diff_x > CONFIG.image_baizheng.model_point_diff || diff_y > CONFIG.image_baizheng.model_point_diff{
        #[cfg(debug_assertions)]
        {
            println!("定位点距离差异常: diff_x {diff_x:?} diff_y {diff_y:?}");
        }
        return false;
    }

    // 四个角是否都接近90度
    let valid_indexs = vec![
        (2,0,1),
        (0,1,3),
        (3,2,0),
        (1,3,2),
    ];
    for index in valid_indexs.iter(){
        let coor3 = [&coors[index.0],&coors[index.1],&coors[index.2]];
        let angle = calculate_coordinates_angle(&coor3);
        let angle_diff = (angle - 90f32).abs();
        if angle_diff >= CONFIG.image_baizheng.model_points_3_angle_threshold {
            #[cfg(debug_assertions)]
            {
                println!("定位点中存在角度不是90的: {:?}_{:?}",index, angle);
            }
            return false;
        }
    }
    
    // 四个框的wh和标注的wh的余弦相似度是否有离群点
    let mut cos_vec = Vec::new();
    for coor in coors.iter(){
        let cos = cosine_similarity(&vec![coor.w as f32,coor.h as f32], &vec![location_info.wh.0 as f32, location_info.wh.1 as f32]);
        cos_vec.push(cos);
    }
    let cos_mean = mean(&cos_vec).unwrap();
    let cos_std = standard_deviation(&cos_vec).unwrap();
    for cos in cos_vec.iter(){
        if *cos > CONFIG.image_baizheng.valid_coordinates4_cosine_similarity {continue}
        if *cos < cos_mean - 1.5*cos_std {
            #[cfg(debug_assertions)]
            {
                println!("定位点中存在离群点: {:?}",cos_vec);
            }
            return false
        }
    }

    // 四个框的w+h是否有离群点
    let mut sum_wh = Vec::new();
    for coor in coors.iter(){
        sum_wh.push((coor.w+coor.h) as f32);
    }
    let wh_mean = mean(&sum_wh).unwrap();
    let wh_std = standard_deviation(&sum_wh).unwrap();
    for wh in sum_wh.iter(){
        if *wh < wh_mean - 1.5*wh_std && (*wh - wh_mean).abs() > CONFIG.image_baizheng.valid_coordinates_wh_sum_mean_dis{
            #[cfg(debug_assertions)]
            {
                println!("定位点中存在离群点: {:?}",sum_wh);
            }
            return false
        }
    }
    
    true
}

/// 从四个定位点中选三个合理的顶点
pub fn find_3_valid_coordinates(coors: &[Coordinate; 4], location_info: &LocationInfo) -> Option<[Coordinate; 3]>{

    let mut _coors = Vec::new();
    // 过滤余弦相似度离群点
    let mut cos_vec = Vec::new();
    for coor in coors.iter(){
        let cos = cosine_similarity(&vec![coor.w as f32,coor.h as f32], &vec![location_info.wh.0 as f32, location_info.wh.1 as f32]);
        cos_vec.push(cos);
    }
    let cos_mean = mean(&cos_vec).unwrap();
    let cos_std = standard_deviation(&cos_vec).unwrap();
    for (i, cos) in cos_vec.iter().enumerate(){
        if *cos > CONFIG.image_baizheng.valid_coordinates4_cosine_similarity {
            _coors.push(coors[i]);
            continue;
        }
        if *cos < cos_mean - 1.5*cos_std {
            continue;
        }
        _coors.push(coors[i]);
    }
    
    if _coors.len() < 3 {
        #[cfg(debug_assertions)]
        {
            println!("三点定位中存在离群点: {:?}",cos_vec);
        }
        return None;
    }
    let coors = _coors;
    let mut _coors = Vec::new();

    // 过滤w+h离群点
    let mut sum_wh = Vec::new();
    for coor in coors.iter(){
        sum_wh.push((coor.w+coor.h) as f32);
    }
    let wh_mean = mean(&sum_wh).unwrap();
    let wh_std = standard_deviation(&sum_wh).unwrap();
    for (i,wh) in sum_wh.iter().enumerate(){
        if *wh < wh_mean - 1.5*wh_std && (*wh - wh_mean).abs() > CONFIG.image_baizheng.valid_coordinates_wh_sum_mean_dis {
            continue;
        }
        _coors.push(coors[i]);

    }

    if _coors.len() < 3 {
        #[cfg(debug_assertions)]
        {
            println!("三点定位中存在离群点: {:?}",sum_wh);
        }
        return None;
    }
    let coors = _coors;
    
    if coors.len() == 3 {
        let valid_indexs = vec![
            (0,1,2),
            (1,2,0),
            (1,0,2),
        ];
        let mut min_diff_indexs = None;
        let mut min_diff = 361f32;
        for index in valid_indexs.iter(){
            let coor3 = [&coors[index.0],&coors[index.1],&coors[index.2]];
            let angle = calculate_coordinates_angle(&coor3);
            let angle_diff = (angle - 90f32).abs();
            if angle_diff >= CONFIG.image_baizheng.model_points_3_angle_threshold {continue}
            if angle_diff < min_diff {
                min_diff = angle_diff;
                min_diff_indexs = Some([coors[index.0], coors[index.1], coors[index.2]]);
            }
        }
        return min_diff_indexs;
    }

    // 剩余四个点
    // 四个顶点分别为夹角，判断是否符合3点直角条件
    let valid_indexs = vec![
        (2,0,1),
        (0,1,3),
        (3,2,0),
        (1,3,2),
    ];
    let mut min_diff_indexs = None;
    let mut min_diff = 361f32;
    for index in valid_indexs.iter(){
        let coor3 = [&coors[index.0],&coors[index.1],&coors[index.2]];
        let angle = calculate_coordinates_angle(&coor3);
        let angle_diff = (angle - 90f32).abs();
        if angle_diff >= CONFIG.image_baizheng.model_points_3_angle_threshold {continue}
        if angle_diff < min_diff {
            min_diff = angle_diff;
            min_diff_indexs = Some([coors[index.0], coors[index.1], coors[index.2]]);
        }
    }
    min_diff_indexs
}

fn calculate_coordinates_angle(coors: &[&Coordinate; 3]) -> f32 {
    let a = coors[0]; // 第一个坐标点
    let b = coors[1]; // 第二个坐标点（夹角点）
    let c = coors[2]; // 第三个坐标点

    // 计算从 b 到 a 和 c 的向量
    let vec1 = ((a.x - b.x) as f32, (a.y - b.y) as f32);
    let vec2 = ((c.x - b.x) as f32, (c.y - b.y) as f32);

    // 计算点积
    let dot_product = vec1.0 * vec2.0 + vec1.1 * vec2.1;

    // 计算向量的大小
    let mag_vec1 = (vec1.0.powf(2.0) + vec1.1.powf(2.0)).sqrt();
    let mag_vec2 = (vec2.0.powf(2.0) + vec2.1.powf(2.0)).sqrt();

    // 计算弧度角
    let angle_rad = (dot_product / (mag_vec1 * mag_vec2)).acos();

    // 将弧度转换为度数
    let angle_deg = angle_rad.to_degrees();

    angle_deg
}


// 如果有三个定位点是正常的，根据这三个点生成第四个点,生成的点是中间点的对角点
pub fn predict_model_points_with_3_coordinate(coors: &[Coordinate; 3]) -> [Coordinate; 4]{
    let w = (coors[0].w + coors[1].w + coors[2].w)/3;
    let h = (coors[0].h + coors[1].h + coors[2].h)/3;
    let x = coors[0].x - coors[1].x + coors[2].x;
    let y = coors[0].y - coors[1].y + coors[2].y;
    let mut _coors = [coors[0],coors[1],coors[2],Coordinate::new(x,y,w,h)];
    get_sort_coordinates(_coors)
}

// [lt, rt, ld, rd]排序
fn get_sort_coordinates(mut coors: [Coordinate; 4]) -> [Coordinate; 4] {

    coors.sort_by_key(|c| c.x+c.y);
    let coor1 = coors[0];

    coors.sort_by_key(|c| c.x-c.y);
    let coor2 = coors[3];

    coors.sort_by_key(|c| c.x-c.y);
    let coor3 = coors[0];

    coors.sort_by_key(|c| c.x+c.y);
    let coor4 = coors[3];

    [coor1, coor2, coor3, coor4]
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

// 计算向量的均值
fn mean(data: &[f32]) -> Option<f32> {
    let sum: f32 = data.iter().sum();
    let count = data.len() as f32;
    if count > 0.0 {
        Some(sum / count)
    } else {
        None
    }
}

// 计算向量的标准差
pub fn standard_deviation(data: &[f32]) -> Option<f32> {
    if let Some(mean) = mean(data) {
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / (data.len() as f32);
        Some(variance.sqrt())
    } else {
        None
    }
}

/// 利用otsu算法求填涂阈值，把f32先转成0-100的u8
pub fn get_otsu(data: &Vec<u8>, weight: f64) -> (u8, f64, f64) {
    let mut hist = [0u32; 256];

    for &value in data {
        hist[value as usize] += 1;
    }

    let total_weight = data.len() as u32;

    let total_value_sum = hist.iter()
        .enumerate()
        .fold(0f64, |sum, (value, count)| sum + (value as u32 * count) as f64);

    let mut background_value_sum = 0f64;

    let mut background_weight = 0u32;
    let mut foreground_weight;

    let mut largest_variance = 0f64;
    let mut best_threshold = 0u8;
    let mut best_back_weight = 0f64;

    for (threshold, &hist_count) in hist.iter().enumerate() {
        background_weight += hist_count;
        if background_weight == 0 {
            continue;
        }

        foreground_weight = total_weight - background_weight;
        if foreground_weight == 0 {
            break;
        }

        background_value_sum += (threshold as u32 * hist_count) as f64;
        let foreground_value_sum = total_value_sum - background_value_sum;

        let background_mean = background_value_sum / (background_weight as f64);
        let foreground_mean = foreground_value_sum / (foreground_weight as f64);

        let mean_diff_squared = (background_mean - foreground_mean).powi(2);
        let intra_class_variance =
            (background_weight as f64/total_weight as f64) * (foreground_weight as f64/total_weight as f64) * mean_diff_squared;

        // Apply the weight to the variance to adjust the threshold
        let weighted_variance = intra_class_variance * (1.0 - weight * (1.0 - (threshold as f64 / 100.0)));

        if weighted_variance > largest_variance {
            largest_variance = weighted_variance;
            best_threshold = threshold as u8;
            best_back_weight = background_weight as f64/total_weight as f64;
        }
    }

    (best_threshold, largest_variance, best_back_weight)
}

