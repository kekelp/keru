use std::clone::Clone;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::default::Default;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref,
    DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr,
    ShrAssign, Sub, SubAssign,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ChangeState {
    Unchanged,
    Changed,
    ChangeSeenAtFrame(u64),
}

/// A wrapper that keeps track of changes to a value.
///
/// `Observed<T>` marks itself as changed when modified. A [`Ui`] can check for changes using
/// [`Ui::observe_changes()`].
///
/// The change state is reset to "unchanged" only at the end of an `Ui` frame. Therefore, different parts of an `Ui` can check the state independently in the same frame, and they will all see the value as "changed".
///
/// # Panics
///
/// An `Observed` value should be only observed by **a single `Ui` instance**.
///
/// In debug mode, `Observed<T>` stores the observing `Ui`'s unique ID and panics if another `Ui`
/// attempts to observe it, even at separate times.
///
/// Since most programs only use a single `Ui` instance, this check is omitted in release mode.
/// This means that in release mode, observing with multiple `Ui`s will result in unchecked incorrect behavior. Don't use multiple `Ui`s!
/// 
/// # Interior Mutability
/// 
/// This type cannot detect changes made through interior mutability or unsafe code.
///
/// # Example
///
/// See the "reactive" example in the repository.
///
pub struct Observed<T> {
    // todo: in the future, require Freeze/ShallowImmutable.
    value: T,
    change_state: ChangeState,
    #[cfg(debug_assertions)]
    observer_id: Option<u64>,
}

impl<T> Observed<T> {
    pub fn new(value: T) -> Self {
        Observed {
            value,
            change_state: ChangeState::Changed,
            observer_id: None,
        }
    }
}

impl<T> Deref for Observed<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Observed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.change_state = ChangeState::Changed;
        &mut self.value
    }
}

impl<T> Observed<T> {
    fn observe_changes(&mut self, current_frame: u64) -> bool {
        match self.change_state {
            ChangeState::Unchanged => false,
            ChangeState::Changed => {
                self.change_state = ChangeState::ChangeSeenAtFrame(current_frame);
                true
            }
            ChangeState::ChangeSeenAtFrame(frame) => {
                if frame == current_frame {
                    true
                } else {
                    self.change_state = ChangeState::Unchanged;
                    false
                }
            }
        }
    }

    #[cfg(debug_assertions)]
    fn update_observer_id(&mut self, new_observer_id: u64) {
        if let Some(existing_id) = self.observer_id {
            if existing_id != new_observer_id {
                panic!("Observed<T> cannot be observed by multiple Ui instances.");
            }
        } else {
            self.observer_id = Some(new_observer_id);
        }
    }
}

use crate::Ui;
impl Ui {
    /// Returns `true` if the value wrapped by `observed` was changed since the last frame.
    ///
    /// # Panics
    ///
    /// A given [`Observed`] value can be observed by a single [`Ui`]. Calling this function from different [`Ui`]s with the same `observer` as argument is incorrect.
    ///
    /// In debug mode, doing so will result in a panic. In release mode, **this check is omitted**, and it will result in unchecked incorrect behavior.
    /// 
    /// # Interior Mutability
    /// 
    /// This function cannot detect changes made through interior mutability or unsafe code.
    ///
    /// # Example
    /// See the "reactive" example in the repository.
    pub fn observe_changes<T>(&self, observer: &mut Observed<T>) -> bool {
        #[cfg(debug_assertions)]
        observer.update_observer_id(self.unique_id());

        observer.observe_changes(self.current_frame())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observer() {
        let mut value = Observed::new(17);
        let mut current_frame = 0;

        assert!(value.observe_changes(current_frame));
        assert!(value.observe_changes(current_frame));
        assert!(value.observe_changes(current_frame));

        current_frame += 1;

        assert!(value.observe_changes(current_frame) == false);

        current_frame += 1;

        value += 123;

        assert!(value.observe_changes(current_frame));
        assert!(value.observe_changes(current_frame));

        current_frame += 1;

        assert!(value.observe_changes(current_frame) == false);

        current_frame += 1;

        assert!(value.observe_changes(current_frame) == false);
        assert!(value.observe_changes(current_frame) == false);
        assert!(value.observe_changes(current_frame) == false);
    }
}

// traits

macro_rules! impl_binary_ops {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident) => {
        impl<T: $trait<T>> $trait<T> for Observed<T> {
            type Output = Observed<T::Output>;
            fn $method(self, rhs: T) -> Self::Output {
                Observed::new(self.value.$method(rhs))
            }
        }

        impl<T: $assign_trait<T>> $assign_trait<T> for Observed<T> {
            fn $assign_method(&mut self, rhs: T) {
                self.value.$assign_method(rhs);
                self.change_state = ChangeState::Changed;
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

impl<T: Neg> Neg for Observed<T> {
    type Output = Observed<T::Output>;
    fn neg(self) -> Self::Output {
        Observed {
            value: -self.value,
            change_state: ChangeState::Changed,
            observer_id: self.observer_id,
        }
    }
}

impl<T: Not> Not for Observed<T> {
    type Output = Observed<T::Output>;
    fn not(self) -> Self::Output {
        Observed {
            value: !self.value,
            change_state: ChangeState::Changed,
            observer_id: self.observer_id,
        }
    }
}

impl<T: Clone> Clone for Observed<T> {
    fn clone(&self) -> Self {
        Observed {
            value: self.value.clone(),
            change_state: self.change_state,
            observer_id: self.observer_id,
        }
    }
}

impl<T: Copy> Copy for Observed<T> {}

impl<T: Default> Default for Observed<T> {
    fn default() -> Self {
        Observed {
            value: T::default(),
            change_state: ChangeState::Changed,
            observer_id: None,
        }
    }
}

impl<T: Debug> Debug for Observed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: Display> Display for Observed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: PartialEq> PartialEq for Observed<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T: Eq> Eq for Observed<T> {}

impl<T: PartialOrd> PartialOrd for Observed<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Ord> Ord for Observed<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T: Hash> Hash for Observed<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}
