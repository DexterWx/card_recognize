use image::DynamicImage;

use crate::recognition::engine::Engine;


pub trait RecVX{
    /// 勾叉识别
    fn rec_vx<T, D>(&self, toinfo: T, img: &DynamicImage) -> D;
}

impl RecVX for Engine {
    fn rec_vx<T, D>(&self, toinfo: T, img: &DynamicImage) -> D {
        
        unimplemented!()
    }
}