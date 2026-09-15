/// The concrete filesystem type, for a caller that must name one in a signature.
///
/// An alias rather than a re-export, and the difference is the point. A caller writing
/// `nomos_composer_std::FileSystemType` names *this platform's* filesystem, which is what a
/// composer exists to let it say; a caller writing `StdFileSystem` names one implementation,
/// which is what every host did before this crate and what leaves it unable to be handed a
/// different one.
pub type FileSystemType = nomos_platform_std::StdFileSystem;
