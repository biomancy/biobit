use crate::{Error, Result};
use std::fmt::{Display, Formatter};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

/// An existing directory stored as a canonical absolute path.
/// Relative file paths use this directory as their root; their files need not exist.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Root(Arc<PathBuf>);

impl Root {
    /// Resolves symlinks and requires an existing directory.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let canonical = std::fs::canonicalize(path)
            .map_err(|error| Error::io("resolve file path root", Some(path), error))?;
        let metadata = std::fs::metadata(&canonical)
            .map_err(|error| Error::io("inspect file path root", Some(path), error))?;
        if !metadata.is_dir() {
            return Err(Error::invalid("Root", "must be a directory"));
        }
        Ok(Self(Arc::new(canonical)))
    }

    /// Returns the canonical absolute directory.
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}

/// A UTF-8 path to a file. A relative path retains the root that resolves it.
/// File existence is not checked.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum FilePath {
    /// A path independent of a root.
    #[non_exhaustive]
    Absolute(PathBuf),
    /// A path resolved against its root.
    #[non_exhaustive]
    Relative { path: PathBuf, root: Root },
}

impl FilePath {
    /// Creates an absolute path to a file.
    pub fn from_absolute(value: impl Into<PathBuf>) -> Result<Self> {
        let value = value.into();
        validate_path(&value)?;
        if !value.is_absolute() {
            return Err(Error::invalid("FilePath", "must be absolute"));
        }
        Ok(Self::Absolute(value))
    }

    /// Creates a path to a file relative to `root`.
    pub fn from_relative(value: impl Into<PathBuf>, root: Root) -> Result<Self> {
        let value = value.into();
        validate_path(&value)?;
        if value.is_absolute() {
            return Err(Error::invalid("FilePath", "must be relative"));
        }
        Ok(Self::Relative { path: value, root })
    }

    /// Returns the stored path, without its root.
    ///
    /// Use [`Self::absolute`] when passing this location to filesystem operations.
    pub fn as_path(&self) -> &Path {
        match self {
            Self::Absolute(path) | Self::Relative { path, .. } => path,
        }
    }

    pub fn is_absolute(&self) -> bool {
        matches!(self, Self::Absolute(..))
    }

    pub fn is_relative(&self) -> bool {
        matches!(self, Self::Relative { .. })
    }

    /// Returns an absolute path without accessing the file.
    pub fn absolute(&self) -> PathBuf {
        match self {
            Self::Absolute(path) => path.clone(),
            // Keep `..`: removing it can change the target across a symlink.
            Self::Relative { path, root } => root.as_path().join(path),
        }
    }

    /// Expresses this path against a new root while preserving its target.
    pub(crate) fn rebase(&self, destination: &Root) -> Result<PathBuf> {
        match self {
            Self::Absolute(path) => Ok(path.clone()),
            Self::Relative { path, root } if root == destination => Ok(path.clone()),
            Self::Relative { .. } => {
                let path =
                    relative_to(&self.absolute(), destination.as_path()).ok_or_else(|| {
                        Error::invalid("FilePath", "cannot rebase relative to output directory")
                    })?;
                validate_path(&path)?;
                Ok(path)
            }
        }
    }
}

fn validate_path(value: &Path) -> Result<()> {
    if value.file_name().is_none() {
        return Err(Error::invalid("FilePath", "must name a file"));
    }

    let absolute = value.is_absolute();
    for component in value.components() {
        match component {
            Component::Prefix(_) | Component::RootDir if !absolute => {
                return Err(Error::invalid("FilePath", "must be absolute or relative"));
            }
            Component::Prefix(_) | Component::Normal(_) => {
                let text = component
                    .as_os_str()
                    .to_str()
                    .ok_or_else(|| Error::invalid("FilePath", "must be valid UTF-8"))?;
                if text.as_bytes().contains(&0) {
                    return Err(Error::invalid("FilePath", "must not contain NUL"));
                }
            }
            Component::RootDir | Component::CurDir | Component::ParentDir => {}
        }
    }

    if value
        .as_os_str()
        .as_encoded_bytes()
        .rsplit(|&byte| std::path::is_separator(byte as char))
        .next()
        .is_some_and(|part| part.is_empty() || part == b".")
    {
        return Err(Error::invalid("FilePath", "must name a file"));
    }
    Ok(())
}

fn relative_to(path: &Path, root: &Path) -> Option<PathBuf> {
    let path: Vec<_> = path.components().collect();
    let root: Vec<_> = root.components().collect();
    let common = path.iter().zip(&root).take_while(|(a, b)| a == b).count();
    if common == 0 {
        return None;
    }
    let mut relative = PathBuf::new();
    for _ in &root[common..] {
        relative.push("..");
    }
    for component in &path[common..] {
        relative.push(component.as_os_str());
    }
    Some(relative)
}

impl Display for FilePath {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        self.as_path().display().fmt(formatter)
    }
}

#[cfg(feature = "bitcode")]
mod bitcode_impl {
    //! PathBuf has no native bitcode Encode/Decode implementation, so Root and
    //! FilePath cannot derive those traits directly. This implementation uses
    //! bitcode's private codec API and may need adjustment when bitcode is updated.
    //!
    //! Both types store Unix paths as bytes, preserving their OS representation
    //! without UTF-8 conversion. Decoding trusts values encoded from valid
    //! objects and restores paths without domain validation or filesystem access.

    use super::*;
    use bitcode::__private::{Buffer, Decoder, Encoder, View};
    use bitcode::{Decode, Encode};
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    type EncodedPath = Vec<u8>;

