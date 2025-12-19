pub trait Production {
    type Head;
    type Body;
    type Ctx;

    fn synthesize(ctx: &mut Self::Ctx, body: Self::Body) -> Self::Head;
}

#[macro_export]
macro_rules! production {
    ($name:ident, $head:ident -> $body:ty, |$ctx:ident, $param:pat_param| $clos:expr) => {
        #[doc = concat!("Production: `", stringify!($head), " -> ", stringify!($body), "`")]
        pub struct $name;

        impl static_sdd::Production for $name {
            type Head = $head;
            type Body = $body;
            type Ctx = __CompilerContext;

            fn synthesize($ctx: &mut Self::Ctx, $param: Self::Body) -> Self::Head {
                $clos
            }
        }
    };
    ($name:ident, $head:ident -> $body:ty, |$param:pat_param| $clos:expr) => {
        #[doc = concat!("Production: `", stringify!($head), " -> ", stringify!($body), "`")]
        pub struct $name;

        impl static_sdd::Production for $name {
            type Head = $head;
            type Body = $body;
            type Ctx = __CompilerContext;

            fn synthesize(_: &mut Self::Ctx, $param: Self::Body) -> Self::Head {
                $clos
            }
        }
    };
    ($name:ident, $head:ident -> $body:ty) => {
        #[doc = concat!("Production: `", stringify!($head), " -> ", stringify!($body), "`")]
        pub struct $name;

        impl static_sdd::Production for $name {
            type Head = $head;
            type Body = $body;
            type Ctx = __CompilerContext;

            fn synthesize(_: &mut Self::Ctx, body: Self::Body) -> Self::Head {
                body.into()
            }
        }
    };
}
