use std::ops::{
    Deref, DerefMut, Add, Sub, Mul, Div, Rem,
    AddAssign, SubAssign, MulAssign, DivAssign, RemAssign,
    Neg, Not, BitAnd, BitOr, BitXor,
    BitAndAssign, BitOrAssign, BitXorAssign,
    Shl, Shr, ShlAssign, ShrAssign
};
use std::fmt::{self, Debug, Display};
use std::cmp::{PartialEq, Eq, PartialOrd, Ord, Ordering};
use std::hash::{Hash, Hasher};
use std::clone::Clone;
use std::default::Default;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ChangeState {
    Changed,
    Unchanged,
    SawChangeAtFrame(u64)
}

pub struct Observed<T> {
    value: T,
    change_state: ChangeState,
}

impl<T> Observed<T> {
    pub fn new(value: T) -> Self {
        Observed { 
            value, 
            change_state: ChangeState::Changed, 
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
        let changed = self.change_state == ChangeState::Changed || self.change_state == ChangeState::SawChangeAtFrame(current_frame);

        if changed {
            self.change_state = ChangeState::SawChangeAtFrame(current_frame);
        } else {
            self.change_state = ChangeState::Unchanged;
        }
        changed
    }
}

use crate::Ui;
impl Ui {
    pub fn observe_changes<T>(&self, observer: &mut Observed<T>) -> bool {
        let current_frame = self.current_frame();
        observer.observe_changes(current_frame)
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
        Observed { value: -self.value, change_state: ChangeState::Changed }
    }
}

impl<T: Not> Not for Observed<T> {
    type Output = Observed<T::Output>;
    fn not(self) -> Self::Output {
        Observed { value: !self.value, change_state: ChangeState::Changed }
    }
}

impl<T: Clone> Clone for Observed<T> {
    fn clone(&self) -> Self {
        Observed {
            value: self.value.clone(),
            change_state: self.change_state,
        }
    }
}

impl<T: Copy> Copy for Observed<T> {}

impl<T: Default> Default for Observed<T> {
    fn default() -> Self {
        Observed {
            value: T::default(),
            change_state: ChangeState::Changed,
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
