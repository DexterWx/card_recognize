use crate::{models::{engine_rec::ProcessedImages, rec_result::{OutputRec, Value}, scan_json::Coordinate}, recognition::engine::Engine};

pub trait RecVX{
    /// 勾叉识别
    fn rec_vx(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
    fn rendering_vx(&self, output: &mut OutputRec);
}

impl RecVX for Engine {
    fn rec_vx(&self, _img: &ProcessedImages, _coordinate: &Coordinate) -> Option<Value> {
        None
    }

    fn rendering_vx(&self, _output: &mut OutputRec) {
        
    }
}