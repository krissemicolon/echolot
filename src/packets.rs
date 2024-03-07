pub struct Initiation;

#[derive(Debug)]
pub struct Respnse {
    file_info_size: u32,
}

pub struct Agreement;

#[derive(Debug)]
pub struct FileInfo {
    pub file_name: String,
    pub file_size: u64,
    pub base64_content_size: usize,
    pub checksum: String,
}

#[derive(Debug)]
pub struct Confirmation {
    pub state: bool,
}

#[derive(Debug)]
pub struct FileTransmission {
    pub base64_content: String,
}
