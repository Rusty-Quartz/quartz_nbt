use crate::NbtCompound;
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

/// An error associated with the translation of a NBT representation to a concrete type. This
/// can either be a structure error, meaning an error in the structure of the NBT tree, or a
/// custom error, which could occur when converting a tag into a concrete type. Most of the conversion
/// processes in this crate return a [`NbtStructureError`] when there is a type mismatch or missing tag.
///
/// [`NbtStructureError`]: crate::repr::NbtStructureError
#[derive(Debug)]
pub enum NbtReprError {
    /// A structure error in the tag tree.
    Structure(Box<NbtStructureError>),
    /// A custom error.
    Custom(anyhow::Error),
}

impl NbtReprError {
    /// Creates a new NBT representation error from the given structure error.
    pub fn structure(error: NbtStructureError) -> Self {
        NbtReprError::Structure(Box::new(error))
    }

    /// Creates a `NbtReprError` from the given error. If the given error is a [`NbtStructureError`],
    /// then the resulting representation error is of the `Structure` variant. If the error is a
    /// `NbtReprError` then it is downcasted and returned. All other error types are considered custom
    /// errors.
    ///
    /// ```
    /// # use quartz_nbt::*;
    /// use std::convert::TryFrom;
    /// use std::error::Error;
    ///
    /// let tag = NbtTag::Byte(0);
    /// let structure_error = NbtReprError::from_any(i32::try_from(tag).unwrap_err());
    /// assert!(matches!(structure_error, NbtReprError::Structure(..)));
    ///
    /// let nested_error = NbtReprError::from_any(structure_error);
    /// assert!(matches!(NbtReprError::from_any(nested_error), NbtReprError::Structure(..)));
    ///
    /// let custom_error = "abc".parse::<i32>().unwrap_err();
    /// assert!(matches!(NbtReprError::from_any(custom_error), NbtReprError::Custom(..)));
    /// ```
    pub fn from_any<E: Into<anyhow::Error>>(error: E) -> Self {
        let mut error = <E as Into<anyhow::Error>>::into(error);

        error = match error.downcast::<Self>() {
            Ok(error) => return error,
            Err(error) => error,
        };

        match error.downcast::<NbtStructureError>() {
            Ok(error) => NbtReprError::Structure(Box::new(error)),
            Err(error) => NbtReprError::Custom(error),
        }
    }
}

impl From<NbtStructureError> for NbtReprError {
    fn from(error: NbtStructureError) -> Self {
        NbtReprError::Structure(Box::new(error))
    }
}

impl From<Box<NbtStructureError>> for NbtReprError {
    fn from(error: Box<NbtStructureError>) -> Self {
        NbtReprError::Structure(error)
    }
}

impl Display for NbtReprError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NbtReprError::Structure(error) => Display::fmt(error, f),
            NbtReprError::Custom(custom) => Display::fmt(custom, f),
        }
    }
}

impl Error for NbtReprError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NbtReprError::Structure(error) => Some(error),
            NbtReprError::Custom(custom) => Some(&**custom),
        }
    }
}

/// An error associated with the structure of an NBT tag tree. This error represents a conflict
/// between the expected and actual structure of an NBT tag tree.
#[repr(transparent)]
pub struct NbtStructureError {
    repr: NbtStructureErrorRepr,
}

impl NbtStructureError {
    pub(crate) fn missing_tag<T: Into<String>>(tag_name: T) -> Self {
        NbtStructureError {
            repr: NbtStructureErrorRepr::MissingTag {
                tag_name: tag_name.into().into_boxed_str(),
            },
        }
    }

    pub(crate) fn invalid_index(index: usize, length: usize) -> Self {
        NbtStructureError {
            repr: NbtStructureErrorRepr::InvalidIndex { index, length },
        }
    }

    pub(crate) fn type_mismatch(expected: &'static str, found: &'static str) -> Self {
        NbtStructureError {
            repr: NbtStructureErrorRepr::TypeMismatch {
                expected: Box::new(expected),
                found: Box::new(found),
            },
        }
    }
}

impl Debug for NbtStructureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.repr, f)
    }
}

impl Display for NbtStructureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.repr {
            NbtStructureErrorRepr::MissingTag { tag_name } =>
                write!(f, "Missing tag \"{}\"", tag_name),
            NbtStructureErrorRepr::InvalidIndex { index, length } =>
                write!(f, "Index out of range: {} >= {}", index, length),
            NbtStructureErrorRepr::TypeMismatch { expected, found } => write!(
                f,
                "Tag type mismatch: expected {} but found {}",
                expected, found
            ),
        }
    }
}

impl Error for NbtStructureError {}

#[derive(Debug)]
enum NbtStructureErrorRepr {
    MissingTag {
        tag_name: Box<str>,
    },
    InvalidIndex {
        index: usize,
        length: usize,
    },
    // Keep the size of this type down to that of a wide pointer
    TypeMismatch {
        expected: Box<&'static str>,
        found: Box<&'static str>,
    },
}

/// Defines a type which has a full representation as a [`NbtCompound`].
///
/// Full representation meaning that the type can be constructed from a [`NbtCompound`], and fully serialized
/// as one as well.
///
/// [`NbtCompound`]: crate::tag::NbtCompound
pub trait NbtRepr: Sized {
    /// Updates the data in this type based on the given compound. The intention is that data is copied, not
    /// moved, from the compound to update this type.
    fn read_nbt(&mut self, nbt: &NbtCompound) -> Result<(), NbtReprError>;

    /// Writes all necessary data to the given compound to serialize this type.
    ///
    /// Although not enforced, the data written should allow for the type to be restored via the
    /// [`read_nbt`] function.
    ///
    /// [`read_nbt`]: crate::repr::NbtRepr::read_nbt
    fn write_nbt(&self, nbt: &mut NbtCompound);

    /// Converts this type into an owned [`NbtCompound`].
    ///
    /// Currently this is just a wrapper around creating an empty compound, proceeding to call [`write_nbt`] on
    /// a mutable reference to that compound, then returning the compound.
    ///
    /// [`NbtCompound`]: crate::tag::NbtCompound
    /// [`write_nbt`]: crate::repr::NbtRepr::write_nbt
    #[inline]
    fn to_nbt(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        self.write_nbt(&mut nbt);
        nbt
    }
}
