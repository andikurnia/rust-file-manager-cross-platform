use std::path::PathBuf;
use gio::prelude::FileExt;
use gio::prelude::FileEnumeratorExt;
use gio::prelude::ListModelExt;

#[derive(Debug, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub mime_type: String,
}

impl FileItem {
    pub fn from_gio_file(file: &gio::File, info: &gio::FileInfo) -> Self {
        let name = info.name().to_string_lossy().to_string();
        let path = file.path().unwrap_or_else(|| PathBuf::from(&name));
        let is_dir = info.file_type() == gio::FileType::Directory;
        let size = if is_dir { 0 } else { info.size() as u64 };
        let modified = info.modification_date_time().map(|dt| dt.to_unix()).unwrap_or(0);
        let mime_type = info.content_type().unwrap_or_else(|| "application/octet-stream".into());
        let mime_type = mime_type.to_string();

        Self { name, path, is_dir, size, modified, mime_type }
    }

    pub fn icon_name(&self) -> String {
        if self.is_dir {
            return "folder".to_string();
        }
        match self.mime_type.as_str() {
            m if m.starts_with("image/") => "image-x-generic",
            m if m.starts_with("text/") => "text-x-generic",
            m if m.starts_with("video/") => "video-x-generic",
            m if m.starts_with("audio/") => "audio-x-generic",
            m if m.starts_with("application/pdf") => "application-pdf",
            _ => "application-x-executable",
        }
        .to_string()
    }
}

pub async fn list_directory(path: &std::path::Path) -> Result<Vec<FileItem>, glib::Error> {
    let dir = gio::File::for_path(path);
    let enumerator = dir
        .enumerate_children(
            "standard::*,time::modified",
            gio::FileQueryInfoFlags::NONE,
            None::<&gio::Cancellable>,
        )?;

    let mut items = Vec::new();
    while let Some(info) = enumerator.next_file(None::<&gio::Cancellable>)? {
        let child = dir.child(info.name());
        items.push(FileItem::from_gio_file(&child, &info));
    }
    Ok(items)
}

/// Remove all items from a StringList (workaround for missing remove_all).
pub fn string_list_remove_all(list: &gtk4::StringList) {
    let n = list.n_items();
    for i in (0..n).rev() {
        list.remove(i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_item_icon_name_directory() {
        let item = FileItem {
            name: "test".to_string(),
            path: PathBuf::from("/tmp/test"),
            is_dir: true,
            size: 0,
            modified: 0,
            mime_type: "inode/directory".to_string(),
        };
        assert_eq!(item.icon_name(), "folder");
    }

    #[test]
    fn test_file_item_icon_name_image() {
        let item = FileItem {
            name: "photo.png".to_string(),
            path: PathBuf::from("/tmp/photo.png"),
            is_dir: false,
            size: 1024,
            modified: 0,
            mime_type: "image/png".to_string(),
        };
        assert_eq!(item.icon_name(), "image-x-generic");
    }

    #[test]
    fn test_file_item_icon_name_text() {
        let item = FileItem {
            name: "doc.txt".to_string(),
            path: PathBuf::from("/tmp/doc.txt"),
            is_dir: false,
            size: 100,
            modified: 0,
            mime_type: "text/plain".to_string(),
        };
        assert_eq!(item.icon_name(), "text-x-generic");
    }

    #[test]
    fn test_file_item_icon_name_pdf() {
        let item = FileItem {
            name: "doc.pdf".to_string(),
            path: PathBuf::from("/tmp/doc.pdf"),
            is_dir: false,
            size: 2048,
            modified: 0,
            mime_type: "application/pdf".to_string(),
        };
        assert_eq!(item.icon_name(), "application-pdf");
    }

    #[test]
    fn test_file_item_icon_name_unknown() {
        let item = FileItem {
            name: "data.bin".to_string(),
            path: PathBuf::from("/tmp/data.bin"),
            is_dir: false,
            size: 512,
            modified: 0,
            mime_type: "application/octet-stream".to_string(),
        };
        assert_eq!(item.icon_name(), "application-x-executable");
    }
}
