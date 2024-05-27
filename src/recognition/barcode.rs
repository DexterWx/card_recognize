use std::collections::{HashMap, HashSet};
use rxing::{
    BarcodeFormat,
    common::HybridBinarizer,
    multi::{GenericMultipleBarcodeReader, MultipleBarcodeReader},
    BinaryBitmap, DecodeHintType, DecodeHintValue,
    MultiUseMultiFormatReader,
    BufferedImageLuminanceSource,
};
use image::DynamicImage;
use crate::{models::{engine_rec::ProcessedImages, rec_result::{OutputRec, Value}}, my_utils::image::crop_image, recognition::engine::Engine};
use crate::models::scan_json::Coordinate;
pub fn decode_barcode(img: DynamicImage) -> std::option::Option<String> {
    let multi_format_reader = MultiUseMultiFormatReader::default();
    let mut scanner = GenericMultipleBarcodeReader::new(multi_format_reader);
    let mut hints = HashMap::new();
    //hard模式尽可能识别，utf-8，单一条形码，只识别128那种条形码
    hints.insert(DecodeHintType::TRY_HARDER, DecodeHintValue::TryHarder(true));
    hints.insert(DecodeHintType::CHARACTER_SET, DecodeHintValue::CharacterSet(String::from("utf-8")));
    hints.insert(DecodeHintType::PURE_BARCODE, DecodeHintValue::PureBarcode(true));
    hints.insert(DecodeHintType::POSSIBLE_FORMATS, DecodeHintValue::PossibleFormats(HashSet::from([BarcodeFormat::CODE_128])));
    let decode_result = scanner.decode_multiple_with_hints(
        &mut BinaryBitmap::new(HybridBinarizer::new(BufferedImageLuminanceSource::new(img))),
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
    fn rec_barcode(img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value>;
    fn rendering_barcode(output: &mut OutputRec);
}

impl RecBarcode for Engine {
    fn rec_barcode(img: &ProcessedImages, coordinate: &Coordinate) -> Option<Value> {
        let crop: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = crop_image(&img.rgb, coordinate);
        match decode_barcode(DynamicImage::ImageRgb8(crop)) {
            Some(p) => Some(Value::String(p)),
            None => None,
        }
    }

    fn rendering_barcode(_output: &mut OutputRec) {
        
    }
}

  