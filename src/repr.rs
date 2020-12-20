use crate::NbtCompound;
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    hash::Hash,
};

/// An error associated with the structure of an NBT tag tree. This error represents a conflict
/// between the expected and actual structure of an NBT tag tree.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NbtStructureError {
    /// The expected type of a tag was not the type encountered.
    TypeMismatch,
    /// An index was out of bounds.
    InvalidIndex,
    /// A tag in a [`NbtCompound`](crate::tag::NbtCompound) was absent.
    MissingTag,
}

impl Display for NbtStructureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for NbtStructureError {}

impl From<NbtReprError<NbtStructureError>> for NbtStructureError {
    fn from(x: NbtReprError<NbtStructureError>) -> Self {
        match x {
            NbtReprError::Structure(e) => e,
            NbtReprError::Conversion(e) => e,
        }
    }
}

/// An error associated with the translation of a NBT representation to a concrete type. This
/// can either be a structure error, meaning an error in the structure of the NBT tree, or a
/// conversion error, meaning an error converting a tag into a concrete type.
///
/// Most of the conversion processes in this crate return a [`NbtStructureError`]
/// when there is a type mismatch. Because of this, the redundant type `NbtReprError<NbtStructureError>`
/// appears fairly often. To remove this redundancy, this type can be converted into a [`NbtStructureError`]
/// via the [`flatten`](crate::NbtReprError::flatten) method or `From`/`Into` conversions.
///
/// [`NbtStructureError`]: crate::repr::NbtStructureError
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NbtReprError<E> {
    /// And error associated with the NBT tree itself. See [`NbtStructureError`](crate::repr::NbtStructureError).
    Structure(NbtStructureError),
    /// A custom error defining an issue during the conversion process.
    Conversion(E),
}

impl<E> NbtReprError<E> {
    /// Creates a [`Conversion`](crate::repr::NbtReprError::Conversion) variant of this error with
    /// the given error.
    pub fn conversion(x: E) -> Self {
        NbtReprError::Conversion(x)
    }
}

impl NbtReprError<NbtStructureError> {
    /// Converts the redundant type [`NbtReprError`]`<`[`NbtStructureError`]`>` into a [`NbtStructureError`].
    ///
    /// [`NbtReprError`]: crate::NbtReprError
    /// [`NbtStructureError`]: crate::NbtStructureError
    pub fn flatten(self) -> NbtStructureError {
        self.into()
    }
}

impl<E: Debug> Display for NbtReprError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<E: Error + 'static> Error for NbtReprError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NbtReprError::Structure(source) => Some(source),
            NbtReprError::Conversion(source) => Some(source),
        }
    }
}

impl<E> From<NbtStructureError> for NbtReprError<E> {
    fn from(x: NbtStructureError) -> Self {
        NbtReprError::Structure(x)
    }
}

/// Defines a type which has a full representation as a [`NbtCompound`].
///
/// Full representation meaning that the type can be constructed from a [`NbtCompound`], and fully serialized
/// as one as well.
///
/// [`NbtCompound`]: crate::tag::NbtCompound
pub trait NbtRepr: Sized {
    /// The error type returned if the [`read_nbt`] function fails.
    ///
    /// [`read_nbt`]: crate::repr::NbtRepr::read_nbt
    type Error;

    /// Updates the data in this type based on the given compound. The intention is that data is copied, not
    /// moved, from the compound to update this type.
    fn read_nbt(&mut self, nbt: &NbtCompound) -> Result<(), Self::Error>;

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
