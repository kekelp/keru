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
use crate::thread_local;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ChangeState {
    Changed,
    Unchanged,
    SawChangeAtFrame(u64)
}

pub struct Observer<T> {
    value: T,
    change_state: ChangeState,
}

impl<T> Observer<T> {
    pub fn new(value: T) -> Self {
        Observer { 
            value, 
            change_state: ChangeState::Changed, 
        }
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
        self.change_state = ChangeState::Changed;
        &mut self.value
    }
}

impl<T> Observer<T> {
    pub fn changed(&mut self) -> bool {
        let current_frame = thread_local::current_frame();
        self.changed_inner(current_frame)
    }

    pub fn changed_inner(&mut self, current_frame: u64) -> bool {
        let changed = self.change_state == ChangeState::Changed || self.change_state == ChangeState::SawChangeAtFrame(current_frame);

        if changed {
            self.change_state = ChangeState::SawChangeAtFrame(current_frame);
        } else {
            self.change_state = ChangeState::Unchanged;
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observer() {
        let mut value = Observer::new(17);
        let mut current_frame = 0;
        
        assert!(value.changed_inner(current_frame));
        assert!(value.changed_inner(current_frame));
        assert!(value.changed_inner(current_frame));
        
        current_frame += 1;
        
        assert!(value.changed_inner(current_frame) == false);

        current_frame += 1;
        
        value += 123;

        assert!(value.changed_inner(current_frame));
        assert!(value.changed_inner(current_frame));
        
        current_frame += 1;
        
        assert!(value.changed_inner(current_frame) == false);

        current_frame += 1;

        assert!(value.changed_inner(current_frame) == false);
        assert!(value.changed_inner(current_frame) == false);
        assert!(value.changed_inner(current_frame) == false);
    }
}

// traits

macro_rules! impl_binary_ops {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident) => {
        impl<T: $trait<T>> $trait<T> for Observer<T> {
            type Output = Observer<T::Output>;
            fn $method(self, rhs: T) -> Self::Output {
                Observer::new(self.value.$method(rhs))
            }
        }

        impl<T: $assign_trait<T>> $assign_trait<T> for Observer<T> {
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

impl<T: Neg> Neg for Observer<T> {
    type Output = Observer<T::Output>;
    fn neg(self) -> Self::Output {
        Observer { value: -self.value, change_state: ChangeState::Changed }
    }
}

impl<T: Not> Not for Observer<T> {
    type Output = Observer<T::Output>;
    fn not(self) -> Self::Output {
        Observer { value: !self.value, change_state: ChangeState::Changed }
    }
}

impl<T: Clone> Clone for Observer<T> {
    fn clone(&self) -> Self {
        Observer {
            value: self.value.clone(),
            change_state: self.change_state,
        }
    }
}

impl<T: Copy> Copy for Observer<T> {}

impl<T: Default> Default for Observer<T> {
    fn default() -> Self {
        Observer {
            value: T::default(),
            change_state: ChangeState::Changed,
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Ord> Ord for Observer<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T: Hash> Hash for Observer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}
