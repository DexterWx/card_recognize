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
    fn rec_black_fill<T, D>(&self, toinfo: T, img: &RgbImage) -> D;
    fn rec_number<T, D>(&self, toinfo: T, img: &RgbImage) -> D;
    fn rec_vx<T, D>(&self, toinfo: T, img: &RgbImage) -> D;
    fn rec_barcode<T, D>(&self, toinfo: T, img: &RgbImage) -> D;
}

impl Recognizes for Engine {
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