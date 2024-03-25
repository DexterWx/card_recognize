use image::{Rgb, RgbImage};

use crate::models::scan_json;

pub struct Engine {
    scan_data: Vec<scan_json::Page>,
    card_type: u8,
    // todo
    // vx_model: torch::onnx,
    // number_model: torch::onnx,
}

impl Engine {
    pub fn new(scan_data: Vec<scan_json::Page>, card_type: u8) -> Self {
        Engine {
            scan_data,
            card_type,
        }
    }
}

pub trait Recognizes{
    /// 输入的图片已经是经过小角度摆正的图片
    /// 该函数根据页码点进行大角度摆正
    fn rotate_with_page_number<T, D>(&self, toinfo: T, img: &RgbImage) -> D;
    fn rec_black_fill<T, D>(&self, toinfo: T, img: &RgbImage) -> D;
    fn rec_number<T, D>(&self, toinfo: T, img: &RgbImage) -> D;
    fn rec_vx<T, D>(&self, toinfo: T, img: &RgbImage) -> D;
    fn rec_barcode<T, D>(&self, toinfo: T, img: &RgbImage) -> D;
}

impl Recognizes for Engine {
    fn rotate_with_page_number<T, D>(&self, toinfo: T, img: &RgbImage) -> D {
        
        unimplemented!()
    }
    fn rec_barcode<T, D>(&self, toinfo: T, img: &RgbImage) -> D {
        unimplemented!()
    }
    fn rec_black_fill<T, D>(&self, toinfo: T, img: &RgbImage) -> D {
        unimplemented!()
    }
    fn rec_number<T, D>(&self, toinfo: T, img: &RgbImage) -> D {
        unimplemented!()
    }
    fn rec_vx<T, D>(&self, toinfo: T, img: &RgbImage) -> D {
        unimplemented!()
    }
}