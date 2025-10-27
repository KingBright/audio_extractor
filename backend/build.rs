fn main() {
    static_files::resource_dir("../frontend/build")
        .build()
        .unwrap();
}
