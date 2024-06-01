pub fn main() {
    tonic_build::configure()
        .compile(&["service.proto"], &["."])
        .unwrap()
}
