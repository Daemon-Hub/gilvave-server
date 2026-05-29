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

    /// Создает бакет, если его не существует
    pub async fn create_bucket(&self, bucket: &str) -> anyhow::Result<()> {
        self.client.create_bucket().bucket(bucket).send().await?;
        Ok(())
    }

    /// Загружает объект в хранилище
    async fn send(
        &self,
        bucket: &str,
        key: &str,
        body: ByteStream,
        content_type: Option<&str>,
    ) -> anyhow::Result<String> {
        let mut req = self.client.put_object().bucket(bucket).key(key).body(body);

        if let Some(ct) = content_type {
            req = req.content_type(ct);
        }

        req.send().await?;
        Ok(format!("{}/{}/{}", settings!().s3_url, bucket, key))
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
    pub async fn send_static(
        &self,
        key: &str,
        body: ByteStream,
        content_type: Option<&str>,
    ) -> anyhow::Result<String> {
        self.send("static", key, body, content_type).await
    }

    /// Загружает изображение (картинки, скриншоты, мемы) в хранилище
    pub async fn send_image(&self, key: &str, body: ByteStream) -> anyhow::Result<String> {
        self.send("images", key, body, None).await
    }

    /// Загружает видео в хранилище
    pub async fn send_video(&self, key: &str, body: ByteStream) -> anyhow::Result<String> {
        self.send("videos", key, body, None).await
    }

    /// Загружает файл (документы, архивы, логи) в хранилище
    pub async fn send_file(&self, key: &str, body: ByteStream) -> anyhow::Result<String> {
        self.send("files", key, body, None).await
    }

    /// Скачивает статический объект (аватарки, иконки, смайлики и т.д.) из хранилища
    pub async fn get_static(&self, key: &str) -> anyhow::Result<AggregatedBytes> {
        self.get("static", key).await
    }

    /// Скачивает изображение (картинки, скриншоты, мемы) из хранилища
    pub async fn get_image(&self, key: &str) -> anyhow::Result<AggregatedBytes> {
        self.get("images", key).await
    }

    /// Скачивает видео из хранилища
    pub async fn get_video(&self, key: &str) -> anyhow::Result<AggregatedBytes> {
        self.get("videos", key).await
    }

    /// Скачивает файл (документы, архивы, логи) из хранилища
    pub async fn get_file(&self, key: &str) -> anyhow::Result<AggregatedBytes> {
        self.get("files", key).await
    }
}
