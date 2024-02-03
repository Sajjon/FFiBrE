pub fn main() {
    uniffi::generate_scaffolding("src/ffibre.udl").expect("Build script panics can be ignored");
}
