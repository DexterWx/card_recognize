use crate::models::scan_json::{self, InputImage, InputSecond};
use crate::config::CONFIG;

use crate::models::engine_rec::ReferenceModelPoints;
use crate::models::rec_result::{OutputRec, OutputRecSecond, PageSize, Value};
use crate::my_utils::image::{generate_real_coordinate_with_model_points, image_to_base64, process_image};
use crate::models::engine_rec::ProcessedImagesAndModelPoints;
use crate::recognition::barcode::RecBarcode;
use crate::recognition::black_fill::RecBlackFill;
use crate::recognition::numbers::RecNumber;
use crate::recognition::vx::RecVX;
use super::baizheng::{fix_coordinate_use_assist_points, Baizheng};

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
        // 如果有辅助定位点，生成对应的矫正操作
        self.set_assist_points(&imgs_and_model_points, &mut output);
        // 识别
        _recognize(self, &imgs_and_model_points, &mut output);
        // 二值化填涂率
        Engine::binary_fill_rate(&mut output);
        
        

        // 渲染
        #[cfg(debug_assertions)]
        {
            self.rendering_model_points(&mut imgs_and_model_points, &mut output);
            self.rendering_assist_points(&mut imgs_and_model_points, &mut output);
            Engine::rendering_black_fill_show_rate(&mut output);
            self.rendering_page_number(&mut imgs_and_model_points, &mut output);
            Engine::rendering_black_fill(&mut output);
            Engine::rendering_number(&mut output);
            Engine::rendering_vx(&mut output);
            Engine::rendering_barcode(&mut output);
        }

        (output, imgs_and_model_points)
    }

    pub fn recognize_second(input: &InputSecond) -> OutputRecSecond {
        let mut output = OutputRecSecond::new(input);
        _recognize_second(input, &mut output);
        output
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
                let mut real_coordinate = generate_real_coordinate_with_model_points(
                    &reference_model_points, &option.coordinate, true, None
                );
                fix_coordinate_use_assist_points(&mut real_coordinate, &page_out.assist_points_move_op.get(&option.coordinate.y));
                let mut res:Option<Value> = None;
                match rec.rec_type {
                    rec_type if rec_type==CONFIG.recognize_type.black_fill => {
                        res = Engine::rec_black_fill(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.single_select => {
                        res = Engine::rec_black_fill(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.multi_select => {
                        res = Engine::rec_black_fill(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.vx => {
                        res = Engine::rec_vx(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.number => {
                        res = Engine::rec_number(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.qrcode => {
                        res = Engine::rec_barcode(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.barcode => {
                        res = Engine::rec_barcode(&img_and_model_points.img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.coordinate => {
                        option_out.coordinate = Some(real_coordinate);
                    }
                    _ =>{}
                }
                option_out.value = res.clone();
                option_out._value = res;
                #[cfg(debug_assertions)]
                {
                    option_out.coordinate = Some(real_coordinate);
                }
                
            }
        }
    }
}


fn _recognize_second(input: &InputSecond, output: &mut OutputRecSecond) {
    for (page,(img,page_out)) in input.pages.iter().zip(input.images.iter().zip(output.pages.iter_mut())){
        let img = process_image(None, img).unwrap();
        for (rec, rec_out) in page.recognizes.iter().zip(page_out.recognizes.iter_mut()){
            for (option, option_out) in rec.options.iter().zip(rec_out.rec_options.iter_mut()) {
                let real_coordinate = &option.coordinate;
                let mut res:Option<Value> = None;
                match rec.rec_type {
                    rec_type if rec_type==CONFIG.recognize_type.black_fill => {
                        res = Engine::rec_black_fill(&img, real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.single_select => {
                        res = Engine::rec_black_fill(&img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.multi_select => {
                        res = Engine::rec_black_fill(&img, &real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.vx => {
                        res = Engine::rec_vx(&img, real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.number => {
                        res = Engine::rec_number(&img, real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.qrcode => {
                        res = Engine::rec_barcode(&img, real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.barcode => {
                        res = Engine::rec_barcode(&img, real_coordinate);
                    }
                    rec_type if rec_type==CONFIG.recognize_type.coordinate => {
                        option_out.coordinate = Some(real_coordinate.clone());
                    }
                    _ =>{}
                }
                option_out.value = res;
                #[cfg(debug_assertions)]
                {
                    option_out.coordinate = Some(real_coordinate.clone());
                }
                
            }
        }
    }
}
