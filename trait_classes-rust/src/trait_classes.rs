#![allow(warnings, unconditional_panic)]
#![allow(nonstandard_style)]
pub mod _module {
    pub use ::dafny_runtime::*;
    pub use ::std::any::Any;
    pub use ::std::ops::Fn;
    pub use ::std::rc::Rc;
    pub struct _default {}

    impl _default {
        pub fn _allocate_rcmut() -> Object<Self> {
            allocate_rcmut::<Self>()
        }
        pub fn Main() -> () {
            let mut plato = object::new(NorthernPlatypus {
                age: string_of("3"),
            });
            md!(plato).age = string_of("4");
            let mut p: Object<dyn Platypus> =
                UpcastTo::<Object<dyn Platypus>>::upcast_to(plato.clone());
            let mut eg: Object<dyn EggLayer> =
                UpcastTo::<Object<dyn EggLayer>>::upcast_to(p.clone());
            let mut o: Object<dyn Any> = UpcastTo::<Object<dyn Any>>::upcast_to(eg.clone());
            let mut a: Object<dyn Animal> = UpcastTo::<Object<dyn Animal>>::upcast_to(UpcastTo::<
                Object<dyn Mammal>,
            >::upcast_to(
                UpcastTo::<Object<dyn Platypus>>::upcast_to(plato.clone()),
            ));
            // Not found a way yet to make it work.
            //let mut m2: Object<dyn Mammal> = UpcastTo::<Object<dyn Mammal>>::upcast_to(UpcastTo::<Object<dyn Any>>::upcast_to(o.clone()));
            //let mut m3: Object<dyn Mammal> = UpcastTo::<Object<dyn Mammal>>::upcast_to(UpcastTo::<Object<dyn Any>>::upcast_to(a.clone()));
            let mut np: Object<NorthernPlatypus> =
                object::downcast(UpcastTo::<Object<dyn Any>>::upcast_to(a.clone()));
            let mut f: Rc<dyn Fn() -> Sequence<DafnyChar>> = Animal::BecomeOlder(md!(a));
            print!("{}\n", DafnyPrintWrapper(&(&f)()));
            EggLayer::Lay2Eggs(md!(eg));
            // I wish we could use Object<[i32]> directly but unfortunately [i32] does not extends Any
            let mut x: Object<Array1<i32>> = Object(Some(unsafe {
                rcmut::from_rc(Rc::new(Array1 {
                    data: Box::new([0, 0]),
                }))
            }));
            // I should probably wrap arrays with a newtype so that I can cast them as Any
            let mut xo: Object<dyn Any> = UpcastTo::<Object<dyn Any>>::upcast_to(x.clone());
            {
                let __idx0 = <usize as NumCast>::from(int!(1)).unwrap();
                md!(x).data[__idx0] = 2;
            };
            {
                let __idx0 = <usize as NumCast>::from(int!(0)).unwrap();
                md!(x).data[__idx0] = x.as_ref().data[1];
            };
            return ();
        }
    }

    pub trait Animal: Any {
        fn BecomeOlder(&mut self) -> Rc<dyn Fn() -> Sequence<DafnyChar>>;
    }

    pub trait EggLayer: Any + Animal {
        fn Lay1Egg(&mut self) -> ();
        fn Lay2Eggs(&mut self) -> () {
            EggLayer::Lay1Egg(self);
            EggLayer::Lay1Egg(self);
            return ();
        }
    }

    pub trait Mammal: Any + Animal {
        fn GiveMilk(&mut self) -> ();
    }

    pub trait Platypus: Any + Animal + EggLayer + Mammal {}

    pub struct NorthernPlatypus {
        pub age: Sequence<DafnyChar>,
    }

    impl NorthernPlatypus {
        pub fn _allocate_rcmut() -> Object<Self> {
            allocate_rcmut::<Self>()
        }
    }

    impl Platypus for NorthernPlatypus {}

    impl Animal for NorthernPlatypus {
        fn BecomeOlder(&mut self) -> Rc<dyn Fn() -> Sequence<DafnyChar>> {
            self.age = string_of("1").concat(&self.age);
            let mut f: Rc<dyn Fn() -> Sequence<DafnyChar>> = {
                let mut _this = Object::<Self>::from_ref(&self);
                Rc::new(move || -> Sequence<DafnyChar> {
                    string_of("I am ")
                        .concat(&rd!((&_this).clone()).age)
                        .concat(&string_of(" years old"))
                })
            };
            return f;
        }
    }

    impl EggLayer for NorthernPlatypus {
        fn Lay1Egg(&mut self) -> () {
            print!(
                "{}",
                DafnyPrintWrapper(&string_of("A northern platypus layed one egg\n"))
            );
            return ();
        }
    }

    impl Mammal for NorthernPlatypus {
        fn GiveMilk(&mut self) -> () {
            print!(
                "{}",
                DafnyPrintWrapper(&string_of("A norther platypus gave milk\n"))
            );
            return ();
        }
    }

    #[derive(Clone, PartialEq)]
    #[repr(transparent)]
    pub struct uint8(pub u8);

    impl uint8 {
        pub fn is(_source: u8) -> bool {
            return true;
        }
    }

    impl ::std::default::Default for uint8 {
        fn default() -> Self {
            uint8(::std::default::Default::default())
        }
    }

    impl DafnyPrint for uint8 {
        fn fmt_print(
            &self,
            _formatter: &mut ::std::fmt::Formatter,
            in_seq: bool,
        ) -> ::std::fmt::Result {
            DafnyPrint::fmt_print(&self.0, _formatter, in_seq)
        }
    }
}
fn main() {
    _module::_default::Main();
}