    fn encode_path(path: &Path) -> EncodedPath {
        path.as_os_str().as_bytes().to_vec()
    }

    fn decode_path(value: EncodedPath) -> PathBuf {
        PathBuf::from(std::ffi::OsString::from_vec(value))
    }

    #[derive(Default)]
    pub struct RootEncoder(<EncodedPath as Encode>::Encoder);

    impl Buffer for RootEncoder {
        fn collect_into(&mut self, out: &mut Vec<u8>) {
            self.0.collect_into(out);
        }

        fn reserve(&mut self, additional: std::num::NonZeroUsize) {
            self.0.reserve(additional);
        }
    }

    impl Encoder<Root> for RootEncoder {
        fn encode(&mut self, value: &Root) {
            Encoder::<EncodedPath>::encode(&mut self.0, &encode_path(value.as_path()));
        }
    }

    #[derive(Default)]
    pub struct RootDecoder<'a>(<EncodedPath as Decode<'a>>::Decoder);

    impl<'a> View<'a> for RootDecoder<'a> {
        fn populate(
            &mut self,
            input: &mut &'a [u8],
            length: usize,
        ) -> std::result::Result<(), bitcode::Error> {
            self.0.populate(input, length)
        }
    }

    impl<'a> Decoder<'a, Root> for RootDecoder<'a> {
        fn decode(&mut self) -> Root {
            let path = decode_path(Decoder::<EncodedPath>::decode(&mut self.0));
            Root(Arc::new(path))
        }
    }

    impl Encode for Root {
        type Encoder = RootEncoder;
    }

    impl<'a> Decode<'a> for Root {
        type Decoder = RootDecoder<'a>;
    }

    #[derive(Encode, Decode)]
    pub enum FilePathValue {
        Absolute(EncodedPath),
        Relative { path: EncodedPath, root: Root },
    }

    impl Encoder<FilePath> for <FilePathValue as Encode>::Encoder {
        fn encode(&mut self, value: &FilePath) {
            let path = encode_path(value.as_path());
            let value = match value {
                FilePath::Absolute(_) => FilePathValue::Absolute(path),
                FilePath::Relative { root, .. } => FilePathValue::Relative {
                    path,
                    root: root.clone(),
                },
            };
            Encoder::<FilePathValue>::encode(self, &value);
        }
    }

    impl<'a> Decoder<'a, FilePath> for <FilePathValue as Decode<'a>>::Decoder {
        fn decode(&mut self) -> FilePath {
            match Decoder::<FilePathValue>::decode(self) {
                FilePathValue::Absolute(path) => FilePath::Absolute(decode_path(path)),
                FilePathValue::Relative { path, root } => FilePath::Relative {
                    path: decode_path(path),
                    root,
                },
            }
        }
    }

    impl Encode for FilePath {
        type Encoder = <FilePathValue as Encode>::Encoder;
    }

    impl<'a> Decode<'a> for FilePath {
        type Decoder = <FilePathValue as Decode<'a>>::Decoder;
    }
}

#[cfg(test)]
mod tests {
    use super::{FilePath, Root};
    use std::path::Path;

    #[cfg(all(feature = "bitcode", unix))]
    #[test]
    fn bitcode_preserves_non_utf8_roots() {
        use std::os::unix::ffi::OsStringExt;
        let path = std::env::temp_dir().join(std::ffi::OsString::from_vec(b"root-\xff".to_vec()));

        // Model the stored snapshot without requiring a directory for this codec test.
        let root = Root(std::sync::Arc::new(path.clone()));
        let restored = bitcode::decode::<Root>(&bitcode::encode(&root)).unwrap();

        assert_eq!(restored, root);
        assert_eq!(restored.as_path(), path);

        let file = FilePath::from_relative("reads.fq.gz", root).unwrap();
        assert_eq!(
            bitcode::decode::<FilePath>(&bitcode::encode(&file)).unwrap(),
            file
        );
    }

    #[test]
    fn roots_require_existing_directories_and_are_canonical() {
        let temporary = std::env::temp_dir();
        let root = Root::new(&temporary).unwrap();
        assert_eq!(root.as_path(), std::fs::canonicalize(temporary).unwrap());

        let executable = std::env::current_exe().unwrap();
        assert!(Root::new(&executable).is_err());
        assert!(Root::new(executable.join("missing")).is_err());
    }

    #[test]
    fn relative_paths_retain_their_root() {
        let root = Root::new(std::env::temp_dir()).unwrap();
        let path = FilePath::from_relative("reads/run1.fq.gz", root.clone()).unwrap();
        assert!(matches!(path, FilePath::Relative { .. }));
        assert_eq!(path.as_path(), Path::new("reads/run1.fq.gz"));
        assert_eq!(path.absolute(), root.as_path().join("reads/run1.fq.gz"));
    }

    #[test]
    fn absolute_paths_do_not_retain_a_root() {
        let absolute = std::env::temp_dir().join("reads.fq.gz");
        let path = FilePath::from_absolute(&absolute).unwrap();
        assert!(path.is_absolute());
        assert_eq!(path.absolute(), absolute);
    }

    #[test]
    fn rejects_invalid_paths() {
        let root = Root::new(std::env::temp_dir()).unwrap();
        for value in ["", ".", "..", "reads/.", "reads/..", "reads/", "a\0b"] {
            assert!(
                FilePath::from_relative(value, root.clone()).is_err(),
                "{value:?}"
            );
        }
        assert!(FilePath::from_absolute("reads.fq.gz").is_err());
        assert!(FilePath::from_relative(std::env::temp_dir().join("reads.fq.gz"), root).is_err());
    }
}
