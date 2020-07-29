use std::{
    ffi::CString,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

/// Errors relating to `Resource`
#[derive(Debug)]
pub enum ResourceError {
    Io(io::Error),
    FileContainsNil,
    FailedToGetExePath,
    DeserializationFailure,
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ResourceError {}

impl From<io::Error> for ResourceError {
    fn from(other: io::Error) -> Self {
        ResourceError::Io(other)
    }
}

/// A `Resource` which points to and loads from a directory containing resources for the application
pub struct Resource {
    root_path: PathBuf,
}

impl Resource {
    /// Create a new `Resource` to the given directory.
    ///
    /// ### Parameters
    ///
    /// - `rel_path`: The relative path from the project executable to the resource directory.
    ///
    /// ### Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: A `Resource` to use to access assets within the folder it points to.
    /// - `Err`: A `ResourceError` describing the various IO errors that may have occurred during creation of the `Resource`.
    pub fn new(rel_path: &Path) -> Result<Resource, ResourceError> {
        // Grab the filename, or return if there's an error (? on Result)
        let exe_filename =
            std::env::current_exe().map_err(|_| ResourceError::FailedToGetExePath)?;
        // Grab the path to the executable via .parent(), checking for errors
        let exe_path = exe_filename
            .parent()
            .ok_or(ResourceError::FailedToGetExePath)?;
        // Return our resource
        Ok(Resource {
            root_path: exe_path.join(rel_path),
        })
    }

    /// Load the given file inside this `Resource`'s root path and return the data in a byte vector.
    ///
    /// ### Parameters
    ///
    /// - `resource_name`: The filename of the resource to load into memory.
    ///
    /// ### Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: A `Vec<u8>` containing the raw data bytes of the resource file in question.
    /// - `Err`: A `ResourceError` describing the various IO errors that may have occurred during loading of the resource file.
    pub fn load_to_bytes(
        &self,
        resource_name: &str,
        check_for_interior_null: bool,
    ) -> Result<Vec<u8>, ResourceError> {
        let mut file = fs::File::open(self.path_for(resource_name))?;
        // File buffer of size +1 for null termination character
        let mut buffer: Vec<u8> = Vec::with_capacity(file.metadata()?.len() as usize + 1);
        file.read_to_end(&mut buffer)?;
        if check_for_interior_null {
            // Check the file for interior 0 (null) bytes
            if buffer.iter().find(|i| **i == 0).is_some() {
                return Err(ResourceError::FileContainsNil);
            }
        }
        Ok(buffer)
    }

    /// Load the given file inside this `Resource`'s root path and return the data as a `CString`.
    ///
    /// ### Parameters
    ///
    /// - `resource_name`: The filename of the resource to load into memory.
    ///
    /// ### Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: A `CString` containing the raw data of the resource file in question.
    /// - `Err`: A `ResourceError` describing the various IO errors that may have occurred during loading of the resource file.
    pub fn load_to_cstring(
        &self,
        resource_name: &str,
        check_for_interior_null: bool,
    ) -> Result<CString, ResourceError> {
        // These file bytes should return a `ResourceError` if there are any interior null bytes
        let file_bytes = self.load_to_bytes(resource_name, check_for_interior_null)?;
        let cstr = unsafe { CString::from_vec_unchecked(file_bytes) };
        Ok(cstr)
    }

    /// Load the given file inside this `Resource`'s root path and return the data as a `String`.
    ///
    /// ### Parameters
    ///
    /// - `resource_name`: The filename of the resource to load into memory.
    ///
    /// ### Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: A `String` containing the utf-8 data of the resource file in question.
    /// - `Err`: A `ResourceError` describing the various IO errors that may have occurred during loading of the resource file.
    pub fn load_to_string(&self, resource_name: &str) -> Result<String, ResourceError> {
        Ok(fs::read_to_string(self.path_for(resource_name))?)
    }

    /// Returns a `PathBuf` representing the full path to the given resource.
    pub fn path_for(&self, resource_name: &str) -> PathBuf {
        let mut path = PathBuf::from(&self.root_path);
        for path_component in resource_name.split("/") {
            path = path.join(path_component);
        }
        path
    }
}
