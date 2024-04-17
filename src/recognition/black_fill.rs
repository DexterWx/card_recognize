use imageproc::integral_image::sum_image_pixels;

use crate::{models::{engine_rec::ProcessedImages, rec_result::Value, scan_json::Coordinate}, recognition::engine::Engine};

pub trait RecBlackFill{
    /// 填涂识别
    fn rec_black_fill(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
}

impl RecBlackFill for Engine {
    fn rec_black_fill(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>{
        // sum_image_pixels(integral_image, left, top, right-1, bottom-1)
        None
    }
}