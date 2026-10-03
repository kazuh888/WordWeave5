#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

/// Synthetic RIFF builders; no recording or learning data is used.
pub fn pcm_format(channels: u16, sample_rate: u32) -> Vec<u8> {
    let align = channels * 2;
    let mut result = Vec::new();
    result.extend_from_slice(&1_u16.to_le_bytes());
    result.extend_from_slice(&channels.to_le_bytes());
    result.extend_from_slice(&sample_rate.to_le_bytes());
    result.extend_from_slice(&(sample_rate * u32::from(align)).to_le_bytes());
    result.extend_from_slice(&align.to_le_bytes());
    result.extend_from_slice(&16_u16.to_le_bytes());
    result
}

pub fn riff(chunks: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
    let mut result = b"RIFF\0\0\0\0WAVE".to_vec();
    for (tag, data) in chunks {
        result.extend_from_slice(*tag);
        result.extend_from_slice(&(data.len() as u32).to_le_bytes());
        result.extend_from_slice(data);
        if data.len() % 2 != 0 {
            result.push(0);
        }
    }
    let declared = (result.len() - 8) as u32;
    result[4..8].copy_from_slice(&declared.to_le_bytes());
    result
}

pub fn wav(channels: u16, sample_rate: u32, frames: usize) -> Vec<u8> {
    let format = pcm_format(channels, sample_rate);
    let samples = vec![0; frames * usize::from(channels) * 2];
    riff(&[(b"fmt ", &format), (b"data", &samples)])
}

pub fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

pub fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// Each test owns one new directory. Drop only removes that directory.
pub struct TempData {
    path: PathBuf,
}

impl TempData {
    pub fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "qwen-audio-test-{}-{stamp}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("create isolated test directory");
        Self { path }
    }

    pub fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        use std::io::Write;
        let path = self.path.join(name);
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .expect("create owned fixture");
        file.write_all(bytes).expect("write synthetic fixture");
        path
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempData {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).expect("remove owned test directory");
    }
}
