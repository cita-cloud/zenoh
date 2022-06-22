//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};
use zenoh_core::Result as ZResult;
pub use zenoh_protocol_core::key_expr::*;

use crate::Session;

#[derive(Clone)]
pub(crate) enum KeyExprInner<'a> {
    Borrowed(&'a keyexpr),
    BorrowedWire {
        key_expr: &'a keyexpr,
        expr_id: u64,
        prefix_len: u32,
        session_id: u16,
    },
    Owned(OwnedKeyExpr),
    Wire {
        key_expr: OwnedKeyExpr,
        expr_id: u64,
        prefix_len: u32,
        session_id: u16,
    },
}

/// A possibly-owned, possibly pre-optimized version of [`keyexpr`].
/// Check [`keyexpr`]'s documentation for detailed explainations.
#[repr(transparent)]
#[derive(Clone)]
pub struct KeyExpr<'a>(pub(crate) KeyExprInner<'a>);
impl std::ops::Deref for KeyExpr<'_> {
    type Target = keyexpr;
    fn deref(&self) -> &Self::Target {
        match &self.0 {
            KeyExprInner::Borrowed(s) => *s,
            KeyExprInner::Owned(s) => s,
            KeyExprInner::Wire { key_expr, .. } => key_expr,
            KeyExprInner::BorrowedWire { key_expr, .. } => key_expr,
        }
    }
}
impl FromStr for KeyExpr<'static> {
    type Err = zenoh_core::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(KeyExprInner::Owned(s.parse()?)))
    }
}
impl<'a> From<super::KeyExpr<'a>> for OwnedKeyExpr {
    fn from(val: super::KeyExpr<'a>) -> Self {
        match val.0 {
            KeyExprInner::Borrowed(key_expr) | KeyExprInner::BorrowedWire { key_expr, .. } => {
                key_expr.into()
            }
            KeyExprInner::Owned(key_expr) | KeyExprInner::Wire { key_expr, .. } => key_expr,
        }
    }
}
impl AsRef<keyexpr> for KeyExpr<'_> {
    fn as_ref(&self) -> &keyexpr {
        self
    }
}
impl<'a> From<&'a keyexpr> for KeyExpr<'a> {
    fn from(ke: &'a keyexpr) -> Self {
        Self(KeyExprInner::Borrowed(ke))
    }
}
impl From<OwnedKeyExpr> for KeyExpr<'_> {
    fn from(v: OwnedKeyExpr) -> Self {
        Self(KeyExprInner::Owned(v))
    }
}
impl<'a> From<&'a OwnedKeyExpr> for KeyExpr<'a> {
    fn from(v: &'a OwnedKeyExpr) -> Self {
        Self(KeyExprInner::Borrowed(&*v))
    }
}
impl<'a> From<&'a KeyExpr<'a>> for KeyExpr<'a> {
    fn from(val: &'a KeyExpr<'a>) -> Self {
        Self::from(val.as_keyexpr())
    }
}
impl TryFrom<String> for KeyExpr<'static> {
    type Error = zenoh_core::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(KeyExprInner::Owned(value.try_into()?)))
    }
}
impl<'a> TryFrom<&'a String> for KeyExpr<'a> {
    type Error = zenoh_core::Error;
    fn try_from(value: &'a String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
impl<'a> TryFrom<&'a mut String> for KeyExpr<'a> {
    type Error = zenoh_core::Error;
    fn try_from(value: &'a mut String) -> Result<Self, Self::Error> {
        Ok(Self::from(keyexpr::new(value)?))
    }
}
impl<'a> TryFrom<&'a str> for KeyExpr<'a> {
    type Error = zenoh_core::Error;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(KeyExprInner::Borrowed(value.try_into()?)))
    }
}
impl<'a> TryFrom<&'a mut str> for KeyExpr<'a> {
    type Error = zenoh_core::Error;
    fn try_from(value: &'a mut str) -> Result<Self, Self::Error> {
        Ok(Self(KeyExprInner::Borrowed(value.try_into()?)))
    }
}
impl std::fmt::Debug for KeyExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_keyexpr(), f)
    }
}
impl std::fmt::Display for KeyExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.as_keyexpr(), f)
    }
}
impl PartialEq for KeyExpr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_keyexpr() == other.as_keyexpr()
    }
}
impl Eq for KeyExpr<'_> {}
impl std::hash::Hash for KeyExpr<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_keyexpr().hash(state);
    }
}

