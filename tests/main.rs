#[cfg(test)]
mod tests {
     uniffi::build_foreign_language_testcases!(
         "tests/test_networking.swift",
         "tests/test_networking.kts"
     );
     uniffi::build_foreign_language_testcases!(
         "tests/test_async_stream_from_rust.swift",
     );
     uniffi::build_foreign_language_testcases!(
         "tests/test_async_stream_from_swift.swift",
         "tests/test_async_stream_from_kotlin.kts",
     );
     uniffi::build_foreign_language_testcases!(
         "tests/test_file_io.swift",
         "tests/test_file_io.kts",
     );
}
