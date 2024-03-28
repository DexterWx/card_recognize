use std::collections::HashMap;
use rxing::{
    common::HybridBinarizer,
    multi::{GenericMultipleBarcodeReader, MultipleBarcodeReader},
    BinaryBitmap, DecodeHintType, DecodeHintValue,
    MultiUseMultiFormatReader,
    BufferedImageLuminanceSource,
};
use image::DynamicImage;
use crate::recognition::engine::Engine;
use crate::models::scan_json::Coordinate;
use crate::my_utils::image::crop_image;

/*
https://klximg.oss-cn-beijing.aliyuncs.com/scan-hb/024110/78d5e89231cc2d86e416353ae8d11a69.jpg
https://klximg.oss-cn-beijing.aliyuncs.com/scan-hb/024110/acea4aee0fcd516ddeea3c6984190790.jpg
https://klximg.oss-cn-beijing.aliyuncs.com/scan-hb/024110/8114d815a48dd733058ffaaaf5991fec.jpg
https://klximg.oss-cn-beijing.aliyuncs.com/scan-hb/024110/d7215de8146ca90abb46a4942a97ff24.jpg
https://klximg.oss-cn-beijing.aliyuncs.com/scan-hb/024110/ac0d822d80200630d655ec8cfa82e4d0.jpg
*/
pub fn decode_barcode(img: &DynamicImage, coor: Coordinate) -> Option<&str> {
    let multi_format_reader = MultiUseMultiFormatReader::default();
    let mut scanner = GenericMultipleBarcodeReader::new(multi_format_reader);
    let mut hints = HashMap::new();

    hints
        .entry(DecodeHintType::TRY_HARDER)
        .or_insert(DecodeHintValue::TryHarder(true));
    let results = scanner.decode_multiple_with_hints(
        &mut BinaryBitmap::new(HybridBinarizer::new(BufferedImageLuminanceSource::new(crop_image(img, coor)))),
        &hints,
    ).expect("decodes");
    for result in results {
        if !result.getText().is_empty() {
            return Some(result.getText());
        }
    }
    return Option::None;
}



pub trait RecBarcode{
    /// 条形码识别
    fn rec_barcode<T, D>(&self, toinfo: T, img: &DynamicImage) -> D;
}

impl RecBarcode for Engine {
    fn rec_barcode<T, D>(&self, toinfo: T, img: &DynamicImage) -> D {
        
        unimplemented!()
    }
}