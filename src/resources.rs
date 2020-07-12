pub mod resources;

use std::ffi;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug)] // Deriving Debug allows us to print ResourceError with the {:?} formatter
pub enum ResourceError {
    Io(io::error),
    FileContainsNil,
    FailedToGetExePath,
}

// Implement From to be able to convert io::Error into Resource error
impl From<io::Error> for ResourceError {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}

pub struct Resource {
    root_path: PathBuf,
}

impl Resource {
    pub fn from_exe_relative_path(rel_path: &Path) -> Result<Resource, Error> {
        // Grab the filename, or return if there's an error (? on Result)
        let exe_file_name =
            ::std::env::current_exe().map_err(|_| ResourceError::FailedToGetExePath)?;
        // Grab the path to the executable via .parent(), checking for errors
        let exe_path = exe_file_name
            .parent()
            .ok_or(ResourceError::FailedToGetExePath)?; // Transforms the Option given by .parent() into a Result, which we can match with ?

        // Return our resource
        Ok(Resource {
            root_path: exe_path.join(rel_path),
        })
    }

    pub fn load_cstring(&self, resource_name: &str) -> Result<ffi::CString, ResourceError> {
        // Open the file, and if the result is an error, return (? is shorthand for a match statement), otherwise store the data in file
        let mut file = fs::File::open(resource_name_to_path(&self.root_path, resource_name))?;
        // Create a byte buffer to read the file into (+1 size for null termination character)
        let mut buffer: Vec<u8> = Vec::with_capacity(file.metadata()?.len() as usize + 1);
        // Read the file's data into the buffer
        file.read_to_end(&mut buffer)?;
        // Check the file for interior 0 (null) bytes
        if buffer.iter().find(|i| **i == 0).is_some() {
            return Err(ResourceError::FileContainsNil);
        }
        // If everything went according to plan, return the buffer as a CString
        Ok(ffi::CString::from(buffer))
    }

    fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
        // Into is implemented on any type A where B::from(A) is implemented, which exists for Path (A) to PathBuf (B)
        let mut path: PathBuf = root_dir.into();
        // Construct a path by splitting the location by path separator
        for part in location.split("/") {
            path = path.join(part);
        }
        path
    }
}
