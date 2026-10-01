use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Helper for binary file manipulation in modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryFile {
    pub filename: String,
    pub data: Vec<u8>,
}

impl BinaryFile {
    pub fn new(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            filename: filename.into(),
            data,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn read_u8(&self, offset: usize) -> Result<u8> {
        self.data
            .get(offset)
            .copied()
            .ok_or_else(|| anyhow!("Offset {} out of bounds for file len {}", offset, self.data.len()))
    }

    pub fn read_slice(&self, start: usize, end: usize) -> Result<&[u8]> {
        if start > end || end > self.data.len() {
            return Err(anyhow!(
                "Range {}..{} out of bounds for file len {}",
                start,
                end,
                self.data.len()
            ));
        }
        Ok(&self.data[start..end])
    }

    pub fn read_string(&self, start: usize, end: usize) -> Result<String> {
        let slice = self.read_slice(start, end)?;
        // Convert ASCII/UTF-8 lossy, trimming null and whitespace bytes
        let s = String::from_utf8_lossy(slice).trim_matches('\0').trim().to_string();
        Ok(s)
    }

    pub fn read_u16_be(&self, offset: usize) -> Result<u16> {
        let slice = self.read_slice(offset, offset + 2)?;
        let bytes: [u8; 2] = slice.try_into()?;
        Ok(u16::from_be_bytes(bytes))
    }

    pub fn read_u16_le(&self, offset: usize) -> Result<u16> {
        let slice = self.read_slice(offset, offset + 2)?;
        let bytes: [u8; 2] = slice.try_into()?;
        Ok(u16::from_le_bytes(bytes))
    }

    pub fn read_u24_le(&self, offset: usize) -> Result<u32> {
        let slice = self.read_slice(offset, offset + 3)?;
        Ok((slice[0] as u32) | ((slice[1] as u32) << 8) | ((slice[2] as u32) << 16))
    }

    pub fn read_u24_be(&self, offset: usize) -> Result<u32> {
        let slice = self.read_slice(offset, offset + 3)?;
        Ok(((slice[0] as u32) << 16) | ((slice[1] as u32) << 8) | (slice[2] as u32))
    }

    pub fn read_u32_be(&self, offset: usize) -> Result<u32> {
        let slice = self.read_slice(offset, offset + 4)?;
        let bytes: [u8; 4] = slice.try_into()?;
        Ok(u32::from_be_bytes(bytes))
    }

    pub fn read_u32_le(&self, offset: usize) -> Result<u32> {
        let slice = self.read_slice(offset, offset + 4)?;
        let bytes: [u8; 4] = slice.try_into()?;
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn write_bytes(&mut self, offset: usize, bytes: &[u8]) -> Result<()> {
        if offset + bytes.len() > self.data.len() {
            return Err(anyhow!(
                "Write at offset {} of {} bytes exceeds buffer len {}",
                offset,
                bytes.len(),
                self.data.len()
            ));
        }
        self.data[offset..offset + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    pub fn write_u24_le(&mut self, offset: usize, val: u32) -> Result<()> {
        let bytes = [
            (val & 0xFF) as u8,
            ((val >> 8) & 0xFF) as u8,
            ((val >> 16) & 0xFF) as u8,
        ];
        self.write_bytes(offset, &bytes)
    }

    pub fn write_u32_be(&mut self, offset: usize, val: u32) -> Result<()> {
        self.write_bytes(offset, &val.to_be_bytes())
    }

    pub fn write_u32_le(&mut self, offset: usize, val: u32) -> Result<()> {
        self.write_bytes(offset, &val.to_le_bytes())
    }

    pub fn fill(&mut self, start: usize, end: usize, byte_val: u8) -> Result<()> {
        let len = self.data.len();
        if start > end || end >= len {
            return Err(anyhow!(
                "Fill range {}..={} out of bounds for file len {}",
                start,
                end,
                len
            ));
        }
        self.data[start..=end].fill(byte_val);
        Ok(())
    }

    pub fn search_all(&self, pattern: &[u8]) -> Vec<usize> {
        if pattern.is_empty() || pattern.len() > self.data.len() {
            return Vec::new();
        }
        self.data
            .windows(pattern.len())
            .enumerate()
            .filter_map(|(idx, window)| if window == pattern { Some(idx) } else { None })
            .collect()
    }

    pub fn add_suffix(&self, suffix: &str) -> String {
        if let Some((stem, ext)) = self.filename.rsplit_once('.') {
            format!("{}{}.{}", stem, suffix, ext)
        } else {
            format!("{}{}", self.filename, suffix)
        }
    }
}

/// Response returned by module APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ApiResponse {
    Ok(serde_json::Value),
    Download { filename: String, data: Vec<u8> },
    Error(String),
}

impl ApiResponse {
    pub fn ok<T: Serialize>(value: T) -> Result<Self> {
        Ok(Self::Ok(serde_json::to_value(value)?))
    }

    pub fn download(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self::Download {
            filename: filename.into(),
            data,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self::Error(msg.into())
    }
}

/// UI Metadata descriptor for the frontend
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UIElement {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_for: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inner_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub review: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_file_open: Option<String>,
    pub ui: Vec<UIElement>,
    pub apis: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// The core trait every module must implement
pub trait Module: Send + Sync {
    fn metadata(&self) -> ModuleMetadata;
    fn execute(
        &mut self,
        api_name: &str,
        file: &mut BinaryFile,
        args: &serde_json::Value,
    ) -> Result<ApiResponse>;
}

// Function pointer signatures for FFI boundary
pub type CreateModuleFn = unsafe extern "C" fn() -> *mut Box<dyn Module>;
pub type DestroyModuleFn = unsafe extern "C" fn(*mut Box<dyn Module>);

/// Macro to make exporting modules a single clean line
#[macro_export]
macro_rules! export_module {
    ($module_type:ty) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn cspw_module_create() -> *mut Box<dyn $crate::Module> {
            let boxed: Box<dyn $crate::Module> = Box::new(<$module_type>::default());
            Box::into_raw(Box::new(boxed))
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn cspw_module_destroy(ptr: *mut Box<dyn $crate::Module>) {
            if !ptr.is_null() {
                unsafe {
                    drop(Box::from_raw(ptr));
                }
            }
        }
    };
}