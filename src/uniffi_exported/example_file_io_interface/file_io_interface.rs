use crate::prelude::*;

#[derive(Object)]
pub struct FileIOInterface {
    writer: FFIOperationDispatcher<FFIFileIOWriteOutcomeListener>,
    reader: FFIOperationDispatcher<FFIFileIOReadOutcomeListener>,
}

#[export]
impl FileIOInterface {
    #[uniffi::constructor]
    pub fn new(
        file_writer: Arc<dyn FFIFileIOWriteExecutor>,
        file_reader: Arc<dyn FFIFileIOReadExecutor>,
    ) -> Self {
        Self {
            writer: FFIOperationDispatcher::<FFIFileIOWriteOutcomeListener>::new(file_writer),
            reader: FFIOperationDispatcher::<FFIFileIOReadOutcomeListener>::new(file_reader),
        }
    }

    pub(crate) async fn read(
        &self,
        file_absolute_path: String,
    ) -> Result<Option<Vec<u8>>, FFIBridgeError> {
        let response = self
            .reader
            .dispatch(FFIFileIOReadRequest::new(file_absolute_path))
            .await?;
        Ok(response.into())
    }

    pub(crate) async fn write(
        &self,
        file_absolute_path: String,
        contents: Vec<u8>,
        exists_strategy: FileAlreadyExistsStrategy,
    ) -> Result<FFIFileIOWriteResponse, FFIBridgeError> {
        self.writer
            .dispatch(FFIFileIOWriteRequest::new(
                file_absolute_path,
                contents,
                exists_strategy,
            ))
            .await
    }

    pub async fn write_to_new_or_extend_existing_file(
        &self,
        file_absolute_path: String,
        extend_strategy: ExtendExistingFileStrategy,
        contents: Vec<u8>,
    ) -> Result<FFIFileIOWriteResponse, FFIBridgeError> {
        let mut contents = contents;
        contents = self.read(file_absolute_path.clone()).await.map(|r| {
            if let Some(mut existing) = r {
                match extend_strategy {
                    ExtendExistingFileStrategy::Append { separator } => {
                        existing.extend(separator.as_bytes());
                        existing.extend(contents);
                        existing
                    }
                    ExtendExistingFileStrategy::Prepend { separator } => {
                        contents.extend(separator.as_bytes());
                        contents.extend(existing);
                        contents
                    }
                }
            } else {
                contents
            }
        })?;

        self.write(
            file_absolute_path,
            contents,
            FileAlreadyExistsStrategy::Overwrite,
        )
        .await
    }
}

#[derive(Enum, Clone, Debug, PartialEq, Eq)]
pub enum ExtendExistingFileStrategy {
    Append { separator: String },
    Prepend { separator: String },
}
