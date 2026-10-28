use std::ops;

/// Possible exits from evaluating a node. No exits means evaluation does not terminate.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Flow(u8);

impl Flow {
    pub(super) const NEVER: Self = Self(0);
    pub(super) const NORMAL: Self = Self(1);
    pub(super) const RETURN: Self = Self(2);
    pub(super) const BREAK: Self = Self(4);
    pub(super) const CONTINUE: Self = Self(8);

    pub(super) fn has(self, exit: Self) -> bool {
        self & exit != Self::NEVER
    }

    pub(super) fn union(self, other: Self) -> Self {
        self | other
    }

    pub(super) fn abrupt(self) -> Self {
        self & !Self::NORMAL
    }

    pub(super) fn then(self, next: Self) -> Self {
        if self.has(Self::NORMAL) {
            self.abrupt().union(next)
        } else {
            self
        }
    }
}

impl ops::BitOr for Flow {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl ops::BitAnd for Flow {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl ops::Not for Flow {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
