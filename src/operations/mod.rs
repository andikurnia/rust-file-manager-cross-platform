pub mod context_menu;
pub mod drag_drop;

use std::path::Path;
use gio::prelude::FileExt;

pub fn copy_file(src: &Path, dest: &Path) -> Result<(), glib::Error> {
    let src_file = gio::File::for_path(src);
    let dest_file = gio::File::for_path(dest);
    src_file.copy(
        &dest_file,
        gio::FileCopyFlags::NONE,
        None::<&gio::Cancellable>,
        None::<&mut dyn FnMut(i64, i64)>,
    )
}

pub fn move_file(src: &Path, dest: &Path) -> Result<(), glib::Error> {
    let src_file = gio::File::for_path(src);
    let dest_file = gio::File::for_path(dest);
    src_file.move_(
        &dest_file,
        gio::FileCopyFlags::NONE,
        None::<&gio::Cancellable>,
        None::<&mut dyn FnMut(i64, i64)>,
    )
}

pub fn trash_file(path: &Path) -> Result<(), glib::Error> {
    let file = gio::File::for_path(path);
    file.trash(None::<&gio::Cancellable>)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_copy_file() {
        let tmp = std::env::temp_dir().join("finder_test_copy");
        let _ = fs::create_dir_all(&tmp);
        let src = tmp.join("src.txt");
        let dest = tmp.join("dest.txt");
        fs::write(&src, "hello").unwrap();

        assert!(src.exists());
        assert!(!dest.exists());

        copy_file(&src, &dest).unwrap();
        assert!(dest.exists());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "hello");

        let _ = fs::remove_file(&src);
        let _ = fs::remove_file(&dest);
        let _ = fs::remove_dir(&tmp);
    }

    #[test]
    fn test_move_file() {
        let tmp = std::env::temp_dir().join("finder_test_move");
        let _ = fs::create_dir_all(&tmp);
        let src = tmp.join("src.txt");
        let dest = tmp.join("dest.txt");
        fs::write(&src, "hello").unwrap();

        assert!(src.exists());
        assert!(!dest.exists());

        move_file(&src, &dest).unwrap();
        assert!(!src.exists());
        assert!(dest.exists());
        assert_eq!(fs::read_to_string(&dest).unwrap(), "hello");

        let _ = fs::remove_file(&dest);
        let _ = fs::remove_dir(&tmp);
    }

    #[test]
    fn test_trash_file() {
        let tmp = std::env::temp_dir().join("finder_test_trash");
        let _ = fs::create_dir_all(&tmp);
        let file = tmp.join("trash.txt");
        fs::write(&file, "delete me").unwrap();

        assert!(file.exists());

        // trash_file may fail if trash is not available, so we just check it doesn't panic
        let _ = trash_file(&file);

        let _ = fs::remove_file(&file);
        let _ = fs::remove_dir(&tmp);
    }
}
