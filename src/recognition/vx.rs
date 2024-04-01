use crate::{models::{engine_rec::ProcessedImages, rec_result::Value, scan_json::Coordinate}, recognition::engine::Engine};

pub trait RecVX{
    /// 勾叉识别
    fn rec_vx(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
}

impl RecVX for Engine {
    fn rec_vx(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value> {
        None
    }
}