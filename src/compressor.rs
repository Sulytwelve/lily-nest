use crate::model::AssetsConfig;
use std::path::PathBuf;
use std::{fs, path::Path};
use tracing::{debug, info, warn};

// 获取需要处理的文件列表
fn get_target_files(config: &AssetsConfig) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for dir in &config.assets_dirs {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("无法读取目录 {}: {}", dir, e);
                continue;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // 只处理文件
            if !path.is_file() {
                continue;
            }

            // 检查扩展名是否匹配
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_string();
                if config.target_exts.contains(&ext_str) {
                    files.push(path);
                }
            }
        }
    }

    files
}

// 判断是否需要重新压缩
fn need_recompress(src: &Path, dst: &Path) -> bool {
    // 1. 如果目标压缩文件不存在，必须压
    let Ok(src_meta) = fs::metadata(src) else {
        return false;
    };
    let Ok(dst_meta) = fs::metadata(dst) else {
        return true;
    };

    // 2. 获取源文件和目标文件的修改时间
    let Ok(src_mtime) = src_meta.modified() else {
        return false;
    };
    let Ok(dst_mtime) = dst_meta.modified() else {
        return true;
    };

    // 3. 只有源文件比压缩文件“新”，才返回 true
    src_mtime > dst_mtime
}

// 压缩处理函数
fn process_compression(file_path: &PathBuf, config: &AssetsConfig) {
    // 读取文件内容
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            warn!("读取文件失败 {}: {}", file_path.display(), e);
            return;
        }
    };

    // 获取文件扩展名和基础名
    let ext = match file_path.extension() {
        Some(e) => e.to_string_lossy().to_string(),
        None => {
            warn!("文件无扩展名: {}", file_path.display());
            return;
        }
    };

    let Some(base_name) = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
    else {
        warn!("文件无文件名: {}", file_path.display());
        return;
    };
    let Some(parent_dir) = file_path.parent() else {
        warn!("文件无父目录: {}", file_path.display());
        return;
    };

    // 根据配置的压缩类型执行压缩
    for comp_type in &config.compression_types {
        let output_path = parent_dir.join(format!("{}.{}.{}", base_name, ext, comp_type));
        if !need_recompress(file_path, &output_path) {
            debug!("命中缓存: {}", output_path.display());
            continue;
        }
        info!(
            "检测到更新，正在压缩: {} -> {}",
            file_path.display(),
            output_path.display()
        );
        match comp_type.as_str() {
            "br" => compress_br(&data, &output_path, config.brotli_quality),
            "gz" => compress_gz(&data, &output_path, config.gzip_level),
            "zst" => compress_zst(&data, &output_path, config.zstd_level),
            _ => warn!("不支持的压缩类型: {}", comp_type),
        }
    }
}

// Zstandard 压缩
fn compress_zst(data: &[u8], output_path: &PathBuf, level: i32) {
    // 使用 zstd 的 stream::encode_all 进行一次性压缩[citation:6]
    match zstd::stream::encode_all(data, level) {
        Ok(compressed) => {
            if let Err(e) = fs::write(output_path, compressed) {
                warn!("Zstd 写入失败 {}: {}", output_path.display(), e);
            } else {
                info!("Zstd 成功: {}", output_path.display());
            }
        }
        Err(e) => warn!("Zstd 压缩失败 {}: {}", output_path.display(), e),
    }
}

// Brotli 压缩
fn compress_br(data: &[u8], output_path: &PathBuf, quality: u32) {
    let result = (|| {
        let mut out = Vec::new();
        let params = brotli::enc::BrotliEncoderParams {
            quality: quality as i32,
            ..Default::default()
        };
        brotli::BrotliCompress(&mut std::io::Cursor::new(data), &mut out, &params)?;
        fs::write(output_path, out)?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })();

    match result {
        Ok(_) => info!("Brotli 成功: {}", output_path.display()),
        Err(e) => warn!("Brotli 失败 {}: {}", output_path.display(), e),
    }
}

// Gzip 压缩
fn compress_gz(data: &[u8], output_path: &PathBuf, level: u32) {
    use flate2::{Compression, write::GzEncoder};
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));

    if let Err(e) = encoder
        .write_all(data)
        .and_then(|_| encoder.finish())
        .and_then(|c| fs::write(output_path, c))
    {
        warn!("Gzip 处理失败 {}: {}", output_path.display(), e);
    } else {
        info!("Gzip 成功: {}", output_path.display());
    }
}

pub fn ensure_precompressed_assets(config: &AssetsConfig) {
    info!("开始预压缩检查，目标目录: {:?}", config.assets_dirs);

    let files = get_target_files(config);
    info!("找到 {} 个需要处理的文件", files.len());

    for file in files {
        process_compression(&file, config);
    }

    info!("预压缩检查完成");
}
