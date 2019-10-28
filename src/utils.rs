use log::info;
use std::fs;

pub fn clear_file(path: &String) {
    let res = fs::write(path, "");
    match res {
        Ok(_) => {}
        Err(_) => {
            let s = format!("Couldn't clear the file {}", path);
            info!("Couldn't clear the file {}", s)
        }
    }
}
