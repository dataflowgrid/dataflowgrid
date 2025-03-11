pub mod parser;
pub mod deserializer;

pub struct JsonStreamReader {
}

impl JsonStreamReader {
    pub fn new() -> JsonStreamReader {
        JsonStreamReader {}
    }

    pub async fn read(&self, mut c: impl AsyncFnMut()) -> String {
        c().await;
        "Hello, World!".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let result = JsonStreamReader::new().read(|| async { println!("Hello")}).await;
        assert_eq!(result, "Hello, World!");
    }
}
