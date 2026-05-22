use aws_config::{BehaviorVersion, meta::region::RegionProviderChain};
use aws_sdk_s3::{
    Client,
    primitives::{AggregatedBytes, ByteStream},
};

use gilvave_settings::settings;

pub struct S3 {
    client: Client,
}

impl S3 {
    pub async fn new() -> Self {
        let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .endpoint_url(settings!().s3_url)
            .load()
            .await;

        Self {
            client: Client::new(&config),
        }
    }

    /// Загружает объект в хранилище
    async fn send(&self, bucket: &str, key: &str, body: ByteStream) -> anyhow::Result<()> {
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(body)
            .send()
            .await?;
        Ok(())
    }

    /// Скачивает объект из хранилища
    async fn get(&self, bucket: &str, key: &str) -> anyhow::Result<AggregatedBytes> {
        let resp = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;
        let data = resp.body.collect().await?;
        Ok(data)
    }

    /// Загружает статический объект (аватарки, иконки, смайлики и т.д.) в хранилище
    pub async fn send_static(&self, key: &str, body: ByteStream) -> anyhow::Result<()> {
        self.send("static", key, body).await
    }

    /// Скачивает статический объект (аватарки, иконки, смайлики и т.д.) из хранилища
    pub async fn get_static(&self, key: &str) -> anyhow::Result<AggregatedBytes> {
        self.get("static", key).await
    }

    /// Загружает изображение (картинки, скриншоты, мемы) в хранилище
    pub async fn send_image(&self, key: &str, body: ByteStream) -> anyhow::Result<()> {
        self.send("images", key, body).await
    }

    /// Скачивает изображение (картинки, скриншоты, мемы) из хранилища
    pub async fn get_image(&self, key: &str) -> anyhow::Result<AggregatedBytes> {
        self.get("images", key).await
    }

    /// Загружает видео в хранилище
    pub async fn send_video(&self, key: &str, body: ByteStream) -> anyhow::Result<()> {
        self.send("videos", key, body).await
    }

    /// Скачивает видео из хранилища
    pub async fn get_video(&self, key: &str) -> anyhow::Result<AggregatedBytes> {
        self.get("videos", key).await
    }

    /// Загружает файл (документы, архивы, логи) в хранилище
    pub async fn send_file(&self, key: &str, body: ByteStream) -> anyhow::Result<()> {
        self.send("files", key, body).await
    }

    /// Скачивает файл (документы, архивы, логи) из хранилища
    pub async fn get_file(&self, key: &str) -> anyhow::Result<AggregatedBytes> {
        self.get("files", key).await
    }
}
