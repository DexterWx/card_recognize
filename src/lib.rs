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
    use image_base64_wasm::to_base64;

    use self::my_utils::image::trans_base64_to_image;

    use super::*;
    use models::scan_json::{InputScan,InputImage};
    use recognition::engine::Engine;

    #[test]
    fn test_demo() -> Result<()> {

        // 直接修改id就可以测试
        let test_id = "194751";
        let json_path = format!("dev/test_data/cards/{test_id}/scan.json");
        let image_dir = format!("dev/test_data/cards/{test_id}/images");

        // 构建第一次输入的scanjson和第二次输入图片
        let input_scan = read_json(&json_path);
        let input_images = read_image(&image_dir).expect("Read Image Failed");

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
            if matches!(page.image_rendering, None){continue;}
            let mut rendering = trans_base64_to_image(&page.image_rendering.expect("image_rendering is None"));
            let out_img_path = format!("dev/test_data/output_view_{index}.jpg");
            rendering.to_rgb8().save(out_img_path);
            let out_mor_path = format!("dev/test_data/output_mor_{index}.jpg");
            img_and_model_points.as_ref().unwrap().img.morphology.save(out_mor_path);
            let out_gray_path = format!("dev/test_data/output_gray_{index}.jpg");
            img_and_model_points.as_ref().unwrap().img.gray.save(out_gray_path);
        }


        Ok(())

    }


    fn read_json(json_path: &str) -> InputScan {
        
        let scan_path = Path::new(json_path).to_str().expect("Parse Json Path Failed").to_string();
        let mut file = File::open(scan_path).expect("Failed to open file");

        // 读取文件内容
        let mut json_str = String::new();
        file.read_to_string(&mut json_str)
            .expect("Failed to read file");

        // 将 JSON 解析为 InputScan 结构体
        let parsed_struct: InputScan = serde_json::from_str(&json_str).expect("Parse InputScan Failed");
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

            // 将图像文件转换为 Base64 编码的字符串
            let base64_image = to_base64(&file_path_str.to_string());

            imgs.push(base64_image);
        }

        let input_image = InputImage{
            task_id: "test".to_string(),
            images: imgs,
            calling_type:Some(0)
        };

        Ok(input_image)
    }
}




use models::scan_json::{InputImage, InputScan};
use recognition::engine::Engine;
use wasm_bindgen::prelude::*;

// 全局变量的引擎结构体
static mut ENGINE: Option<Engine> = None;

#[wasm_bindgen]
pub fn initialize(input_json: &str){
    println!("{:?}",input_json);
    let input_scan: InputScan = serde_json::from_str(input_json).expect("Parse Input Failed");
    let input_scan = input_scan.renew();
    // 进行一些初始化操作
    unsafe {
        ENGINE = Some(Engine::new(input_scan));
    }
}

#[wasm_bindgen]
pub fn inference(input_json:&str) -> String {
    unsafe {
        // 检查引擎是否已初始化
        let engine = ENGINE.as_ref().expect("Engine not initialized");

        let input_image: InputImage = serde_json::from_str(input_json).expect("Parse Input Failed");
        let result = engine.recognize(&input_image);
        let output_json = result.0;

        // 使用 serde_json 将结果序列化为 JSON 字符串
        serde_json::to_string(&output_json).expect("Failed to serialize JSON")
    }
}