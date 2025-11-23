pub trait Production {
    type Driver;
    type Body;

    fn synthesize(body: Self::Body) -> Self::Driver;
}

#[macro_export]
macro_rules! production {
    ($name:ident, $driver:ident -> $body:ty, |$param:pat_param| $clos:expr) => {
        #[doc = concat!("Production: `", stringify!($driver), " -> ", stringify!($body), "`")]
        pub struct $name;

        impl static_sdd::Production for $name {
            type Driver = $driver;
            type Body = $body;

            fn synthesize($param: Self::Body) -> Self::Driver {
                $clos
            }
        }
    };
}

