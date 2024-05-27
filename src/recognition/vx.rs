use crate::{models::{engine_rec::ProcessedImages, rec_result::{OutputRec, Value}, scan_json::Coordinate}, recognition::engine::Engine};

pub trait RecVX{
    /// 勾叉识别
    fn rec_vx(img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
    fn rendering_vx(output: &mut OutputRec);
}

impl RecVX for Engine {
    fn rec_vx( _img: &ProcessedImages, _coordinate: &Coordinate) -> Option<Value> {
        None
    }

    fn rendering_vx(_output: &mut OutputRec) {
        
    }
}