use std::fmt::{Display, Formatter};
use std::path::Path;
use async_compression::Level;
use async_zip::Compression;
use async_zip::error::ZipError;
use async_zip::write::{EntryOptions, ZipFileWriter};
use tokio::fs::File;
use tokio::io::BufReader;
use clap::Parser;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
struct Args {
    /// Output .zip file
    #[clap(short, long)]
    output: String,

    /// Input directory
    #[clap(short, long, default_value = ".")]
    input: String,
}

#[derive(Debug)]
struct WrongFilenameError;

impl WrongFilenameError {
    pub fn new() -> Self {
        Self {}
    }
}

impl Display for WrongFilenameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not UTF-8")
    }
}


#[derive(Debug)]
enum MyError {
    StdIo(std::io::Error),
    WalkDir(walkdir::Error),
    WrongFilename(WrongFilenameError),
    Zip(ZipError),
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StdIo(err) => write!(f, "File error: {err}"),
            Self::WalkDir(err) => write!(f, "Traversing directories error: {err}"),
            Self::WrongFilename(err) => write!(f, "Wrong filename: {err}"),
            Self::Zip(err) => write!(f, "ZIP error: {err}"),
        }
    }
}

impl From<std::io::Error> for MyError {
    fn from(value: std::io::Error) -> Self {
        Self::StdIo(value)
    }
}

impl From<walkdir::Error> for MyError {
    fn from(value: walkdir::Error) -> Self {
        Self::WalkDir(value)
    }
}

impl From<WrongFilenameError> for MyError {
    fn from(value: WrongFilenameError) -> Self {
        Self::WrongFilename(value)
    }
}

impl From<ZipError> for MyError {
    fn from(value: ZipError) -> Self {
        Self::Zip(value)
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = almost_main().await {
        println!("Error: {err}");
    }
}

async fn almost_main() -> Result<(), MyError> {
    let args = Args::parse();

    let mut file = File::create(args.output).await?;
    let mut writer = ZipFileWriter::new(&mut file);

    for entry in WalkDir::new(args.input.clone()).into_iter() {
        let entry = entry?;
        // if entry.path_is_symlink() // FIXME
        if entry.file_type().is_dir() {
            continue;
        }
        let mut file = File::open(Path::new(args.input.as_str()).join(entry.path())).await?;

        let file_reader = tokio::io::BufReader::new(&mut file);
        let compressed_reader =
            async_compression::tokio::bufread::BrotliEncoder::with_quality(file_reader, Level::Best);

        let opts =
            EntryOptions::new(
                entry.path().to_str().ok_or::<MyError>(WrongFilenameError::new().into())?.to_string(),
                Compression::Stored
            )
                .extra(Vec::from([0u8; 32]));

        let mut entry_writer = writer.write_entry_stream(opts).await?;
        tokio::io::copy(&mut BufReader::new(compressed_reader), &mut entry_writer).await?;

        entry_writer.close().await?;
    }
    writer.close().await?;

    Ok(())
}
