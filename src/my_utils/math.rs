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

/// 欧氏距离
pub fn euclidean_distance(point1: (f32, f32), point2: (f32, f32)) -> f32 {
    let dx = point2.0 - point1.0;
    let dy = point2.1 - point1.1;

    (dx.powi(2) + dy.powi(2)).sqrt()
}

pub fn coordinates4_is_valid(coors: &[Coordinate; 4], location_info: &LocationInfo) -> bool{
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
        if *cos > 0.997 {continue}
        if *cos < cos_mean - 1.5*cos_std {
            #[cfg(debug_assertions)]
            {
                println!("定位点中存在离群点: {:?}",cos_vec);
            }
            return false
        }
    }
    //let cos = cosine_similarity(&vec![w as f32,h as f32], &vec![location_info.wh.0 as f32, location_info.wh.1 as f32]);
    
    true
}

/// 从四个定位点中选三个合理的顶点
pub fn find_3_valid_coordinates(coors: &[Coordinate; 4]) -> Option<[Coordinate; 3]>{
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
fn standard_deviation(data: &[f32]) -> Option<f32> {
    if let Some(mean) = mean(data) {
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / (data.len() as f32);
        Some(variance.sqrt())
    } else {
        None
    }
}

fn leverage_ratios(data: &Vec<f32>) -> Vec<f32> {
    let n = data.len() as f32;
    let mean: f32 = data.iter().sum::<f32>() / n;

    let sum_of_squares: f32 = data.iter().map(|&x| (x - mean).powi(2)).sum();

    data.iter().map(|&x| ((x - mean).powi(2)) / sum_of_squares).collect()
}