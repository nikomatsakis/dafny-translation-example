use std::any::Any;
// This test passes in mod.rs but not in this file. Why is that?
#[test]
fn test_downcast_mut() {
    let v: i32 = 42;
    let mut value: Box<dyn Any> = Box::new(v);
    assert!(value.downcast_mut::<i32>().is_some());
    return;
}

// Test module
#[cfg(test)]
mod experimental {
    use std::{any::Any, backtrace, cell::RefCell, fmt::Formatter, mem::{transmute, MaybeUninit}, rc::{Rc, Weak}};
    use num::{BigInt, One, Zero};
    use once_cell::unsync::Lazy;

    use crate::*;

    // A datatype encoded in Rust
    // T can either be an allocated type *const X or a reference type Rc<X>
    // Either way, T must extend Clone and DafnyPrint
    // T must be equatable
    #[derive(PartialEq)]
    enum Tree<T: Clone>
      where T: DafnyPrint
    {
        Leaf,
        Node{left: Rc<Tree<T>>, value: T, right: Rc<Tree<T>>}
    }
    impl <T: Clone> Tree<T>
      where T: DafnyPrint
    {
        #[allow(non_snake_case)]
        fn _isLeaf(&self) -> bool {
            match self {
                Tree::Leaf => true,
                Tree::Node{..} => false
            }
        }
        #[allow(non_snake_case)]
        fn _isNode(&self) -> bool {
            match self {
                Tree::Leaf => false,
                Tree::Node{..} => true
            }
        }
        fn value(&self) -> T {
            match self {
                Tree::Leaf => panic!("Leaf has not value"),
                Tree::Node{value, ..} => value.clone()
            }
        }
        fn left(&self) -> Rc<Tree<T>> {
            match self {
                Tree::Leaf => panic!("Leaf has not left"),
                Tree::Node{left, ..} => Rc::clone(left)
            }
        }
        fn right(&self) -> Rc<Tree<T>> {
            match self {
                Tree::Leaf => panic!("Leaf has not right"),
                Tree::Node{right, ..} => Rc::clone(right)
            }
        }
    }

    trait HasFirst: AsAny
    {
        // Encoding of "var first"
        fn _get_first(&self) -> Rc<String>;
        fn _set_first(&self, new_first: &Rc<String>);
        fn replace_first(&self, new_first: &Rc<String>) -> Rc<String> {
            let old_first = self._get_first();
            self._set_first(new_first);
            old_first
        }
    }
    struct NoStruct {}

