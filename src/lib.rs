use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use models::scan_json::InputTest1;

pub mod recognition;
pub mod models;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Debug, Serialize, Deserialize)]
struct TStruct{
    name: String,
    id: u8,
}

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
    fn test_opencv(){

    }
}
