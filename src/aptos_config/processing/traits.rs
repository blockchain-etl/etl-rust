pub trait Encode<T> {
    fn encode(&self) -> T;
}

pub trait TryEncode<T> {
    type Error;
    fn try_encode(&self) -> Result<T, Self::Error>;
}

/// Adds `from_vec` which allows easily turning a vector of T into Vec
pub trait FromVec<T> {
    /// Consumes the vector and its items, returns a vector of Self
    fn from_vec(vector: Vec<T>) -> Vec<Self>
    where
        Self: Sized + From<T>,
    {
        Vec::from_iter(vector.into_iter().map(|i| Self::from(i)))
    }

    /// Consumes the vector and its items, returns a vector of Self or an Error
    fn try_from_vec(vector: Vec<T>) -> Result<Vec<Self>, <Self as TryFrom<T>>::Error>
    where
        Self: Sized + TryFrom<T>,
    {
        let mut output = Vec::with_capacity(vector.len());
        for item in vector {
            output.push(Self::try_from(item)?);
        }
        Ok(output)
    }
}

/// Adds `from_vecref` which allows easily turning a &vector of T into Vec
pub trait FromVecRef<T> {
    /// Clones items then turns into vector of selves
    fn from_vecref(vector: &[T]) -> Vec<Self>
    where
        Self: Sized + From<T>,
        T: Clone,
    {
        Vec::from_iter(vector.iter().map(|i| i.clone().into()))
    }

    /// Clones items then turns into vector of selves
    fn try_from_vecref(vector: &Vec<T>) -> Result<Vec<Self>, <Self as TryFrom<T>>::Error>
    where
        Self: Sized + TryFrom<T>,
        T: Clone,
    {
        let mut output = Vec::with_capacity(vector.len());
        for item in vector {
            output.push(Self::try_from(item.clone())?);
        }
        Ok(output)
    }
}
