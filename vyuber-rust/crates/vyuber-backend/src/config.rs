/// 環境変数から設定を読み込む
#[allow(dead_code)]
pub struct Config {
    pub gemini_api_key: String,
    pub rtmp_port: u16,
    pub http_flv_port: u16,
}

impl Config {
    #[allow(dead_code)]
    pub fn from_env() -> Self {
        let gemini_api_key = std::env::var("GEMINI_API_KEY")
            .expect("GEMINI_API_KEY must be set");

        let rtmp_port = std::env::var("RTMP_PORT")
            .unwrap_or("1935".to_string())
            .parse()
            .expect("RTMP_PORT must be a valid port number");

        let http_flv_port = std::env::var("HTTP_FLV_PORT")
            .unwrap_or("8888".to_string())
            .parse()
            .expect("HTTP_FLV_PORT must be a valid port number");

        Self {
            gemini_api_key,
            rtmp_port,
            http_flv_port,
        }
    }
}
