use std::time::UNIX_EPOCH;

use chrono::{DateTime, Local};
use axum::extract::State;
use maud::{Markup, html};

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", size, UNITS[unit])
}

fn format_time(secs: u64) -> String {
    let dt = DateTime::from_timestamp(secs as i64, 0)
        .unwrap_or_default()
        .with_timezone(&Local);
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub async fn page_handler(State(dir): State<String>) -> Markup {
    let files: Vec<(String, String, String)> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let meta = std::fs::metadata(e.path()).ok()?;
                let size = format_size(meta.len());
                let modified = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| format_time(d.as_secs()))
                    .unwrap_or_default();
                Some((name, size, modified))
            })
            .collect(),
        Err(_) => vec![],
    };

    // no sorting needed, display as-is

    let has_files = !files.is_empty();

    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "ShareFiles" }
                style {
                    (maud::PreEscaped(r#"
                        * { margin: 0; padding: 0; box-sizing: border-box; }
                        body {
                            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
                            background: linear-gradient(135deg, #0f0f1a 0%, #1a1a2e 50%, #16213e 100%);
                            min-height: 100vh;
                            padding: 40px 20px;
                            color: #c0c0d0;
                        }
                        .container {
                            max-width: 680px;
                            margin: 0 auto;
                        }
                        .card {
                            background: #1e1e2e;
                            border-radius: 16px;
                            box-shadow: 0 20px 60px rgba(0,0,0,.4);
                            padding: 40px;
                            margin-bottom: 24px;
                        }
                        h1 {
                            font-size: 28px;
                            font-weight: 700;
                            color: #e0e0f0;
                            margin-bottom: 4px;
                        }
                        .subtitle {
                            color: #6c6c8a;
                            font-size: 14px;
                            margin-bottom: 28px;
                        }
                        .file-list {
                            list-style: none;
                        }
                        .file-item {
                            display: flex;
                            align-items: center;
                            justify-content: space-between;
                            padding: 14px 16px;
                            border-radius: 10px;
                            transition: background .15s;
                            text-decoration: none;
                            color: inherit;
                            gap: 12px;
                        }
                        .file-item:hover {
                            background: #2a2a40;
                        }
                        .file-item + .file-item {
                            margin-top: 4px;
                        }
                        .file-icon {
                            font-size: 20px;
                            flex-shrink: 0;
                        }
                        .file-info {
                            flex: 1;
                            min-width: 0;
                        }
                        .file-name {
                            font-weight: 500;
                            font-size: 15px;
                            color: #e0e0f0;
                            word-break: break-all;
                        }
                        .file-meta {
                            font-size: 12px;
                            color: #6c6c8a;
                            margin-top: 2px;
                        }
                        .file-size {
                            font-size: 13px;
                            color: #8c8caa;
                            white-space: nowrap;
                            flex-shrink: 0;
                        }
                        .empty-state {
                            text-align: center;
                            padding: 40px 0;
                            color: #6c6c8a;
                        }
                        .empty-state .icon {
                            font-size: 48px;
                            margin-bottom: 12px;
                        }
                        .empty-state p {
                            font-size: 15px;
                        }
                        h2 {
                            font-size: 18px;
                            font-weight: 600;
                            color: #e0e0f0;
                            margin-bottom: 16px;
                        }
                        .upload-area {
                            border: 2px dashed #3d3d5c;
                            border-radius: 12px;
                            padding: 32px;
                            text-align: center;
                            transition: border-color .2s, background .2s;
                            cursor: pointer;
                        }
                        .upload-area:hover {
                            border-color: #7c6ff0;
                            background: #232338;
                        }
                        .upload-area .icon {
                            font-size: 36px;
                            margin-bottom: 8px;
                        }
                        .upload-area p {
                            color: #6c6c8a;
                            font-size: 14px;
                            margin-bottom: 16px;
                        }
                        .upload-area input[type="file"] {
                            display: block;
                            margin: 0 auto 12px;
                            font-size: 14px;
                            color: #aaa;
                        }
                        .upload-area input[type="file"]::file-selector-button {
                            background: #7c6ff0;
                            color: #fff;
                            border: none;
                            border-radius: 8px;
                            padding: 8px 20px;
                            font-size: 14px;
                            cursor: pointer;
                            margin-right: 12px;
                            transition: background .15s;
                        }
                        .upload-area input[type="file"]::file-selector-button:hover {
                            background: #6b5de0;
                        }
                        .btn-upload {
                            background: #7c6ff0;
                            color: #fff;
                            border: none;
                            border-radius: 8px;
                            padding: 10px 32px;
                            font-size: 15px;
                            font-weight: 500;
                            cursor: pointer;
                            transition: background .15s, transform .1s;
                        }
                        .btn-upload:hover {
                            background: #6b5de0;
                        }
                        .btn-upload:active {
                            transform: scale(.97);
                        }
                        .footer {
                            text-align: center;
                            color: rgba(255,255,255,.3);
                            font-size: 13px;
                            margin-top: 24px;
                        }
                        @media (max-width: 480px) {
                            body { padding: 16px 12px; }
                            .card { padding: 24px 16px; }
                            h1 { font-size: 22px; }
                            .file-item { flex-wrap: wrap; }
                        }
                    "#))
                }
            }
            body {
                .container {
                    .card {
                        h1 { "ShareFiles" }
                        p.subtitle { "上传文件并分享给局域网中的其他设备" }
                    }

                    .card {
                        h2 { "📁 文件列表" }
                        @if has_files {
                            .file-list {
                                @for (name, size, time) in &files {
                                    a.file-item href={ "/download/"(name) } {
                                        span.file-icon { "📄" }
                                        .file-info {
                                            .file-name { (name) }
                                            .file-meta { (time) }
                                        }
                                        span.file-size { (size) }
                                    }
                                }
                            }
                        } @else {
                            .empty-state {
                                .icon { "📂" }
                                p { "暂无文件" }
                            }
                        }
                    }

                    .card {
                        h2 { "⬆️ 上传文件" }
                        form action="/upload" method="post" enctype="multipart/form-data" {
                            .upload-area {
                                .icon { "☁️" }
                                p { "点击选择或拖拽文件到此处" }
                                input type="file" name="file";
                                button.btn-upload { "上传" }
                            }
                        }
                    }

                    .footer { "ShareFiles — 局域网文件分享" }
                }
            }
        }
    }
}
