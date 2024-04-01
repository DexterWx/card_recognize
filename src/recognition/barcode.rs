use std::collections::HashMap;
use rxing::{
    common::HybridBinarizer,
    multi::{GenericMultipleBarcodeReader, MultipleBarcodeReader},
    BinaryBitmap, DecodeHintType, DecodeHintValue,
    MultiUseMultiFormatReader,
    BufferedImageLuminanceSource,
};
use image::DynamicImage;
use crate::{models::{engine_rec::{ProcessedImages, ProcessedImagesAndModelPoints}, rec_result::Value}, recognition::engine::Engine};
use crate::models::scan_json::Coordinate;
use crate::my_utils::image::crop_image;
pub fn decode_barcode(img: &DynamicImage, coor: Coordinate) -> Option<String> {
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
        //todo 需要处理识别多个结果
        if !result.getText().is_empty() {
            return Some(result.getText().to_string());
        }
    }
    return Option::None;
}



pub trait RecBarcode{
    /// 条形码识别
    fn rec_barcode(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
}

impl RecBarcode for Engine {
    fn rec_barcode(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value> {
        None
    }
}