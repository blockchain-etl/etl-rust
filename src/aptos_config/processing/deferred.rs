#[derive(Debug, Clone)]
pub enum DeferredError<T> {
    CurrentlyDeferred,
    AlreadyPresent(T, T),
}

impl<T> std::fmt::Display for DeferredError<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentlyDeferred => write!(f, "Value is currently deferred (not set)"),
            Self::AlreadyPresent(new, old) => write!(
                f,
                "Value is already present({:?}), did you mean to overwrite with {:?}",
                new, old
            ),
        }
    }
}

impl<T> std::error::Error for DeferredError<T> where T: std::fmt::Debug {}

/// An enum to represent a value we may want to
/// set at a later time.  
#[derive(Debug, Clone)]
pub enum Deferred<T> {
    /// Represents a value that has been set, and
    /// is generally expected to not be modified.
    Present(T),
    /// Represents a value that has not been set yet,
    /// and is inticipated to be determined later.
    Deferred,
    /// Represents a value that has not been set yet,
    /// however a fallback value is provided such that
    /// if the value is never set, the fallback value
    /// can still be used.
    DeferredFallback(T),
}

impl<T> std::fmt::Display for Deferred<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Present(inner) => write!(f, "Present({})", inner),
            Self::Deferred => write!(f, "Deferred"),
            Self::DeferredFallback(inner) => write!(f, "DeferredFallback({})", inner),
        }
    }
}

impl<T> Deferred<T> {
    /// Returns true if the value is currently Deferred.  Do not confuse
    /// with `is_usable` which would return True if we are still Deferred
    /// but a fallback is set.
    pub fn is_deferred(&self) -> bool {
        matches!(self, Deferred::Deferred | Deferred::DeferredFallback(_))
    }

    /// Returns true if the value is currently Deferred, but a fallback
    /// is established
    pub fn is_deferred_with_fallback(&self) -> bool {
        matches!(self, Deferred::DeferredFallback(_))
    }

    /// Returns true if a value we explicitly specified as Present.  This
    /// does not return True if a fallback was implemented and we are
    /// still deferred
    pub fn is_present(&self) -> bool {
        matches!(self, Deferred::Present(_))
    }

    /// Returns true if we can extract a value from Deferred.  This includes
    /// if a fallback is set.
    pub fn is_usable(&self) -> bool {
        matches!(self, Deferred::Present(_) | Deferred::DeferredFallback(_))
    }

    /// If not present, returns a new Deferred with a Fallback.
    pub fn set_fallback(self, value: T) -> Self {
        match self {
            Self::Deferred | Self::DeferredFallback(_) => Self::DeferredFallback(value),
            Self::Present(_) => self,
        }
    }

    /// Turns a deferred variant into a Present variant.  Raises an Error if already
    /// present.
    pub fn make_present(self, value: T) -> Result<Self, DeferredError<T>> {
        match self {
            Self::Deferred | Self::DeferredFallback(_) => Ok(Self::Present(value)),
            Self::Present(old) => Err(DeferredError::AlreadyPresent(old, value)),
        }
    }

    /// Turns any variant into a Present version containing the value.  
    pub fn overwrite(self, value: T) -> Self {
        Self::Present(value)
    }

    /// Unwraps the enum and returns T, panics if Deferred
    pub fn unwrap(self) -> T {
        match self {
            Self::Present(value) | Self::DeferredFallback(value) => value,
            Self::Deferred => panic!("Cannot unwrap deferred without FallBack"),
        }
    }

    /// Like unwrap, except will only return if present and will ignore fallback
    pub fn unwrap_present(self) -> T {
        match self {
            Self::Present(value) => value,
            Self::Deferred | Self::DeferredFallback(_) => panic!("Is Deferred, not present"),
        }
    }

    /// Like unwrap, but can provide a value to be returned if Deferred
    pub fn unwrap_or(self, value: T) -> T {
        match self {
            Self::Present(inner) | Self::DeferredFallback(inner) => inner,
            Self::Deferred => value,
        }
    }

    /// Like unwrap_or, except will return the provided value over the fallback.
    pub fn unwrap_present_or(self, value: T) -> T {
        match self {
            Self::Present(inner) => inner,
            Self::Deferred | Self::DeferredFallback(_) => value,
        }
    }

    /// Extracts value, utilizes fallback if set.
    pub fn extract(self) -> Result<T, DeferredError<T>> {
        match self {
            Self::DeferredFallback(value) | Self::Present(value) => Ok(value),
            Self::Deferred => Err(DeferredError::CurrentlyDeferred),
        }
    }

    /// Extracts value, ignores fallback and only returns value if present.
    pub fn extract_present(self) -> Result<T, DeferredError<T>> {
        match self {
            Self::Present(value) => Ok(value),
            _ => Err(DeferredError::CurrentlyDeferred),
        }
    }
}

#[allow(clippy::from_over_into)]
impl<T> Into<Option<T>> for Deferred<T> {
    /// Allows converting into an Option. If `is_usable` is true, you should
    /// get the the value stored as Some(T), otherwise you will receive None.
    /// Note that this will utilize the Fallback.
    fn into(self) -> Option<T> {
        match self {
            Self::Present(inner) => Some(inner),
            Self::DeferredFallback(inner) => Some(inner),
            Self::Deferred => None,
        }
    }
}

impl<T> From<T> for Deferred<T> {
    fn from(value: T) -> Self {
        Self::Present(value)
    }
}
