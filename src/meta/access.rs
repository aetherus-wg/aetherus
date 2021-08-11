//! Access macro.

/// Create access-by-reference methods for the given struct member.
/// You can use this to macro generate both a getter and a setter.
/// To automatically implement a getter, use it like so:
/// ```rust
/// pub struct DocStruct {
///     str_prop: String,   
/// }
/// 
/// impl DocStruct {
///     access!(str_prop: String)   
/// }
/// ```
/// which generates a getter at `DocStruct::str_prop()`.
/// To generate a setter, in addition to a getter, use the macro like so:
/// ```rust
/// pub struct DocStruct {
///     str_prop: String,
/// }
/// 
/// impl DocStruct {
///     access!(str_prop, str_prop_mut: String)
/// }
/// ```
/// which also generates a setter at `DocStruct::str_prop_mut()`. 
#[macro_export]
macro_rules! access {
    ($field:ident: $type:ty) => {
        #[inline]
        #[must_use]
        pub const fn $field(&self) -> &$type {
            &self.$field
        }
    };

    ($field:ident, $setter:ident: $type:ty) => {
        #[inline]
        #[must_use]
        pub const fn $field(&self) -> &$type {
            &self.$field
        }

        #[allow(clippy::mut_mut)]
        #[inline]
        #[must_use]
        pub fn $setter(&mut self) -> &mut $type {
            &mut self.$field
        }
    };
}

#[cfg(test)]
mod tests {
    /// Test implementation structure.
    pub struct Testy {
        /// Mutable parameter.
        a: String,
        /// Immutable parameter.
        b: String,
    }

    impl Testy {
        access!(a: String);
        access!(b, b_mut: String);
    }

    /// A simple test for immutable access with an access-generated getter.
    #[test]
    fn test_access() {
        let testy = Testy {
            a: "one".to_owned(),
            b: "two".to_owned(),
        };

        assert_eq!(testy.a(), &"one");
        assert_eq!(testy.b(), &"two");
    }

    /// A simple test for mutable access with an access-generated setter. 
    #[test]
    fn test_mut_access() {
        let mut testy = Testy {
            a: "one".to_owned(),
            b: "two".to_owned(),
        };

        *testy.b_mut() = "three".to_owned();

        assert_eq!(testy.b(), &"three");
    }
}
