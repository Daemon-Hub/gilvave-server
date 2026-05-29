use aws_sdk_s3::primitives::ByteStream;
use std::fs::File;
use std::io::Read;

use gilvave_s3::S3;

#[tokio::test]
async fn test_s3_upload_and_download_real_image() {
    gilvave_settings::setup_settings();
    let s3 = S3::new().await;

    // 1. Читаем НАСТОЯЩИЙ файл с диска
    let mut file = File::open("tests/test.png").expect("Не нашли test.png в папке tests");
    let mut original_data = Vec::new();
    file.read_to_end(&mut original_data)
        .expect("Не смогли прочитать файл");

    // 2. Загружаем его
    let byte_stream = ByteStream::from(original_data.clone());
    let key = "real_test_image.png";

    s3.send_image(key, byte_stream)
        .await
        .expect("Загрузка упала");

    // 3. Скачиваем обратно
    let downloaded = s3.get_image(key).await.expect("Скачивание упало");
    let downloaded_bytes = downloaded.into_bytes();

    // 4. Сравниваем байты оригинала и скачанного
    assert_eq!(
        downloaded_bytes.as_ref(),
        original_data.as_slice(),
        "Байты не совпадают!"
    );
}
