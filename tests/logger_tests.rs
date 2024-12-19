#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Read;
    use lib::logger::log;

    #[test]
    fn test_log() {
        let filepath = "test_ssh.logs";
        let status = "INFO";
        let message = "Test log message";

        log(filepath, status, message).expect("Failed to write log");

        let mut file = fs::File::open(filepath).expect("Failed to open log file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read log file");

        assert!(contents.contains("Test log message"));
        assert!(contents.contains("INFO"));

        fs::remove_file(filepath).expect("Failed to delete test log file");
    }
}