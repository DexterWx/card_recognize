use std::collections::HashMap;
use rxing::{
    BarcodeFormat,
    common::HybridBinarizer,
    multi::{GenericMultipleBarcodeReader, MultipleBarcodeReader},
    BinaryBitmap, DecodeHintType, DecodeHintValue,
    MultiUseMultiFormatReader,
    BufferedImageLuminanceSource,
};
use image::DynamicImage;
use crate::{models::{engine_rec::ProcessedImages, rec_result::{OutputRec, Value}}, recognition::engine::Engine};
use crate::models::scan_json::Coordinate;
use crate::my_utils::image::crop_image;
pub fn decode_barcode(img: &DynamicImage, coor: &Coordinate) -> std::option::Option<String> {
    let multi_format_reader = MultiUseMultiFormatReader::default();
    let mut scanner = GenericMultipleBarcodeReader::new(multi_format_reader);
    let mut hints = HashMap::new();

    hints
        .entry(DecodeHintType::TRY_HARDER)
        .or_insert(DecodeHintValue::TryHarder(true));
    let decode_result = scanner.decode_multiple_with_hints(
        &mut BinaryBitmap::new(HybridBinarizer::new(BufferedImageLuminanceSource::new(crop_image(img, coor)))),
        &hints,
    );
    match decode_result {
        Ok(decode_list) => {
            let mut decode_f:Vec<rxing::RXingResult> = decode_list.into_iter().filter(|x| !x.getText().is_empty()).collect();
            //128格式优先
            decode_f.sort_by_key(|result| {  
                if result.getBarcodeFormat() == &BarcodeFormat::CODE_128 {  
                    0  
                } else {  
                    1  
                }  
            });  
            if decode_f.len() > 0 {
                Some(decode_f[0].getText().to_owned())
            }else {
                return Option::None;
            }
        },
        Err(_err) => return Option::None,
    }
}



pub trait RecBarcode{
    /// 条形码识别
    fn rec_barcode(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
    fn rendering_barcode(&self, output: &mut OutputRec);
}

impl RecBarcode for Engine {
    fn rec_barcode(&self, img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value> {
        match decode_barcode(&DynamicImage::ImageRgb8(img.rgb.clone()), coordinate) {
            Some(p) => Some(Value::String(p)),
            None => None,
        }
    }

    fn rendering_barcode(&self, output: &mut OutputRec) {
        
    }
}