    struct MyStructDatatype { // For a class
        first: Rc<String>,
        last: Rc<String>,
    }
    impl DafnyPrint for MyStructDatatype {
        fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
            write!(f, "MyStruct({}, {})",
              self.first,
              self.last)
        }
    }

    //#[derive(PartialEq)]
    struct _MyStructUninit {
        first: MaybeUninit<Rc<String>>,
        last: MaybeUninit<Rc<String>>,
    }
    struct MyStruct { // For a class
        first: Rc<String>,
        last: Rc<String>,
    }
    
    // TODO: create an attribute that, if applied to the definition MyStruct above, would also create the variant definition _MyStructUninit



    impl DafnyPrint for MyStruct {
        fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
            write!(f, "MyStruct({}, {})",
              self.first, self.last)
        }
    }
    
    impl MyStruct {
        fn constructor(first: &Rc<String>, last: &Rc<String>) -> *mut MyStruct {
            let this =
              Box::into_raw(Box::new(_MyStructUninit {
                first: MaybeUninit::uninit(),
                last: MaybeUninit::uninit()}));
            // Two ways to write uninitialized values
            unsafe {(*(this  as *mut _MyStructUninit)).first.as_mut_ptr().write(Rc::clone(first))};
            unsafe {(*(this as *mut _MyStructUninit)).last = transmute(Rc::clone(last))};
            // new;
            this as *mut MyStruct
        }
    }
    impl AsAny for MyStruct {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }
    impl HasFirst for MyStruct {
        // Use unsafe and pointer casting if necessary
        fn _get_first(&self) -> Rc<String> {
            self.first.clone()
        }
        fn _set_first(&self, new_first: &Rc<String>) {
            unsafe {(*(self as *const MyStruct as *mut MyStruct)).first
              = transmute(Rc::clone(new_first))};
        }
    }
    impl MyStruct {
        fn _set_first_mut(&mut self, new_first: &Rc<String>) {
            unsafe {(*(self as *mut MyStruct)).first
              = transmute(Rc::clone(new_first))};
        }
        fn _set_first_static(this: *const MyStruct, new_first: &Rc<String>) {
            unsafe {transmute::<*const MyStruct, &mut MyStruct>(this)}
              ._set_first_mut(new_first)
        }
    }
    #[test]
    fn test_has_first() {
        let doe = Rc::new("Doe".to_string());
        let theobject: *const dyn HasFirst = MyStruct::constructor(
            &Rc::new("John".to_string()),
            &doe);
        let original_first = unsafe { (*theobject)._get_first() };
        assert_eq!(original_first, Rc::new("John".to_string()), "Initial value should be 'John'");
    
        let new_first = Rc::new("Jane".to_string());
        unsafe { (*theobject)._set_first(&new_first) };

        // Test if the pointer theobject points to a struct HasFirst, and then if it's a pointer to a Tree
        // In the first case, do nothing, in the second case, panic
        let is_no_struct = unsafe { &*theobject }.as_any().downcast_ref::<NoStruct>().is_some();
        assert!(!is_no_struct, "The value should not be a NoStruct");
        assert!(!is_instance_of::<dyn HasFirst, NoStruct>(theobject)); // Why is this working??!
        //assert!(!DafnyUpdowncast::<NoStruct>::is_instance_of(&theobject)); // TODO: Clone instead
        
        let is_has_first = unsafe { &*theobject }.as_any().downcast_ref::<MyStruct>().is_some();
        assert!(is_has_first, "The value should be a HasFirst");
        assert!(is_instance_of::<dyn HasFirst, MyStruct>(theobject));
        //assert!(DafnyUpdowncast::<MyStruct>::is_instance_of(&theobject));

        let replaced_value = unsafe { (*theobject).replace_first(&Rc::new("Jack".to_string())) };
        assert_eq!(replaced_value, Rc::new("Jane".to_string()), "Replaced value should be 'Jane'");
        let old_count = Rc::strong_count(&doe);
        //unsafe { drop(Box::from_raw(theobject as *mut dyn HasFirst)) };
        deallocate(theobject);
        assert_eq!(Rc::strong_count(&doe), old_count - 1, "Doe should be deallocated");
    }

    // Function to test allocation and aliasing
    #[test]
    fn test_full_reuse() {
        test_reuse(true);
        test_reuse(false);
    }

    // Function to test allocation and aliasing
    fn test_reuse(reuse: bool) {
        // Create a struct for "John" "Doe" and wrap it with the function allocate()
        let theobject: *mut MyStruct = MyStruct::constructor(
            &Rc::new("John".to_string()),
            &Rc::new("Doe".to_string()));

        // Assign the result to a *const on a raw pointer named "theobject"
        let mut possiblealias: *const MyStruct = theobject;

        // If 'reuse' is true, assign the same pointer to another named "possiblealias"
        // Otherwise, use the method allocate() on a new structure
        if !reuse {
            possiblealias = MyStruct::constructor(
                &Rc::new("John".to_string()),
                &Rc::new("Doe".to_string()));
        }

        // Modify the field "first" to "Jane" in theobject (unsafe code is fine)
        modify!(theobject).first = Rc::new("Jane".to_string());
        //unsafe {(*(theobject as *mut MyStruct)).first = transmute(Rc::new("Jane".to_string()))};
        // Using std::ptr::write:
        //unsafe {std::ptr::write(&mut (*theobject).first, "Jane".to_string())};}h

        // If !reuse and theobject.first == possiblealias.first, panic (unreachable code)
        if !reuse && read!(theobject).first == read!(possiblealias).first {
            panic!("Unreachable code reached!");
        }

        // Deallocate possiblealias
        deallocate(possiblealias);

        // If !reuse, deallocate theobject
        if !reuse {
            deallocate(theobject);
        }
    }

    #[test]
    fn test_tree() {
        let tree: Tree<Rc<MyStructDatatype>> = Tree::Node{
            left: Rc::new(Tree::Leaf),
            value: Rc::new(MyStructDatatype{
                first: Rc::new("Jane".to_string()),
                last: Rc::new("Doe".to_string())}),
            right: Rc::new(Tree::Leaf)
        };
        assert!(tree._isNode());
        assert!(!tree._isLeaf());
        let value = unsafe{std::ptr::read(&(*tree.value()).first)};
        assert_eq!((*value).as_ref(), "Jane".to_string());

        assert!(tree.left().as_ref()._isLeaf());
        assert!(tree.right().as_ref()._isLeaf());

        // Now we test with a *const MyStruct
        let tree: Tree<*const MyStruct> = Tree::Node{
            left: Rc::new(Tree::Leaf),
            value: Box::into_raw(Box::new(MyStruct{
                first: unsafe{transmute(Rc::new("Jane".to_string()))},
                last: unsafe{transmute(Rc::new("Doe".to_string()))}
            })),
            right: Rc::new(Tree::Leaf)};
        
        assert!(tree._isNode());
        assert!(!tree._isLeaf());
        // Use the unsafe read in the previous test
        let value = unsafe{std::ptr::read(&(*tree.value()).first)};
        assert_eq!((*value).as_ref(), "Jane".to_string());
    }

    // Now let's encode a codatatype from Dafny
    // A codatatype is like a datatype but it can expand infinitely
    // For example, an infinite stream of numbers
    //codatatype NumberStream = NumberStream(value: int, tail: NumberStream)
    //{
    //    static function from(i: int): NumberStream {
    //        NumberStream(i, from(i + 1))
    //    }
    //    function get(i: nat): int {
    //        if i == 0 then value else tail.get(i-1)
    //    }
    // }
    struct NumberStream {
        value: Rc<BigInt>,
        // tail is a lazily initialized Rc<NumberStream>
        tail: LazyFieldWrapper<Rc<NumberStream>>
    }
    impl NumberStream {
        fn from(i: &Rc<BigInt>) -> Rc<NumberStream> {
            let i_copy = i.clone(); // Create a cloned BigInt
            Rc::new(NumberStream {
                value: i.clone(),
                tail: LazyFieldWrapper(Lazy::new(::std::boxed::Box::new({
                    move || NumberStream::from(&Rc::new(i_copy.as_ref() + BigInt::one()))})))
                
                /*Lazy::new(
                    Box::new(move || NumberStream::from(&(i_copy + BigInt::one())))
                as Box<dyn FnOnce() -> Rc<NumberStream>>)*/
            })
        }
        pub fn value(&self) -> Rc<BigInt> {
            Rc::clone(&self.value)
        }
        fn tail(&self) -> Rc<NumberStream> {
            Rc::clone(Lazy::force(&self.tail.0))
        }

        fn get(&self, i: &Rc<BigInt>) -> Rc<BigInt> {
            if i.as_ref() == &BigInt::zero() {
                self.value.clone()
            } else {
                self.tail().get(&Rc::new(i.as_ref() - BigInt::one()))
            }
        }
    }

    #[test]
    fn test_numberstream() {
        let stream = NumberStream::from(&Rc::new(BigInt::zero()));
        assert_eq!(*stream.get(&Rc::new(BigInt::zero())), BigInt::zero());
        assert_eq!(*stream.get(&Rc::new(BigInt::one())), BigInt::one());
    }

    struct Wrapper {
        w: Rc<WithConstInitializer>
    }

    trait _WithConstInitializer_consts<T> {
        fn _itself(&self) -> Rc<Wrapper>;
        fn _z(&self) -> i16;
    }

    struct WithConstInitializer {
        x: i16,
        z: RefCell<Option<i16>>,
        itself: RefCell<Option<Weak<Wrapper>>>
    }

    impl WithConstInitializer {
        fn _new(x: i16) -> Rc<WithConstInitializer> {
            let result = Rc::new(WithConstInitializer {
                x: x,
                z: RefCell::new(None),
                itself: RefCell::new(None),
            });
            result.z.replace(Some(if x <= 0 { x } else { x - 1}));
            result
        }
    }
    impl _WithConstInitializer_consts<WithConstInitializer> for Rc<WithConstInitializer> {
        fn _itself(&self) -> Rc<Wrapper> {
            // If itself points to nothing, we compute it
            if self.itself.borrow().as_ref().is_none() ||
              self.itself.borrow().as_ref().unwrap().upgrade().is_none()
            {
              let result = Rc::new(Wrapper{w: Rc::clone(self)});
              self.itself.replace(Some(Rc::downgrade(&result)));
              result
            } else {
              Rc::clone(&self.itself.borrow().as_ref().unwrap().upgrade().unwrap())
            }          
        }
        fn _z(&self) -> i16 {
            self.z.borrow().as_ref().unwrap().clone()
        }
    }

    #[test]
    fn test_const_this_in_datatype() {
        let w: Rc<WithConstInitializer> = WithConstInitializer::_new(2);

        assert_eq!(w.x, 2);
        assert_eq!(w._z(), 1);
        assert_eq!(w._itself().w.x, 2);
    }

    #[test]
    fn test_native_array_pointer() {
        let values: *const Vec<i32> = Box::into_raw(Box::new(vec![1, 2, 3]));
        // allocate another vec of size 100
        let values2: *const Vec<i32> = Box::into_raw(Box::new(vec![0; 100]));

        // Verify that the length of values is 3
        assert_eq!(unsafe{(*values).len()}, 3);
        // If we change the first element to 4, we should read it as being 4 again
        unsafe{*(*(values as *mut Vec<i32>)).get_unchecked_mut(0) = 4};
        assert_eq!(unsafe{*(*values).get_unchecked(0)}, 4);

        deallocate(values);
    }

    fn Test2(x: bool, y: &mut bool, z: &mut bool) {
        *y = !x;
        *z = !x || *y;
    }
    fn Test3() { 
      let mut y = true; let mut z = true;
      Test2(true, &mut y, &mut z);
      let mut i; let mut j;
      i = y;
      j = z;
    }

    trait Func1<T1, O1> {
        fn apply(&self, x: &T1) -> O1;
    }

    struct Closure1 {  y: Rc<BigInt>}
    impl Func1<Rc<BigInt>, bool> for Closure1 {
        fn apply(&self, x: &Rc<BigInt>) -> bool {
            x.eq(&self.y)
        }
    }
    #[test]
    fn test_apply1() {
      let y = Rc::new(BigInt::one());
      let f: Rc<dyn Func1<Rc<BigInt>, bool>> = Rc::new(Closure1{ y });
      assert_eq!(f.apply(&Rc::new(BigInt::zero())), false);
    }

    #[test]
    fn test_apply1Native() {
        let y: Rc<BigInt> = Rc::new(BigInt::one());
        let y_copy = Rc::clone(&y); // Create a cloned BigInt
        let f: Rc<dyn Fn(&Rc<BigInt>) -> bool> = Rc::new(
            move |x: &Rc<BigInt>|y_copy.eq(x));
        assert_eq!(f.as_ref()(&Rc::new(BigInt::zero())), false);
    }

    // Covariance and contravariance for traits
    trait Input {
        fn get_value(&self) -> Rc<BigInt>;
        fn with_initial(&self) -> Option<*const dyn InputWithInitial>;
        /*fn as_input(&self) -> Rc<dyn Input>;
        fn as_input_allocated(&self) -> Rc<*const dyn Input>;*/
    }
    trait InputWithInitial: Input {
        fn initial_value(&self) -> Rc<BigInt>;    
    }
    struct JustInput {
        value: Rc<BigInt>
    }
    // Allocated version
    impl Input for *const JustInput {
        fn get_value(&self) -> Rc<BigInt> {
            unsafe { Rc::clone(&(**self).value) }
        }
        fn with_initial(&self) -> Option<*const dyn InputWithInitial> {
            None
        }
    }
    // Datatype version
    impl Input for Rc<JustInput> {
        fn get_value(&self) -> Rc<BigInt> {
            Rc::clone(&self.value)
        }
        fn with_initial(&self) -> Option<*const dyn InputWithInitial> {
            None
        }
    }
    impl Input for JustInput {
        fn get_value(&self) -> Rc<BigInt> {
            Rc::clone(&self.value)
        }
        fn with_initial(&self) -> Option<*const dyn InputWithInitial> {
            None
        }
    }

    struct JustInputWithInitial {
        value: Rc<BigInt>,
        initial: Rc<BigInt>
    }
    impl InputWithInitial for Rc<JustInputWithInitial> {
        fn initial_value(&self) -> Rc<BigInt> {
            Rc::clone(&self.initial)
        }
    }
    impl InputWithInitial for *const JustInputWithInitial {
        fn initial_value(&self) -> Rc<BigInt> {
            unsafe { Rc::clone(&(**self).initial) }
        }
    }
    impl InputWithInitial for JustInputWithInitial {
        fn initial_value(&self) -> Rc<BigInt> {
            Rc::clone(&self.initial)
        }
    }
    impl Input for Rc<JustInputWithInitial> {
        fn get_value(&self) -> Rc<BigInt> {
            Rc::clone(&self.value)
        }
        fn with_initial(&self) -> Option<*const dyn InputWithInitial> {
            Some(self as *const dyn InputWithInitial)
        }
    }
    impl Input for *const JustInputWithInitial {
        fn get_value(&self) -> Rc<BigInt> {
            unsafe { Rc::clone(&(**self).value) }
        }
        fn with_initial(&self) -> Option<*const dyn InputWithInitial> {
            Some(self)
        }
    }
    impl Input for JustInputWithInitial {
        fn get_value(&self) -> Rc<BigInt> {
            Rc::clone(&self.value)
        }
        fn with_initial(&self) -> Option<*const dyn InputWithInitial> {
            Some(self as *const dyn InputWithInitial)
        }
    }
    
    #[test]
    fn test_allocated_native() {
        let a: Rc<dyn Fn(*const dyn Input) -> *const dyn InputWithInitial> =
          Rc::new(|x: *const dyn Input| {
            // Just return a new value 
            Box::into_raw(Box::new(JustInputWithInitial {
                value: (unsafe{&*x}).get_value(),
                initial: (unsafe{&*x}).get_value()
            }))
          });
        let b: Rc<dyn Fn(*const dyn InputWithInitial) -> *const dyn Input> =
          unsafe { transmute(Rc::clone(&a))};
        // Let's try to pass a regular Input and a regular InputWithInitial to a
        // Let's try to pass a regular InputWithInitial to b
        let just_input = Box::into_raw(Box::new(JustInput { value: Rc::new(BigInt::zero())}));
        let just_input_with_initial = Box::into_raw(Box::new(JustInputWithInitial {
            value: Rc::new(BigInt::zero()),
            initial: Rc::new(BigInt::one())
        }));
        let input: *const dyn Input = just_input;
        let input_with_initial: *const dyn InputWithInitial = just_input_with_initial;
        // The following will be fixed by Rust on February 8, 2024
        // https://github.com/rust-lang/rust/pull/118133
        //let input_with_initial_as_input: *const dyn Input = input_with_initial;
        let result1: *const dyn Input = (*b)(input_with_initial);
        //let result2: *const dyn InputWithInitial = (*a)(input_with_initial_as_input);
        let result3: *const dyn InputWithInitial = (*a)(input);
        assert_eq!(unsafe { &*result1 }.get_value(), (unsafe{&*input}).get_value());
        //assert_eq!(unsafe { &*result2 }.get_value(), (unsafe{&*input}).get_value());
        assert_eq!(unsafe { &*result3 }.get_value(), (unsafe{&*input}).get_value());
    }

    struct Closure2 { }
    impl Func1<*const dyn Input, *const dyn InputWithInitial> for Closure2 {
        fn apply(&self, x: &*const dyn Input) -> *const dyn InputWithInitial {
            Box::into_raw(Box::new(JustInputWithInitial {
                value: unsafe { (**x).get_value() },
                initial: unsafe { (**x).get_value() }
            }))
        }
    }

    #[test]
    fn test_allocated_interpreted() {
        let a: Rc<dyn Func1<*const dyn Input, *const dyn InputWithInitial>> =
          Rc::new(Closure2{});
        let b: Rc<dyn Func1<*const dyn InputWithInitial, *const dyn Input>> =
          unsafe { transmute(Rc::clone(&a))};
        // Let's try to pass a regular Input and a regular InputWithInitial to a
        // Let's try to pass a regular InputWithInitial to b
        let just_input = Box::into_raw(Box::new(JustInput { value: Rc::new(BigInt::zero())}));
        let just_input_with_initial = Box::into_raw(Box::new(JustInputWithInitial {
            value: Rc::new(BigInt::zero()),
            initial: Rc::new(BigInt::one())
        }));
        let input: *const dyn Input = just_input;
        let input_with_initial: *const dyn InputWithInitial = just_input_with_initial;
        // The following will be fixed by Rust on February 8, 2024
        // https://github.com/rust-lang/rust/pull/118133
        //let input_with_initial_as_input: *const dyn Input = input_with_initial;
        let result1: *const dyn Input = (*b).apply(&input_with_initial);
        //let result2: *const dyn InputWithInitial = (*a)(input_with_initial_as_input);
        let result3: *const dyn InputWithInitial = (*a).apply(&input);
        assert_eq!(unsafe { &*result1 }.get_value(), (unsafe{&*input}).get_value());
        //assert_eq!(unsafe { &*result2 }.get_value(), (unsafe{&*input}).get_value());
        assert_eq!(unsafe { &*result3 }.get_value(), (unsafe{&*input}).get_value());
    }

    #[test]
    fn test_datatype_native() {
        let a: Rc<dyn Fn(&Rc<dyn Input>) -> Rc<dyn InputWithInitial>> =
          Rc::new(|x: &Rc<dyn Input>| {
            // Just return a new value 
            Rc::new(JustInputWithInitial {
                value: x.get_value(),
                initial: x.get_value()
            })
          });
        let b: Rc<dyn Fn(&Rc<dyn InputWithInitial>) -> Rc<dyn Input>> =
          unsafe { transmute(Rc::clone(&a))};
        let just_input = Rc::new(JustInput { value: Rc::new(BigInt::zero())});
        let just_input_with_initial = Rc::new(JustInputWithInitial {
            value: Rc::new(BigInt::zero()),
            initial: Rc::new(BigInt::one())
        });
        let input: Rc<dyn Input> = just_input;
        let input_with_initial: Rc<dyn InputWithInitial> = just_input_with_initial;
        // The following will be fixed by Rust on February 8, 2024
        // https://github.com/rust-lang/rust/pull/118133
        //let input_with_initial_as_input: Rc<dyn Input> = input_with_initial;
        let result1: Rc<dyn Input> = (*b)(&input_with_initial);
        //let result2: Rc<dyn InputWithInitial> = (*a)(input_with_initial_as_input);
        let result3: Rc<dyn InputWithInitial> = (*a)(&input);
        assert_eq!(result1.get_value(), input.get_value());
        //assert_eq!(result2.get_value(), input.get_value());
        assert_eq!(result3.get_value(), input.get_value());
    }

    fn test_partial_initialization_aux(b: bool) {
        let mut c: Option<Rc<MyStructDatatype>> = None;
        if b {
            c = Some(Rc::new(MyStructDatatype {
                first: Rc::new("John".to_string()),
                last: Rc::new("Doe".to_string()) }));
        }
        if b {
            assert_eq!(c.as_ref().unwrap().first.as_str(), "John");
            assert_eq!(c.as_ref().unwrap().last.as_str(), "Doe");
        }
        
    }

    #[test]
    fn test_partial_initialization() {
        test_partial_initialization_aux(true);
        test_partial_initialization_aux(false);
    }

    type Even = Rc<BigInt>;
    mod _Even {
        use std::rc::Rc;
        use num::{Integer, BigInt};
        use super::Even;

        pub fn halve(this: &Even) -> Rc<BigInt> {
            Rc::new(this.div_floor(
              &crate::BigInt::parse_bytes(b"2", 10).unwrap()
            ))
        }
    }

    fn TestEven(even: &Even) {
        let half= _Even::halve(even);
        assert_eq!(half.as_ref() * crate::BigInt::parse_bytes(b"2", 10).unwrap(),
        even.as_ref().clone());
    }

    #[test]
    fn test_even() {
        let even = Even::new(crate::BigInt::parse_bytes(b"8", 10).unwrap());
        TestEven(&even);
    }

    #[test]
    fn test_newtypes() {
        let five_bigint = Rc::new(crate::BigInt::parse_bytes(b"5", 10).unwrap());
        // Convert five_bigint to u16
        let five_u16: u16 = num::ToPrimitive::to_u16(five_bigint.as_ref()).unwrap();
    }

    /*pub trait object: AsAny {
        fn as_object_mut(self: &mut Self) -> *mut dyn object;
        //fn is_<T>(self: Box<Self>) -> bool;
    }*/
    pub trait Trait: AsAny + Any {
       // fn as_trait(self: &mut Self) -> *mut dyn Trait;
       //fn from_object(this: *mut dyn object) -> *mut dyn Trait;
       fn current_class(&self) -> Rc<str>;

       fn get_number(&self) -> i32;
    }

    struct Class {}

    impl Trait for Class {
        /*fn as_trait(self: &mut Self) -> *mut dyn Trait {
            self as *mut dyn Trait
        }*/
        fn current_class(&self) -> Rc<str> {
            Rc::from("Class")
        }
        fn get_number(&self) -> i32 {
            0
        }
    }

    /*impl object for Class {
        fn as_object_mut(self: &mut Self) -> *mut dyn object {
           self
        }
    }*/
    impl AsAny for Class {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    
    struct Class2 {
        i: i32
    }

    impl Trait for Class2 {
        fn current_class(&self) -> Rc<str> {
            Rc::from("Class2")
        }
        fn get_number(&self) -> i32 {
            self.i
        }
    }

    impl AsAny for Class2 {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    impl Trait for *mut Class2 {
        fn current_class(&self) -> Rc<str> {
            read!(*self).current_class()
        }
        fn get_number(&self) -> i32 {
            read!(*self).get_number()
        }
    }

    impl AsAny for *mut Class2 {
        fn as_any(&self) -> &dyn Any {
            read!(*self).as_any()
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            modify!(*self).as_any_mut()
        }
    }

    /*impl object for Class2 {
        fn as_object_mut(self: &mut Self) -> *mut dyn object {
           self
        }
    }*/

/*
    struct Class2 {}
    impl Trait for Class2 {
        fn as_trait(self: &mut Self) -> *mut dyn Trait {
            self as *mut dyn Trait
        }
    }
    impl object for Class2 {
        fn as_object_mut(self: &mut Self) -> *mut dyn object {
            self as *mut dyn object
        }
    }
    impl AsAny for Class2 {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }
 */
    #[derive(Clone)]
    enum DafnyOption<T: Clone> {
        Some{value: T},
        None
    }

    //UpcastTo!{Class, dyn Trait} // Mare sure coercions are only from classes to dynamic objects

    impl <T: Clone> DafnyOption<T> {
        pub fn coerce<U: Clone>(f: Rc<impl Fn(T) -> U>) -> Rc<impl Fn(DafnyOption<T>) -> DafnyOption<U>> {
            Rc::new(move |x: DafnyOption<T>| {
                let f2 = f.clone();
                match x {
                    DafnyOption::Some{value} =>
                        DafnyOption::Some{value: f2.as_ref()(value)},
                    DafnyOption::None =>
                        DafnyOption::None
                }
            })
        }
    }

    #[test]
    fn test_downcast_mut() {
        let v: i32 = 42;
        let mut value: Box<dyn Any> = Box::new(v);
        assert!(value.downcast_mut::<i32>().is_some());
        return;
    }

    #[test]
    fn test_covariance() {
        let mut c: *mut Class = Box::into_raw(Box::new(Class{}));
        let mut o: *mut dyn Any = c.upcast_to();
        let mut c_t: *mut dyn Trait = c.upcast_to();
        let mut c_t_o: *mut dyn Any = c_t.upcast_to();
        // Test if c_t_o is of type Class
        assert!(is!(c_t_o, Class));
        assert!(!is!(c_t_o, Class2));
        let mut c_o: *mut dyn Any = c;
        //let mut c_o_t: *mut dyn Trait = c_o.upcast_to();
        //let mut c_o_t: *mut dyn Trait = c_o;
        let mut c_o_c: *mut Class = cast!(c_o, Class);
        let mut c_t_c: *mut Class = cast!(c_t, Class);
        
        // Now let's put a c into an Option<*mut class> and upcast it to an Option<*mut dyn object>
        // and let's do the reverse transformation
        let mut opt_c = Rc::new(DafnyOption::Some{value: c});
        let mut opt_c_o: Rc<DafnyOption<*mut dyn Any>> =
          rc_coerce(DafnyOption::<*mut Class>::coerce(upcast::<*mut Class, *mut dyn Any>())).as_ref()(opt_c);
          //Rc::new(DafnyOption::<*mut dyn object>::Some{value: c});
          //<Rc<DafnyOption<*mut Class>> as CastableTo<Rc<DafnyOption<*mut dyn object>>>>::cast_to(&mut opt_c);
        //let mut opt_c_o_c: Rc<DafnyOption<*mut Class>> = opt_c_o.cast_to();
        //let mut opt_c_t: Rc<DafnyOption<*mut dyn Trait>> = opt_c.cast_to();
        //let mut opt_c_t_c: Rc<DafnyOption<*mut Class>> = opt_c_t.cast_to();
    }

    
    #[test]
    fn test_covariance_object() {
        let mut c: Object<Class> = object::new(Class{});
        assert_eq!(refcount!(c.clone()), 2);
        println!("From class to Any\n");
        let mut o: Object<dyn Any> = c.clone().upcast_to();
        assert_eq!(refcount!(c.clone()), 3);
        println!("From class to Trait\n");
        let mut c_t: Object<dyn Trait> = c.clone().upcast_to();
        assert_eq!(refcount!(c.clone()), 4);
        println!("From Trait to Any\n");
        let x: Rc<dyn Trait> = unsafe { rcmut::to_rc(c_t.clone().0.unwrap()) };
        //let y: Rc<dyn Any> = x as Rc<dyn Any>; // Does not work without UpcastTo
        let mut c_t_o: Object<dyn Any> = c_t.clone().upcast_to();
        assert_eq!(refcount!(c.clone()), 6);

        // Test if c_t_o is of type Class
        assert!(object::is::<Class>(c_t_o.clone()));
        assert!(!object::is::<Class2>(c_t_o.clone()));
        let mut c_o = o.clone();
        //let mut c_o_t: *mut dyn Trait = c_o.upcast_to();
        //let mut c_o_t: *mut dyn Trait = c_o;
        let mut c_o_c: Object<Class> = object::downcast(c_o.clone());
        let mut c_t_c: Object<Class> = object::downcast(c_t.clone().upcast_to());
        
        // Now let's put a c into an Option<*mut class> and upcast it to an Option<*mut dyn object>
        // and let's do the reverse transformation
        let mut opt_c = Rc::new(DafnyOption::Some{value: c});
        let mut opt_c_o: Rc<DafnyOption<Object<dyn Any>>> =
          rc_coerce(DafnyOption::<Object<Class>>::coerce(upcast::<Object<Class>, Object<dyn Any>>())).as_ref()(opt_c);
          //Rc::new(DafnyOption::<*mut dyn object>::Some{value: c});
          //<Rc<DafnyOption<*mut Class>> as CastableTo<Rc<DafnyOption<*mut dyn object>>>>::cast_to(&mut opt_c);
        //let mut opt_c_o_c: Rc<DafnyOption<*mut Class>> = opt_c_o.cast_to();
        //let mut opt_c_t: Rc<DafnyOption<*mut dyn Trait>> = opt_c.cast_to();
        //let mut opt_c_t_c: Rc<DafnyOption<*mut Class>> = opt_c_t.cast_to();
    }

    enum Test {
        Int{i: i32},
        String{s: String}
    }
    impl Test {
        fn _update_string(ss: &mut Test, t: &String) {
            match ss {
                Test::String{s} => *s = t.clone(),
                Test::Int{i} => panic!("Case should not be reachable"),
            }
        }
    }

    fn UpdateTest(t: &mut Test) {
        match t {
            Test::Int{i} => *i = 42,
            Test::String{s} => *s = "Hello".to_string()
        }
    }

    trait DafnyCloneTo<T> {
        fn _clone_to(&self) -> T;
    }

    trait ValueTrait<R> {
        fn addv(self, i: i32) -> R;
        fn _clone_box(&self) -> Box<dyn ValueTrait<R>>;
    }
    impl <T> DafnyPrint for Box<dyn ValueTrait<T>> {
        fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
            write!(f, "ValueTrait<T>")
        }
    }
    impl <T> Debug for Box<dyn ValueTrait<T>> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "ValueTrait<T>")
        }
    }
    impl <T> Clone for Box<dyn ValueTrait<T>> {
        fn clone(&self) -> Self {
           self._clone_box()
        }
    }

    impl ValueTrait<i128> for i32 {
        fn addv(self, i: i32) -> i128 {
            self as i128 + i as i128
        }
        fn _clone_box(&self) -> Box<dyn ValueTrait<i128>> {
            Box::new(self.clone())
        }
    }
    impl ValueTrait<i128> for i128 {
        fn addv(self, i: i32) -> i128 {
            self + i as i128
        }
        fn _clone_box(&self) -> Box<dyn ValueTrait<i128>> {
            Box::new(self.clone())
        }
    }
    #[derive(Debug, Clone)]
    struct Datatype { i: i32}
    impl DafnyPrint for Datatype {
        fn fmt_print(&self, f: &mut Formatter<'_>, in_seq: bool) -> std::fmt::Result {
            write!(f, "Datatype{{i: {}}}", self.i)
        }
    }
    impl ValueTrait<i128> for Datatype {
        fn addv(self, i: i32) -> i128 {
            self.i as i128 + i as i128
        }
        fn _clone_box(&self) -> Box<dyn ValueTrait<i128>> {
            Box::new(self.clone())
        }
    }
    impl ValueTrait<i128> for Rc<Datatype> {
        fn addv(self, i: i32) -> i128 {
            self.i as i128 + i as i128
        }
        fn _clone_box(&self) -> Box<dyn ValueTrait<i128>> {
            Box::new(self.clone())
        }
    }
    #[derive(Debug, Clone)]
    struct ClassType {i: i32}
    impl ValueTrait<i128> for *mut ClassType {
        fn addv(self, i: i32) -> i128 {
            read!(self).i as i128 + i as i128
        }
        fn _clone_box(&self) -> Box<dyn ValueTrait<i128>> {
            Box::new(self.clone())
        }
    }
    #[test]
    fn many_values_trait() {
        let i: i32 = 42;
        let i128: i128 = 42;
        let datatype = Rc::new(Datatype{i: 42});
        let classtype = Box::into_raw(Box::new(ClassType{i: 42}));
        assert_eq!(i.addv(1), 43);
        assert_eq!(i128.addv(1), 43);
        assert_eq!(datatype.clone().addv(1), 43);
        assert_eq!(classtype.addv(1), 43);

        let _all = vec![
            Box::new(i) as Box<dyn ValueTrait<i128>>,
            Box::new(i128) as Box<dyn ValueTrait<i128>>,
            Box::new(datatype.clone()) as Box<dyn ValueTrait<i128>>,
            Box::new(classtype) as Box<dyn ValueTrait<i128>>,
        ];
        let _ = _all[0].clone(); // Should not loop;
        let _ = _all[1].clone(); // Should not loop;
        let _ = _all[2].clone(); // Should not loop;
        let _ = _all[3].clone(); // Should not loop;
        let _all = vec![
            Box::new(i) as Box<dyn ValueTrait<i128>>,
            Box::new(i128) as Box<dyn ValueTrait<i128>>,
            Box::new(datatype.clone()) as Box<dyn ValueTrait<i128>>,
            Box::new(classtype) as Box<dyn ValueTrait<i128>>,
        ];
    }

    // Now the same but with &self instead of self.
    trait AddressTrait<R> {
        fn adda(&self, i: i32) -> R;
        fn _clone_box(&self) -> Box<dyn AddressTrait<R>>;
    }
    impl <T> DafnyPrint for Box<dyn AddressTrait<T>> {
        fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
            write!(f, "AddressTraitDafny<T>")
        }
    }
    impl <T> Clone for Box<dyn AddressTrait<T>> {
        fn clone(&self) -> Self {
            self._clone_box()
        }
    }
    impl <T> Debug for Box<dyn AddressTrait<T>> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "AddressTraitDafny<T>")
        }
    }
    
    impl AddressTrait<i128> for i32 {
        fn adda(&self, i: i32) -> i128 {
            *self as i128 + i as i128
        }
        fn _clone_box(&self) -> Box<dyn AddressTrait<i128>> {
            Box::new(*self)
        }
    }
    impl AddressTrait<i128> for i128 {
        fn adda(&self, i: i32) -> i128 {
            self + i as i128
        }
        fn _clone_box(&self) -> Box<dyn AddressTrait<i128>> {
            Box::new(*self)
        }
    }
    impl AddressTrait<i128> for Datatype {
        fn adda(&self, i: i32) -> i128 {
            self.i as i128 + i as i128
        }
        fn _clone_box(&self) -> Box<dyn AddressTrait<i128>> {
            Box::new(self.clone())
        }
    }
    impl AddressTrait<i128> for Rc<Datatype> {
        fn adda(&self, i: i32) -> i128 {
            self.i as i128 + i as i128
        }
        fn _clone_box(&self) -> Box<dyn AddressTrait<i128>> {
            Box::new(self.clone())
        }
    }
    impl DafnyPrint for ClassType {
        fn fmt_print(&self, f: &mut Formatter<'_>, in_seq: bool) -> std::fmt::Result {
            write!(f, "ClassType{{i: {}}}", self.i)
        }
    }
    impl AddressTrait<i128> for ClassType {
        fn adda(&self, i: i32) -> i128 {
            self.i as i128 + i as i128
        }
        fn _clone_box(&self) -> Box<dyn AddressTrait<i128>> {
            Box::new(self.clone())
        }
    }
    impl AddressTrait<i128> for *mut ClassType {
        fn adda(&self, i: i32) -> i128 {
            read!(*self).adda(i)
        }
        fn _clone_box(&self) -> Box<dyn AddressTrait<i128>> {
            Box::new(*self)
        }
    }
    #[test]
    fn many_values_address() {
        let i: i32 = 0;
        let i128: i128 = 0;
        let datatype = Rc::new(Datatype{i: 0});
        let classtype = Box::into_raw(Box::new(ClassType{i: 0}));
        assert_eq!(i.adda(1), 1);
        assert_eq!(i128.adda(1), 1);
        assert_eq!(datatype.adda(1), 1);
        assert_eq!(classtype.adda(1), 1);
        let _all = vec![
            Box::new(i) as Box<dyn AddressTrait<i128>>,
            Box::new(i128) as Box<dyn AddressTrait<i128>>,
            Box::new(datatype.clone()) as Box<dyn AddressTrait<i128>>,
            Box::new(classtype) as Box<dyn AddressTrait<i128>>,
        ];
        let _ = _all[0].clone(); // Should not loop;
        let _ = _all[1].clone(); // Should not loop;
        let _ = _all[2].clone(); // Should not loop;
        let _ = _all[3].clone(); // Should not loop;
        let _all2 = vec![
            Box::new(i) as Box<dyn AddressTrait<i128>>,
            Box::new(i128) as Box<dyn AddressTrait<i128>>,
            Box::new(datatype.clone()) as Box<dyn AddressTrait<i128>>,
            Box::new(classtype) as Box<dyn AddressTrait<i128>>,
        ];
    }
    #[test]
    fn test_lifetimes() {
        let a = &int!(1);
        let b = &a.clone();
    }

    use std::cell::UnsafeCell;
    use std::mem;

    /// A reference counted smart pointer with unrestricted mutability.
    pub struct RcMut<T> {
        inner: Rc<UnsafeCell<T>>
    }

    impl<T> Clone for RcMut<T> {
        fn clone(&self) -> RcMut<T> {
            RcMut { inner: self.inner.clone() }
        }
    }

    impl<T> RcMut<T> {
        /// Create a new RcMut for a value.
        pub fn new(val: T) -> RcMut<T> {
            RcMut {
                inner: Rc::new(UnsafeCell::new(val))
            }
        }

        /// Retrieve the inner Rc as a reference.
        pub unsafe fn as_rc(&self) -> &Rc<T> {
            mem::transmute(&self.inner)
        }

        /// Retrieve the inner Rc as a mutable reference.
        pub unsafe fn as_rc_mut(&mut self) -> &mut Rc<T> {
            mem::transmute(&mut self.inner)
        }

        /// Get a reference to the value.
        pub unsafe fn borrow(&self) -> &T {
            mem::transmute(self.inner.get())
        }

        /// Get a mutable reference to the value.
        pub unsafe fn borrow_mut(&mut self) -> &mut T {
            mem::transmute(self.inner.get())
        }
    }

    #[test]
    fn test_rc_modify() {
        let mut a = RcMut::new(MyStructDatatype {
           first: Rc::new("test".to_string()),
           last: Rc::new("two".to_string())
        });
        let replacement = Rc::new("one".to_string());
        unsafe {a.borrow_mut()}.first = replacement.clone();
        assert_eq!(unsafe {a.borrow()}.first, replacement);
    }

    #[test]
    fn test_multidimensional_array() {
        let mut a = Array2::placebos(&int!(3), &int!(4));
        for i in 0..3 {
            for j in 0..4 {
                modify!(a).data[i][j] = MaybeUninit::new(int!(i * j));
            }
        }
        let mut a = Array2::construct(a);
        assert_eq!(read!(a).data[0][0], int!(0));
        assert_eq!(read!(a).data[2][3], int!(6));
        deallocate(a);
    }

    #[test]
    fn test_array_initializer() {
        let a = array::placebos::<DafnyInt>(&int!(3));
        modify!(a)[0] = MaybeUninit::new(int!(5));
        modify!(a)[1] = MaybeUninit::new(int!(4));
        modify!(a)[2] = MaybeUninit::new(int!(3));
        let a: *mut[DafnyInt] = unsafe {std::mem::transmute(a)};
        assert_eq!(read!(a)[0], int!(5));
        assert_eq!(read!(a)[1], int!(4));
        assert_eq!(read!(a)[2], int!(3));
        deallocate(a);
    }

    fn do_panic<T>() -> T {
        panic!();
    }

    #[test]
    fn test_multidimensionalarray_four_initializer() {
        let n1 = int!(1);
        let n2 = int!(1);
        let n3 = int!(1);
        let n4 = int!(1);
        
        let a: *mut [Box<[Box<[Box<[MaybeUninit<DafnyInt>]>]>]>] =
            Box::into_raw(
            array::initialize_box(&n1,
            Rc::new(move |_|
                array::initialize_box(&n2, {
                let n3 = n3.clone();
                let n4 = n4.clone();
                Rc::new(move |_|
                    array::initialize_box(&n3, {
                        let n4 = n4.clone();
                        Rc::new(move |_|
                        array::placebos_box::<DafnyInt>(&n4))
                }))
            }
        ))));
        deallocate(a);
    }

    #[test]
    fn test_multidimensionalarray_initializer() {
        let n1 = int!(3);
        let n2 = int!(2);
        let n3 = int!(2);
        let a: *mut [Box<[Box<[MaybeUninit<DafnyInt>]>]>] = Box::into_raw(
            array::initialize_box(&n1,
            Rc::new(move |_| array::initialize_box(&n2, {
                let n3 = n3.clone();
                Rc::new(move |_| array::placebos_box::<DafnyInt>(&n3))
            }
        ))));
        for i in 0..3 {
            for j in 0..2 {
                for k in 0..2 {
                    modify!(a)[i][j][k] = MaybeUninit::new(int!(i*j + k));
                }
            }
        }
        let a: *mut [Box<[Box<[DafnyInt]>]>]  = unsafe {std::mem::transmute(a)};
        for i in 0..3 {
            for j in 0..2 {
                for k in 0..2 {
                    assert_eq!(read!(a)[i][j][k], int!(i*j + k));
                }
            }
        }
        deallocate(a);
    }
    // Try to pass a mutable reference as a trait.
}
// Struct containing two reference-counted fields
