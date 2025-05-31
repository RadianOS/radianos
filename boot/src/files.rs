use alloc::vec::Vec;
use uefi::{boot::{self, ScopedProtocol}, fs::{FileSystem, FileSystemResult}, proto::media::fs::SimpleFileSystem, CString16};

pub fn read_file(path: &str) -> FileSystemResult<Vec<u8>> {
    let path: CString16 = CString16::try_from(path).unwrap();
    let fs: ScopedProtocol<SimpleFileSystem> = boot::get_image_file_system(boot::image_handle()).unwrap();
    let mut fs = FileSystem::new(fs);
    fs.read(path.as_ref())
}