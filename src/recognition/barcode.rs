use std::collections::HashMap;
use rxing::{
    common::HybridBinarizer,
    multi::{GenericMultipleBarcodeReader, MultipleBarcodeReader},
    BinaryBitmap, DecodeHintType, DecodeHintValue,
    MultiUseMultiFormatReader,
    BufferedImageLuminanceSource,
};
use image::DynamicImage;
use crate::models::card::MyPoint;

pub fn decode_barcode(img: &DynamicImage, points: [MyPoint;4]) -> Option<&str> {
    let multi_format_reader = MultiUseMultiFormatReader::default();
    let mut scanner = GenericMultipleBarcodeReader::new(multi_format_reader);
    let mut hints = HashMap::new();

    hints
        .entry(DecodeHintType::TRY_HARDER)
        .or_insert(DecodeHintValue::TryHarder(true));
    let results = scanner.decode_multiple_with_hints(
        &mut BinaryBitmap::new(HybridBinarizer::new(BufferedImageLuminanceSource::new(img))),
        &hints,
    ).expect("decodes");
    for result in results {
        if !result.getText().is_empty() {
            return Some(result.getText());
        }
    }
    return Option::None;
}