impl KeyExpr<'static> {
    /// Constructs an [`KeyExpr`] without checking [`keyexpr`]'s invariants
    /// # Safety
    /// Key Expressions must follow some rules to be accepted by a Zenoh network.
    /// Messages addressed with invalid key expressions will be dropped.
    pub unsafe fn from_string_unchecked(s: String) -> Self {
        Self(KeyExprInner::Owned(OwnedKeyExpr::from_string_unchecked(s)))
    }

    /// Constructs an [`KeyExpr`] without checking [`keyexpr`]'s invariants
    /// # Safety
    /// Key Expressions must follow some rules to be accepted by a Zenoh network.
    /// Messages addressed with invalid key expressions will be dropped.
    pub unsafe fn from_boxed_string_unchecked(s: Box<str>) -> Self {
        Self(KeyExprInner::Owned(
            OwnedKeyExpr::from_boxed_string_unchecked(s),
        ))
    }
}
impl<'a> KeyExpr<'a> {
    /// Constructs an [`KeyExpr`] without checking [`keyexpr`]'s invariants
    /// # Safety
    /// Key Expressions must follow some rules to be accepted by a Zenoh network.
    /// Messages addressed with invalid key expressions will be dropped.
    pub unsafe fn from_str_uncheckend(s: &'a str) -> Self {
        keyexpr::from_str_unchecked(s).into()
    }

    /// Returns the borrowed version of `self`
    pub fn as_keyexpr(&self) -> &keyexpr {
        self
    }

    /// Creates a `KeyExpr` that borrows `self`'s internals.
    ///
    /// This is only useful when you need to pass a `KeyExpr<'a>` by value.
    pub fn borrowing_clone(&'a self) -> Self {
        Self(match &self.0 {
            KeyExprInner::Borrowed(key_expr) => KeyExprInner::Borrowed(key_expr),
            KeyExprInner::BorrowedWire {
                key_expr,
                expr_id,
                prefix_len,
                session_id,
            } => KeyExprInner::BorrowedWire {
                key_expr,
                expr_id: *expr_id,
                prefix_len: *prefix_len,
                session_id: *session_id,
            },
            KeyExprInner::Owned(key_expr) => KeyExprInner::Borrowed(key_expr),
            KeyExprInner::Wire {
                key_expr,
                expr_id,
                prefix_len,
                session_id,
            } => KeyExprInner::BorrowedWire {
                key_expr,
                expr_id: *expr_id,
                prefix_len: *prefix_len,
                session_id: *session_id,
            },
        })
    }

