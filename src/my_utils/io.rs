use std::path::PathBuf;


/// 路径直接按linux下的写法，自动判断系统类型给出兼容格式
pub fn compatible_path_format(path: &str) -> String{
    let mut img_path = PathBuf::new();
    let parts:Vec<&str> = path.split('/').collect();
    for i in parts{
        img_path.push(i);
    }
    img_path.to_string_lossy().into_owned()
}