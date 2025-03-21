use std::clone::Clone;
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};
use std::default::Default;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref,
    DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr,
    ShrAssign, Sub, SubAssign,
};
use std::sync::atomic::{AtomicU64, Ordering};
use crate::*;

pub(crate) static FAKE_TIME: AtomicU64 = AtomicU64::new(10);

/// Returns the current value of the "timestamp" that the [`Ui`] uses to determine if values in an [`Observer`] have changed.
/// 
/// Might be useful to implement custom versions of [`Observer`].
pub(crate) fn observer_timestamp() -> u64 {
    // todo: is relaxed right here?
    return FAKE_TIME.fetch_add(1, Ordering::Relaxed);
}

/// A wrapper that keeps track of changes to a value.
///
/// `Observer<T>` marks itself as changed when modified. A [`Ui`] can check for changes using
/// [`Ui::check_changes()`].
///
/// # Limitations
/// 
/// - This struct cannot keep track of changes made through interior mutability or unsafe code.
///
/// # Example
///
/// See the "reactive" example in the repository.
pub struct Observer<T> {
    pub(crate) value: T,
    pub changed_at: u64,
}

impl<T> Observer<T> {
    // todo: also impl Into<Observer<T>> for T?
    pub fn new(value: T) -> Self {
        Observer {
            value,
            changed_at: observer_timestamp(),
        }
    }

    // maybe you want to pass a const into something that uses an observer? In that case, the time should stay at zero, so it's never changed
    pub const fn new_const(value: T) -> Self {
        Observer {
            value,
            changed_at: 0,
        }
    }

    pub(crate) fn changed_at(&self) -> Changed {
        Changed::ChangedAt(self.changed_at)
    }
}

impl<T> Deref for Observer<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Observer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.changed_at = observer_timestamp();
        &mut self.value
    }
}

use crate::Ui;
impl Ui {
    /// Returns `true` if the value wrapped by `observer` was changed in the last frame.
    /// 
    /// # Limitations
    /// 
    /// - The `Observer` struct can't keep track of changes made through interior mutability or unsafe code.
    ///
    /// # Example
    /// See the "reactive_block" example in the repository.
    pub fn check_changes<T>(&self, observer: &mut Observer<T>) -> bool {
        return observer.changed_at > self.sys.third_last_frame_end_fake_time;
    }
}

// Operator traits (autoderef doesn't work on operators)
macro_rules! impl_binary_ops {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident) => {
        impl<T: $trait<T>> $trait<T> for Observer<T> {
            type Output = Observer<T::Output>;
            fn $method(self, rhs: T) -> Self::Output {
                let value = self.value.$method(rhs);
                Observer {
                    value,
                    changed_at: observer_timestamp(),
                }
            }
        }

        impl<T: $assign_trait<T>> $assign_trait<T> for Observer<T> {
            fn $assign_method(&mut self, rhs: T) {
                self.changed_at = observer_timestamp();
                self.value.$assign_method(rhs);
            }
        }
    };
}

impl_binary_ops!(Add, add, AddAssign, add_assign);
impl_binary_ops!(Sub, sub, SubAssign, sub_assign);
impl_binary_ops!(Mul, mul, MulAssign, mul_assign);
impl_binary_ops!(Div, div, DivAssign, div_assign);
impl_binary_ops!(Rem, rem, RemAssign, rem_assign);
impl_binary_ops!(BitAnd, bitand, BitAndAssign, bitand_assign);
impl_binary_ops!(BitOr, bitor, BitOrAssign, bitor_assign);
impl_binary_ops!(BitXor, bitxor, BitXorAssign, bitxor_assign);
impl_binary_ops!(Shl, shl, ShlAssign, shl_assign);
impl_binary_ops!(Shr, shr, ShrAssign, shr_assign);

impl<T: Neg> Neg for Observer<T> {
    type Output = Observer<T::Output>;
    fn neg(self) -> Self::Output {
        Observer {
            value: -self.value,
            changed_at: observer_timestamp(),
        }
    }
}

impl<T: Not> Not for Observer<T> {
    type Output = Observer<T::Output>;
    fn not(self) -> Self::Output {
        Observer {
            value: !self.value,
            changed_at: observer_timestamp(),

        }
    }
}

impl<T: Clone> Clone for Observer<T> {
    fn clone(&self) -> Self {
        Observer {
            value: self.value.clone(),
            changed_at: observer_timestamp(),
        }
    }
}

impl<T: Copy> Copy for Observer<T> {}

impl<T: Default> Default for Observer<T> {
    fn default() -> Self {
        Observer {
            value: T::default(),
            changed_at: observer_timestamp(),
        }
    }
}

impl<T: Debug> Debug for Observer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: Display> Display for Observer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: PartialEq> PartialEq for Observer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T: Eq> Eq for Observer<T> {}

impl<T: PartialOrd> PartialOrd for Observer<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Ord> Ord for Observer<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T: Hash> Hash for Observer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}
