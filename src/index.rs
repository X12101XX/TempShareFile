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
        html lang="zh-CN" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                link rel="icon" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Crect width='16' height='16' fill='%23181818'/%3E%3Crect x='3' y='3' width='10' height='10' fill='%23ffdd33'/%3E%3C/svg%3E";
                title { "ShareFiles" }
                style {
                    (maud::PreEscaped(r#"
                        :root {
                            /* Gruber Darker palette */
                            --bg: #181818;
                            --bg-1: #101010;
                            --bg-hl: #282828;
                            --region: #453d41;
                            --fg: #e4e4ef;
                            --fg-bright: #f4f4ff;
                            --yellow: #ffdd33;
                            --orange: #cc8c3c;
                            --green: #73c936;
                            --red: #f43841;
                            --niagara: #96a6c8;
                            --quartz: #95a99f;
                        }
                        * { margin: 0; padding: 0; box-sizing: border-box; }
                        html { color-scheme: dark; }
                        ::selection { background: var(--region); color: var(--fg-bright); }
                        body {
                            font-family: ui-monospace, "JetBrains Mono", "Fira Code", "Cascadia Mono", "Iosevka", Menlo, Consolas, "Liberation Mono", monospace;
                            background: var(--bg);
                            color: var(--fg);
                            font-size: 14px;
                            line-height: 1.5;
                            min-height: 100vh;
                            padding: 48px 20px 24px;
                            display: flex;
                            flex-direction: column;
                        }
                        .container {
                            max-width: 720px;
                            width: 100%;
                            margin: 0 auto;
                            flex: 1;
                            display: flex;
                            flex-direction: column;
                        }
                        .masthead { margin-bottom: 28px; }
                        h1 {
                            font-size: 26px;
                            font-weight: 700;
                            color: var(--yellow);
                            letter-spacing: -0.02em;
                        }
                        .cursor {
                            display: inline-block;
                            width: 0.6em;
                            height: 1em;
                            margin-left: 2px;
                            background: var(--yellow);
                            vertical-align: -0.12em;
                            animation: blink 1.1s steps(1) infinite;
                        }
                        @keyframes blink { 50% { opacity: 0; } }
                        .subtitle { color: var(--orange); font-size: 13px; margin-top: 4px; }
                        .panel {
                            background: var(--bg-1);
                            border: 1px solid var(--bg-hl);
                            padding: 20px;
                            margin-bottom: 20px;
                        }
                        .panel-title {
                            color: var(--orange);
                            font-size: 12px;
                            margin-bottom: 14px;
                            user-select: none;
                        }
                        .file-list { list-style: none; }
                        .file-item {
                            display: grid;
                            grid-template-columns: minmax(0, 1fr) auto auto;
                            gap: 4px 20px;
                            align-items: baseline;
                            padding: 7px 10px;
                            text-decoration: none;
                            border-left: 2px solid transparent;
                            transition: background 0.12s ease, border-color 0.12s ease;
                        }
                        .file-item:hover { background: var(--bg-hl); border-left-color: var(--yellow); }
                        .file-name { color: var(--niagara); word-break: break-all; }
                        .file-item:hover .file-name { color: var(--fg-bright); text-decoration: underline; }
                        .file-time { color: var(--quartz); font-size: 12px; white-space: nowrap; }
                        .file-size {
                            color: var(--green);
                            font-size: 12px;
                            white-space: nowrap;
                            text-align: right;
                            min-width: 70px;
                        }
                        .empty-state { color: var(--orange); padding: 18px 10px; }
                        #dropzone {
                            border: 1px dashed var(--region);
                            padding: 26px 16px;
                            text-align: center;
                            cursor: pointer;
                            transition: border-color 0.15s ease, background 0.15s ease;
                            user-select: none;
                        }
                        #dropzone:hover, #dropzone.dragover {
                            border-color: var(--yellow);
                            background: rgba(255, 221, 51, 0.05);
                        }
                        #dropzone .arrow {
                            display: block;
                            color: var(--yellow);
                            font-size: 20px;
                            margin-bottom: 6px;
                        }
                        #dropzone .hint { color: var(--quartz); font-size: 13px; }
                        #selected {
                            display: none;
                            color: var(--fg-bright);
                            font-size: 12px;
                            margin-top: 10px;
                            word-break: break-all;
                        }
                        #file-input { display: block; margin: 14px auto 0; color: var(--quartz); font-size: 13px; }
                        #file-input::file-selector-button {
                            background: var(--yellow);
                            color: var(--bg);
                            border: none;
                            padding: 7px 18px;
                            font: inherit;
                            font-weight: 700;
                            cursor: pointer;
                            margin-right: 12px;
                        }
                        #file-input::file-selector-button:hover { background: var(--fg-bright); }
                        .controls {
                            display: flex;
                            align-items: center;
                            gap: 14px;
                            margin-top: 16px;
                            flex-wrap: wrap;
                        }
                        #btn-upload {
                            background: var(--yellow);
                            color: var(--bg);
                            border: none;
                            padding: 8px 26px;
                            font: inherit;
                            font-weight: 700;
                            cursor: pointer;
                            transition: background 0.12s ease;
                        }
                        #btn-upload:hover:not(:disabled) { background: var(--fg-bright); }
                        #btn-upload:disabled { opacity: 0.35; cursor: default; }
                        #progress {
                            display: none;
                            flex: 1;
                            min-width: 110px;
                            height: 5px;
                            background: var(--bg-hl);
                        }
                        #progress-fill {
                            height: 100%;
                            width: 0;
                            background: var(--yellow);
                            transition: width 0.1s linear;
                        }
                        .status { display: none; width: 100%; font-size: 13px; }
                        .status.ok { display: block; color: var(--green); }
                        .status.err { display: block; color: var(--red); }
                        .status.info { display: block; color: var(--quartz); }
                        .mode-line {
                            margin-top: auto;
                            background: var(--bg-1);
                            border: 1px solid var(--bg-hl);
                            padding: 9px 14px;
                            display: flex;
                            justify-content: space-between;
                            gap: 12px;
                            font-size: 12px;
                        }
                        .mode-line .brand { color: var(--yellow); font-weight: 700; }
                        .mode-line .desc { color: var(--quartz); }
                        :focus-visible { outline: 1px solid var(--yellow); outline-offset: 2px; }
                        @media (max-width: 560px) {
                            body { padding: 24px 12px 16px; }
                            h1 { font-size: 22px; }
                            .file-item { grid-template-columns: minmax(0, 1fr) auto; }
                            .file-time { display: none; }
                        }
                    "#))
                }
            }
            body {
                .container {
                    header.masthead {
                        h1 { "ShareFiles" span.cursor {} }
                        p.subtitle { ";; 上传文件并分享给局域网中的其他设备" }
                    }

                    .panel {
                        .panel-title { ";; 文件列表" }
                        @if has_files {
                            ul.file-list {
                                @for (name, size, time) in &files {
                                    li {
                                        a.file-item href={ "/download/"(name) } {
                                            span.file-name { (name) }
                                            span.file-time { (time) }
                                            span.file-size { (size) }
                                        }
                                    }
                                }
                            }
                        } @else {
                            .empty-state { ";; 暂无文件，拖一个上来吧" }
                        }
                    }

                    .panel {
                        .panel-title { ";; 上传文件" }
                        form id="upload-form" action="/upload" method="post" enctype="multipart/form-data" {
                            #dropzone {
                                span.arrow { "↑" }
                                p.hint { "拖拽文件到此处，或点击选择（支持多选）" }
                                #selected {}
                            }
                            input id="file-input" type="file" name="file" multiple;
                            .controls {
                                button.btn-upload id="btn-upload" type="submit" { "上传" }
                                #progress { #progress-fill {} }
                                .status {}
                            }
                        }
                    }

                    footer.mode-line {
                        span.brand { "ShareFiles" }
                        span.desc { "局域网文件分享" }
                    }
                }

                script {
                    (maud::PreEscaped(r#"
                        (function () {
                            var dz = document.getElementById('dropzone');
                            var input = document.getElementById('file-input');
                            var selected = document.getElementById('selected');
                            var form = document.getElementById('upload-form');
                            var btn = document.getElementById('btn-upload');
                            var progress = document.getElementById('progress');
                            var fill = document.getElementById('progress-fill');
                            var status = document.querySelector('.status');

                            input.style.display = 'none';

                            function fmt(n) {
                                var u = ['B', 'KB', 'MB', 'GB'], i = 0;
                                while (n >= 1024 && i < u.length - 1) { n /= 1024; i++; }
                                return n.toFixed(1) + ' ' + u[i];
                            }
                            function showSelected() {
                                var fs = input.files;
                                if (!fs || fs.length === 0) {
                                    selected.style.display = 'none';
                                    selected.textContent = '';
                                    return;
                                }
                                var total = 0, names = [];
                                for (var i = 0; i < fs.length; i++) {
                                    total += fs[i].size;
                                    names.push(fs[i].name);
                                }
                                selected.textContent = ';; 已选择 ' + fs.length + ' 个文件（共 ' + fmt(total) + '）：' + names.join('、');
                                selected.style.display = 'block';
                            }
                            function setStatus(text, kind) {
                                status.textContent = text;
                                status.className = 'status' + (kind ? ' ' + kind : '');
                            }

                            dz.addEventListener('click', function () { input.click(); });
                            input.addEventListener('change', showSelected);
                            ['dragenter', 'dragover'].forEach(function (ev) {
                                dz.addEventListener(ev, function (e) {
                                    e.preventDefault();
                                    dz.classList.add('dragover');
                                });
                            });
                            ['dragleave', 'drop'].forEach(function (ev) {
                                dz.addEventListener(ev, function (e) {
                                    e.preventDefault();
                                    dz.classList.remove('dragover');
                                });
                            });
                            dz.addEventListener('drop', function (e) {
                                if (e.dataTransfer && e.dataTransfer.files.length) {
                                    input.files = e.dataTransfer.files;
                                    showSelected();
                                }
                            });

                            form.addEventListener('submit', function (e) {
                                e.preventDefault();
                                if (!input.files || input.files.length === 0) {
                                    setStatus(';; 请先选择文件', 'err');
                                    return;
                                }
                                btn.disabled = true;
                                progress.style.display = 'block';
                                fill.style.width = '0%';
                                setStatus(';; 上传中…', 'info');

                                var xhr = new XMLHttpRequest();
                                xhr.open('POST', '/upload');
                                xhr.upload.addEventListener('progress', function (ev) {
                                    if (ev.lengthComputable) {
                                        fill.style.width = Math.round(ev.loaded / ev.total * 100) + '%';
                                    }
                                });
                                xhr.addEventListener('load', function () {
                                    btn.disabled = false;
                                    if (xhr.status >= 200 && xhr.status < 300) {
                                        fill.style.width = '100%';
                                        setStatus(';; 上传成功，正在刷新列表…', 'ok');
                                        setTimeout(function () { location.reload(); }, 700);
                                    } else {
                                        setStatus(';; 上传失败：' + xhr.responseText, 'err');
                                    }
                                });
                                xhr.addEventListener('error', function () {
                                    btn.disabled = false;
                                    setStatus(';; 上传失败：网络错误', 'err');
                                });
                                xhr.send(new FormData(form));
                            });
                        })();
                    "#))
                }
            }
        }
    }
}
