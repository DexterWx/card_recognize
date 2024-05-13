use crate::{config::CONFIG, models::{card::MyPoint, scan_json::Coordinate}};

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

pub fn coordinates4_is_valid(coors: &[Coordinate; 4]) -> bool{
    let diff_x = ((coors[2].x - coors[0].x) - (coors[3].x - coors[1].x)).abs();
    let diff_y = ((coors[2].y - coors[0].y) - (coors[3].y - coors[1].y)).abs();
    if diff_x > CONFIG.image_baizheng.model_point_diff{return false;}
    if diff_y > CONFIG.image_baizheng.model_point_diff{return false;}
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