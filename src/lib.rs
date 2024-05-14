#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

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
        let (output, _imgs_and_model_points) = engine.recognize(&input_images);


        let out_json_path = format!("dev/test_data/{test_id}.json");
        let mut file = File::create(out_json_path)?;
        serde_json::to_writer(&mut file, &output)?;

        for (index,page) in output.pages.iter().enumerate(){
            if matches!(page.image_rendering, None){continue;}
            let img = page.image_rendering.as_ref().unwrap();
            let img = trans_base64_to_image(img);
            let path = format!("dev/test_data/output_rendering_{index}.jpg");
            let _ = img.to_rgb8().save(path);
        }

        // 图片code
        for (_index,image) in output.images.iter().enumerate(){
            println!("{:?}",image.code); 
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

    #[test]
    fn test_barcode() -> Result<()> {
        let image = image::open("dev/test_data/barcode_image/220420.png")?;
        let result = recognition::barcode::decode_barcode(image);
        assert!(result.unwrap() == "220420");
        return Result::Ok(());

    }
}


pub mod build{
    use crate::{models, recognition};

    use models::scan_json::{InputImage, InputScan};
    use recognition::engine::Engine;

    // 全局变量的引擎结构体
    static mut ENGINE: Option<Engine> = None;

    #[napi]
    pub fn initialize(input_json: String){
        let input_scan: InputScan = serde_json::from_str(&input_json).expect("Parse Input Failed");
        let input_scan = input_scan.renew();
        // 进行一些初始化操作
        unsafe {
            ENGINE = Some(Engine::new(input_scan));
        }
    }

    #[napi]
    pub fn inference(input_json:String) -> String {
        unsafe {
            // 检查引擎是否已初始化
            let engine = ENGINE.as_ref().expect("Engine not initialized");

            let input_image: InputImage = serde_json::from_str(&input_json).expect("Parse Input Failed");
            let result = engine.recognize(&input_image);
            let output_json = result.0;

            // 使用 serde_json 将结果序列化为 JSON 字符串
            serde_json::to_string(&output_json).expect("Failed to serialize JSON")
        }
    }
}
