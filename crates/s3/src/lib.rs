use aws_config::{BehaviorVersion, defaults, meta::region::RegionProviderChain};
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    Client,
    config::Builder as S3ConfigBuilder,
    primitives::{AggregatedBytes, ByteStream},
};

use gilvave_settings::settings;

#[derive(Clone)]
pub struct S3 {
    client: Client,
}

impl S3 {
    pub async fn new() -> Self {
        let credentials = Credentials::new(
            settings!().s3_access_key,
            settings!().s3_secret_key,
            None,
            None,
            "seaweedfs-provider",
        );

        let base_config = defaults(BehaviorVersion::latest())
            .region(RegionProviderChain::default_provider().or_else("us-east-1"))
            .endpoint_url(settings!().s3_url)
            .credentials_provider(credentials)
            .load()
            .await;

        let s3_config = S3ConfigBuilder::from(&base_config)
            .force_path_style(true)
            .build();

        Self {
            client: Client::from_conf(s3_config),
        }
    }

    /// Создание бакета
    pub async fn create_bucket(&self, bucket: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client.create_bucket().bucket(bucket).send().await?;
        Ok(())
    }

    /// Загрузка объекта
    async fn send(
        &self,
        bucket: &str,
        key: &str,
        body: ByteStream,
        content_type: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut req = self.client.put_object().bucket(bucket).key(key).body(body);

        if let Some(ct) = content_type {
            req = req.content_type(ct);
        }

        req.send().await?;
        Ok(format!("{}/{}/{}", settings!().s3_url, bucket, key))
    }

    /// Получение объекта
    async fn get(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<AggregatedBytes, Box<dyn std::error::Error + Send + Sync>> {
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

    /// Удаление объекта
    pub async fn delete_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }

    /// Удаление изображения
    pub async fn delete_image(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.delete_object("images", key).await
    }

    /// Удаление видео
    pub async fn delete_video(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.delete_object("videos", key).await
    }

    /// Удаление файла
    pub async fn delete_file(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.delete_object("files", key).await
    }

    /// Загрузка статики
    pub async fn send_static(
        &self,
        key: &str,
        body: ByteStream,
        content_type: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.send("static", key, body, content_type).await
    }

    /// Загрузка изображения
    pub async fn send_image(
        &self,
        key: &str,
        body: ByteStream,
        content_type: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.send("images", key, body, content_type).await
    }

    /// Загрузка видео
    pub async fn send_video(
        &self,
        key: &str,
        body: ByteStream,
        content_type: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.send("videos", key, body, content_type).await
    }

    /// Загрузка файла
    pub async fn send_file(
        &self,
        key: &str,
        body: ByteStream,
        content_type: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.send("files", key, body, content_type).await
    }

    /// Получение статики
    pub async fn get_static(
        &self,
        key: &str,
    ) -> Result<AggregatedBytes, Box<dyn std::error::Error + Send + Sync>> {
        self.get("static", key).await
    }

    /// Получение изображения
    pub async fn get_image(
        &self,
        key: &str,
    ) -> Result<AggregatedBytes, Box<dyn std::error::Error + Send + Sync>> {
        self.get("images", key).await
    }

    /// Получение видео
    pub async fn get_video(
        &self,
        key: &str,
    ) -> Result<AggregatedBytes, Box<dyn std::error::Error + Send + Sync>> {
        self.get("videos", key).await
    }

    /// Получение файла
    pub async fn get_file(&self, key: &str) -> Result<AggregatedBytes, Box<dyn std::error::Error + Send + Sync>> {
        self.get("files", key).await
    }

    /// Получение из бакета
    pub async fn get_by_bucket(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<AggregatedBytes, Box<dyn std::error::Error + Send + Sync>> {
        self.get(bucket, key).await
    }
}
