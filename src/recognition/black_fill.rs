use image::DynamicImage;

use crate::recognition::engine::Engine;


pub trait RecBlackFill{
    /// 填涂识别
    fn rec_black_fill<T, D>(&self, toinfo: T, img: &DynamicImage) -> D;
}

impl RecBlackFill for Engine {
    fn rec_black_fill<T, D>(&self, toinfo: T, img: &DynamicImage) -> D {
        
        unimplemented!()
    }
}