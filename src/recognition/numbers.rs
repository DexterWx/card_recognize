use image::DynamicImage;

use crate::recognition::engine::Engine;


pub trait RecNumber{
    /// 数字识别
    fn rec_number<T, D>(&self, toinfo: T, img: &DynamicImage) -> D;
}

impl RecNumber for Engine {
    fn rec_number<T, D>(&self, toinfo: T, img: &DynamicImage) -> D {
        
        unimplemented!()
    }
}