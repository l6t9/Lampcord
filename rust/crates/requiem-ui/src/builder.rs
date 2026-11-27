#[macro_export]
macro_rules! builder_fields {
    ($ty:ty { $($tail:tt)* }) => {
        $crate::builder_fields!(@fields $ty; $($tail)*);
    };

    (@fields $ty:ty; opt $field:ident : $inner:ty, $($rest:tt)*) => {
        $crate::builder_fields!(@opt $ty; $field; $inner);
        $crate::builder_fields!(@fields $ty; $($rest)*);
    };
    (@fields $ty:ty; opt $field:ident : $inner:ty) => {
        $crate::builder_fields!(@opt $ty; $field; $inner);
    };
    (@fields $ty:ty; $field:ident : $fty:ty, $($rest:tt)*) => {
        $crate::builder_fields!(@plain $ty; $field; $fty);
        $crate::builder_fields!(@fields $ty; $($rest)*);
    };
    (@fields $ty:ty; $field:ident : $fty:ty) => {
        $crate::builder_fields!(@plain $ty; $field; $fty);
    };
    (@fields $ty:ty;) => {};

    (@opt $ty:ty; $field:ident; $inner:ty) => {
        impl $ty {
            pub fn $field(mut self, v: impl Into<$inner>) -> Self {
                self.$field = Some(v.into());
                self
            }
        }
    };
    (@plain $ty:ty; $field:ident; $fty:ty) => {
        impl $ty {
            pub fn $field(mut self, v: impl Into<$fty>) -> Self {
                self.$field = v.into();
                self
            }
        }
    };
}
