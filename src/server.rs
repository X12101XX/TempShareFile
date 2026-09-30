use chrono::{self, Local};

use axum::{
    Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};

use local_ip_address::{local_ip, local_ipv6};
use qr_terminal::TermQrCode;

use crate::index;

async fn file_handler(Path(filename): Path<String>, State(dir): State<String>) -> impl IntoResponse {
    let now = Local::now().to_string();
    println!("[{}] 下载: {}", now, filename);

    if filename.contains('/') || filename.contains("..") || filename.contains('\\') {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let path = format!("{}/{}", dir, filename);

    match tokio::fs::read(&path).await {
        Ok(content) => Response::builder()
            .header("Content-Type", "application/octet-stream")
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", filename),
            )
            .body(axum::body::Body::from(content))
            .unwrap(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn upload_handler(State(dir): State<String>, mut multipart: Multipart) -> impl IntoResponse {
    use tokio::io::AsyncWriteExt;

    while let Ok(Some(mut filed)) = multipart.next_field().await {
        let name = filed.file_name().unwrap_or("uploaded").to_string();
        let path = format!("{}/{}", dir, name);

        if name.contains('/') || name.contains("..") || name.contains('\\') {
            eprintln!("非法的文件名: {}", name);
            continue;
        }

        let mut file = match tokio::fs::File::create(&path).await {
            Ok(f) => f,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("创建文件失败: {}", e),
                )
                    .into_response();
            }
        };

        loop {
            match filed.chunk().await {
                Ok(Some(data)) => {
                    if let Err(e) = file.write_all(&data).await {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("写入失败: {}", e),
                        )
                            .into_response();
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("读取上传失败: {}", e),
                    )
                        .into_response();
                }
            }
        }
    }
    "上传成功\n".into_response()
}

pub async fn open_server(port: u16, dir: String) -> Result<(), Box<dyn std::error::Error>> {
    tokio::fs::create_dir_all(&dir).await?;

    let ip_v4 = local_ip()?;

    // IPv6 仅用于生成二维码；无全局 IPv6 时跳过，不影响服务启动
    if let Ok(ip_v6) = local_ipv6() {
        let url_v6 = format!("http://[{}]:{port}", ip_v6);

        let code = TermQrCode::from_bytes(url_v6.as_bytes());

        println!("服务已启动：\n{}\nhttp://{}:{port}", &url_v6, ip_v4);
        println!("二维码为ipv6地址");
        code.print();
    } else {
        println!("服务已启动：\nhttp://{}:{port}", ip_v4);
        println!("未获取到可用的 IPv6 地址，已跳过二维码");
    }
    println!("按 Ctrl+C 停止");
    
    let app = Router::new()
        .route("/", get(index::page_handler))
        .route("/download/{filename}", get(file_handler))
        .route("/upload", post(upload_handler))
        .with_state(dir)
        .layer(DefaultBodyLimit::disable());

    let listener = tokio::net::TcpListener::bind(format!("[::]:{port}")).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
