use image::DynamicImage;

use crate::recognition::engine::Engine;


pub trait RecBarcode{
    /// 输入的图片已经是经过小角度摆正的图片
    /// 该函数根据页码点进行大角度摆正
    fn rec_barcode<T, D>(&self, toinfo: T, img: &DynamicImage) -> D;
}

impl RecBarcode for Engine {
    fn rec_barcode<T, D>(&self, toinfo: T, img: &DynamicImage) -> D {
        
        unimplemented!()
    }
}