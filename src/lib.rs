pub mod recognition;
pub mod models;
pub mod my_utils;
pub mod config;

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::fs;
    use std::path::Path;

    use anyhow::{Result, Ok};

    use super::*;
    use models::scan_json::{InputScan,InputImage};
    use models::rec_result::OutputRec;
    use recognition::baizheng::Baizheng;
    use recognition::engine::Engine;
    use config::CONFIG;

    use image::Rgb;
    use imageproc::drawing::draw_filled_circle_mut;

    fn read_json(json_path: &str) -> InputScan {
        
        let scan_path = Path::new(json_path).to_str().unwrap().to_string();
        let mut file = File::open(scan_path).expect("Failed to open file");

        // 读取文件内容
        let mut json_str = String::new();
        file.read_to_string(&mut json_str)
            .expect("Failed to read file");

        // 将 JSON 解析为 MyStruct 结构体
        let parsed_struct: InputScan = serde_json::from_str(&json_str).unwrap();
        let input1 = parsed_struct.renew();
        input1
    }

    fn read_image(image_dir:&str) -> Result<InputImage> {
        let mut imgs = Vec::new();
        // 读取目录中的所有条目
        let entries = fs::read_dir(image_dir)?;
    
        // 遍历所有条目并输出文件名
        for entry in entries {
            let entry = entry?;
            let file_path = entry.path();
            let file_path_str = file_path.to_string_lossy();
            imgs.push(file_path_str.to_string());
        }

        let input_image = InputImage{
            task_id: "test".to_string(),
            images: imgs,
            calling_type:Some(0)
        };

        Ok(input_image)
    }

    #[test]
    fn test_demo() -> Result<()> {

        // 直接修改id就可以测试
        let test_id = "194751";
        let json_path = format!("dev/test_data/cards/{test_id}/scan.json");
        let image_dir = format!("dev/test_data/cards/{test_id}/images");

        // 构建第一次输入的scanjson和第二次输入图片
        let input_scan = read_json(&json_path);
        let input_images = read_image(&image_dir).unwrap();

        // 引擎初始化
        let engine = Engine::new(input_scan);
        // 识别
        let (output, imgs_and_model_points) = engine.recognize(&input_images);


        let out_json_path = format!("dev/test_data/{test_id}.json");
        let mut file = File::create(out_json_path)?;
        serde_json::to_writer(&mut file, &output)?;

        // 可视化
        for (index,(img_and_model_points, page)) in imgs_and_model_points.iter().zip(output.pages).enumerate(){
            if matches!(img_and_model_points, None){continue;}
            let img_and_model_points = img_and_model_points.as_ref().unwrap();
            let mut img = img_and_model_points.img.rgb.clone();
            for rec in page.recognizes.iter(){
                if rec.rec_type==CONFIG.recognize_type.coordinate{continue;}
                for option in rec.rec_options.iter(){
                    let coor = option.coordinate;
                    if matches!(coor,None){continue;}
                    let coor = coor.unwrap();
                    draw_filled_circle_mut(&mut img, (coor.x,coor.y), 5, Rgb([0,0,255]));
                    draw_filled_circle_mut(&mut img, (coor.x+coor.w,coor.y+coor.h), 5, Rgb([0,0,255]));
                }
            }
            for coor in img_and_model_points.real_model_points.iter(){
                draw_filled_circle_mut(&mut img, (coor.x,coor.y), 5, Rgb([0,0,255]));
                draw_filled_circle_mut(&mut img, (coor.x+coor.w,coor.y+coor.h), 5, Rgb([0,0,255]));
            }
            let out_img_path = format!("dev/test_data/output_view_{index}.jpg");
            img.save(out_img_path);

        }

        Ok(())

    }

}




// 静态变量声明方式
// use once_cell::sync::Lazy;

// // 定义模型类型
// struct Model {
//     // 模型的属性
//     param1: i32,
//     param2: f64,
// }

// // 静态变量，用于存储初始化的模型
// static MODEL: Lazy<Option<Model>> = Lazy::new(|| {
//     None
// });

// // 初始化模型的函数
// #[no_mangle]
// pub extern "C" fn initialize_model(param1: i32, param2: f64) {
//     // 更新静态变量中的模型，使用传入的参数
//     *MODEL.force() = Some(Model {
//         param1,
//         param2,
//     });
// }

// // 推理函数
// #[no_mangle]
// pub extern "C" fn inference() -> i32 {
//     // 获取静态变量中的模型，并执行推理操作
//     if let Some(model) = MODEL.force() {
//         // 使用模型执行推理操作
//         42 // 示例返回值
//     } else {
//         // 模型尚未初始化，返回错误值或者抛出异常
//         panic!("Model has not been initialized");
//     }
// }