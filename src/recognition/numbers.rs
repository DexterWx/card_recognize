use crate::{models::{engine_rec::ProcessedImages, rec_result::Value, scan_json::Coordinate}, recognition::engine::Engine};


pub trait RecNumber{
    /// 数字识别
    fn rec_number(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
}

impl RecNumber for Engine {
    fn rec_number(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value> {
        None
    }
}