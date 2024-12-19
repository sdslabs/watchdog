#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Read;
    use lib::logger::log;

    #[test]
    fn test_log() {
        let filetype = "ssh";
        let status = "INFO";
        let message = "Test log message";

        log(filetype, status, message).expect("Failed to write log");

        let mut file = fs::File::open("ssh.logs").expect("Failed to open log file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read log file");

        assert!(contents.contains("Test log message"));
        assert!(contents.contains("INFO"));
    }
}