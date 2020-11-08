use hive::sgf_parser::read_sgf_file;

fn main() {
    std::fs::read_dir("./test_data")
        .expect("failed to open dir")
        .flat_map(|entry| entry)
        .for_each(|entry| { read_sgf_file(entry.path()); });
}
