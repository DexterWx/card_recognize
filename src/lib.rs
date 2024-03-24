pub mod recognition;
pub mod models;
pub mod my_utils;

use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use image::{ImageBuffer, Rgb, RgbImage};

use models::scan_json::InputTest1;
use recognition::baizheng::process_img;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_json() -> Result<()>{
        let json_str = r#"
        {
            "index1": 1
        }
        "#;
        let input_json:InputTest1 = serde_json::from_str(json_str).context("dep failed")?;
        let input_str = serde_json::to_string(&input_json).context("p failed")?;
        println!("{input_str:?}");
        Ok(())
    }
        
    #[test]
    fn test_image(){
        use imageproc::drawing::{draw_filled_circle_mut};
        let (mut img,[lt,rt,ld,rd]) = process_img("test_data/test_2.jpg");
        for point in [lt,rt,ld,rd]{
            draw_filled_circle_mut(&mut img, (point.x as i32, point.y as i32), 10, Rgb([0, 0, 255]));
        }
        img.save("output_location.jpg").expect("Failed to save image");
    }
}
