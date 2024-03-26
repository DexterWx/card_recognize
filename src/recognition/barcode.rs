use image::DynamicImage;

use crate::recognition::engine::Engine;


pub trait RecBarcode{
    /// 条形码识别
    fn rec_barcode<T, D>(&self, toinfo: T, img: &DynamicImage) -> D;
}

impl RecBarcode for Engine {
    fn rec_barcode<T, D>(&self, toinfo: T, img: &DynamicImage) -> D {
        
        unimplemented!()
    }
}