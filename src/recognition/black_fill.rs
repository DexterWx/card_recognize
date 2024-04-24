

use crate::{models::{engine_rec::ProcessedImages, rec_result::{OutputRec, Value}, scan_json::Coordinate}, recognition::engine::Engine};

pub trait RecBlackFill{
    /// 填涂识别
    fn rec_black_fill(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
    fn rendering_black_fill(&self, output: &mut OutputRec);
}

impl RecBlackFill for Engine {
    fn rec_black_fill(&self, _img: &ProcessedImages, _coordinate: &Coordinate) -> Option<Value>{
        // sum_image_pixels(integral_image, left, top, right-1, bottom-1)
        None
    }
    
    fn rendering_black_fill(&self, _output: &mut OutputRec) {
        
    }
}