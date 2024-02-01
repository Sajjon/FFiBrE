pub fn main() {
    uniffi::generate_scaffolding("src/network.udl").expect("Build script panics can be ignored");
}
