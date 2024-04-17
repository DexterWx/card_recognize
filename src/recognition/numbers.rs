use crate::{models::{engine_rec::ProcessedImages, rec_result::{OutputRec, Value}, scan_json::Coordinate}, recognition::engine::Engine};


pub trait RecNumber{
    /// 数字识别
    fn rec_number(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
    fn rendering_number(&self, output: &mut OutputRec);
}

impl RecNumber for Engine {
    fn rec_number(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value> {
        None
    }

    fn rendering_number(&self, output: &mut OutputRec) {
        
    }
}