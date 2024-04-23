use std::fs::read_link;

use image::{DynamicImage, Rgb};
use image_base64_wasm::to_base64;
use imageproc::drawing::draw_filled_circle_mut;

use crate::models::scan_json::{self, Coordinate, InputImage};
use crate::config::CONFIG;

use crate::models::engine_rec::ReferenceModelPoints;
use crate::models::rec_result::{OutputRec, PageSize, Value};
use crate::my_utils::image::{generate_real_coordinate_with_model_points, image_to_base64, trans_base64_to_image};
use crate::models::engine_rec::ProcessedImagesAndModelPoints;
use crate::recognition::barcode::RecBarcode;
use crate::recognition::black_fill::RecBlackFill;
use crate::recognition::numbers::RecNumber;
use crate::recognition::vx::RecVX;
use super::baizheng::Baizheng;

#[derive(Debug)]
pub struct Engine {
    scan_data: scan_json::InputScan
    // todo
    // vx_model: torch::onnx,
    // number_model: torch::onnx,
}

impl Engine {
    pub fn new(scan_data: scan_json::InputScan) -> Self {
        Engine {
            scan_data
        }
    }
    /// 跨模块实现方法的时候访问不到成员变量，需要调用此函数
    pub fn get_scan_data(&self) -> &scan_json::InputScan {
        &self.scan_data
    }
    /// 识别，输出第二个变量用于可视化
    pub fn recognize(&self, input_images: &InputImage) -> (OutputRec,  Vec<Option<ProcessedImagesAndModelPoints>>){
        
        // 构建输出结构
        let scan_data = self.get_scan_data();
        let mut output = OutputRec::new(scan_data);
        
        // 摆正+匹配+找到定位点
        let mut imgs_and_model_points = self.baizheng_and_match_page(&input_images, &mut output);

        // 识别
        _recognize(self, &imgs_and_model_points, &mut output);

        // 渲染
        #[cfg(debug_assertions)]
        {
            self.rendering_model_points(&mut imgs_and_model_points, &mut output);
            self.rendering_black_fill(&mut output);
            self.rendering_number(&mut output);
            self.rendering_vx(&mut output);
            self.rendering_barcode(&mut output);
        }

        (output, imgs_and_model_points)
    }
}


/// 遍历所有option，根据rec_type调用不同的识别trait
fn _recognize(engine: &Engine, imgs_and_model_points: &Vec<Option<ProcessedImagesAndModelPoints>>, output: &mut OutputRec) {
    let scan_data = engine.get_scan_data();
    for (page,(img_and_model_points,page_out)) in scan_data.pages.iter().zip(imgs_and_model_points.iter().zip(output.pages.iter_mut())){
        // 没有图片跳过
        // ps.一种避免解析option嵌套的写法
        if matches!(img_and_model_points,None) {continue;}
        let img_and_model_points = img_and_model_points.as_ref().expect("img_and_model_points is None");
        // 填充输出图片信息
        page_out.has_page = true;
        page_out.image_source = img_and_model_points.img.org.clone();
        page_out.image_rotated = Some(image_to_base64(&img_and_model_points.img.rgb));
        page_out.page_size = Some(
            PageSize{
                w: img_and_model_points.img.rgb.width() as i32,
                h: img_and_model_points.img.rgb.height() as i32,
            }
        );

        // 构建坐标转换需要用到的参照定位点
        let reference_model_points = ReferenceModelPoints{
            model_points: &page.model_points_4.expect("model_points_4 is None"),
            real_model_points: &img_and_model_points.real_model_points
        };
        // 遍历每个option，根据识别类型调用不同的方法
        for (rec, rec_out) in page.recognizes.iter().zip(page_out.recognizes.iter_mut()){
            for (option, option_out) in rec.options.iter().zip(rec_out.rec_options.iter_mut()) {
                let real_coordinate = generate_real_coordinate_with_model_points(
                    &reference_model_points, &option.coordinate
                );
                let mut res:Option<Value> = None;
                match rec.rec_type {
                    rec_type if rec_type==CONFIG.recognize_type.black_fill => {
                        res = engine.rec_black_fill(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.vx => {
                        res = engine.rec_vx(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.number => {
                        res = engine.rec_number(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.qrcode => {
                        res = engine.rec_barcode(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.barcode => {
                        res = engine.rec_barcode(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.coordinate => {
                        option_out.coordinate = Some(real_coordinate);
                    }
                    _ =>{}
                }
                option_out.value = res;
                #[cfg(debug_assertions)]
                {
                    option_out.coordinate = Some(real_coordinate);
                }
                
            }
        }
    }
}