    /// Ensure's `self` owns all of its data, and informs rustc that it does.
    pub fn into_owned(self) -> KeyExpr<'static> {
        match self.0 {
            KeyExprInner::Borrowed(s) => KeyExpr(KeyExprInner::Owned(s.into())),
            KeyExprInner::Owned(s) => KeyExpr(KeyExprInner::Owned(s)),
            KeyExprInner::BorrowedWire {
                key_expr,
                expr_id,
                prefix_len,
                session_id,
            } => KeyExpr(KeyExprInner::Wire {
                key_expr: key_expr.into(),
                expr_id,
                prefix_len,
                session_id,
            }),
            KeyExprInner::Wire {
                key_expr,
                expr_id,
                prefix_len,
                session_id,
            } => KeyExpr(KeyExprInner::Wire {
                key_expr,
                expr_id,
                prefix_len,
                session_id,
            }),
        }
    }

    /// Joins both sides, inserting a `/` in between them.
    ///
    /// This should be your prefered method when concatenating path segments.
    ///
    /// This is notably useful for workspaces:
    /// ```rust
    /// # use std::convert::TryFrom;
    /// # use zenoh::prelude::KeyExpr;
    /// # let get_workspace = || KeyExpr::try_from("some/workspace").unwrap();
    /// let workspace: KeyExpr = get_workspace();
    /// let topic = workspace.join("some/topic").unwrap();
    /// ```
    pub fn join<S: AsRef<str> + ?Sized>(&self, s: &S) -> ZResult<KeyExpr<'static>> {
        let r = OwnedKeyExpr::try_from(format!("{}/{}", self, s.as_ref()))?;
        if let KeyExprInner::Wire {
            expr_id,
            prefix_len,
            session_id,
            ..
        } = &self.0
        {
            Ok(KeyExpr(KeyExprInner::Wire {
                key_expr: r,
                expr_id: *expr_id,
                prefix_len: *prefix_len,
                session_id: *session_id,
            }))
        } else {
            Ok(r.into())
        }
    }

    /// Performs string concatenation and returns the result as a [`KeyExpr`] if possible.
    ///
    /// You should probably prefer [`KeyExpr::join`] as Zenoh may then take advantage of the hierachical separation it inserts.
    pub fn concat<S: AsRef<str> + ?Sized>(&self, s: &S) -> ZResult<KeyExpr<'static>> {
        let s = s.as_ref();
        if self.ends_with('*') && s.starts_with('*') {
            bail!("Tried to concatenate {} (ends with *) and {} (starts with *), which would likely have caused bugs. If you're sure you want to do this, concatenate these into a string and then try to convert.", self, s)
        }
        let r = OwnedKeyExpr::try_from(format!("{}{}", self, s))?;
        if let KeyExprInner::Wire {
            expr_id,
            prefix_len,
            session_id,
            ..
        }
        | KeyExprInner::BorrowedWire {
            expr_id,
            prefix_len,
            session_id,
            ..
        } = &self.0
        {
            Ok(KeyExpr(KeyExprInner::Wire {
                key_expr: r,
                expr_id: *expr_id,
                prefix_len: *prefix_len,
                session_id: *session_id,
            }))
        } else {
            Ok(r.into())
        }
    }
}

impl<'a> KeyExpr<'a> {
    pub(crate) fn is_optimized(&self, session: &Session) -> bool {
        matches!(&self.0, KeyExprInner::Wire { expr_id, session_id, .. } | KeyExprInner::BorrowedWire { expr_id, session_id, .. } if *expr_id != 0 && session.id == *session_id)
    }
    pub(crate) fn is_fully_optimized(&self, session: &Session) -> bool {
        matches!(&self.0, KeyExprInner::Wire { expr_id, session_id, .. } | KeyExprInner::BorrowedWire { expr_id, session_id, .. } if *expr_id != 0 && session.id == *session_id)
    }
    pub(crate) fn to_wire(&'a self, session: &crate::Session) -> zenoh_protocol_core::WireExpr<'a> {
        match &self.0 {
            KeyExprInner::Wire {
                key_expr,
                expr_id,
                prefix_len,
                session_id,
            } if session.id == *session_id => zenoh_protocol_core::WireExpr {
                scope: *expr_id as u64,
                suffix: std::borrow::Cow::Borrowed(&key_expr.as_str()[((*prefix_len) as usize)..]),
            },
            KeyExprInner::BorrowedWire {
                key_expr,
                expr_id,
                prefix_len,
                session_id,
            } if session.id == *session_id => zenoh_protocol_core::WireExpr {
                scope: *expr_id as u64,
                suffix: std::borrow::Cow::Borrowed(&key_expr.as_str()[((*prefix_len) as usize)..]),
            },
            KeyExprInner::Owned(key_expr) | KeyExprInner::Wire { key_expr, .. } => {
                zenoh_protocol_core::WireExpr {
                    scope: 0,
                    suffix: std::borrow::Cow::Borrowed(key_expr.as_str()),
                }
            }
            KeyExprInner::Borrowed(key_expr) | KeyExprInner::BorrowedWire { key_expr, .. } => {
                zenoh_protocol_core::WireExpr {
                    scope: 0,
                    suffix: std::borrow::Cow::Borrowed(key_expr.as_str()),
                }
            }
        }
    }
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
#[test]
fn size_of_KeyExpr() {
    assert_eq!(
        std::mem::size_of::<KeyExpr>(),
        4 * std::mem::size_of::<usize>()
    );
    assert_eq!(
        std::mem::size_of::<Option<KeyExpr>>(),
        4 * std::mem::size_of::<usize>()
    );
}
