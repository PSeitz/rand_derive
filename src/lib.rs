// Copyright 2016 bluss
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation of `derive(Rand!)` for `macro_attr!{}`.
//!
//! This crate defines a macro `Rand!{}` that can be used through `macro_attr!{}`
//! to derive an implementation of the `Rand` trait (crate rand version 0.3.x).
//!
//! Using this macro also depends on crates `parse_macros` and `parse_generics_shim`,
//! which must be included in the crate that uses them.
//!
//! ## Example
//!
//! ```
//! extern crate rand;
//!
//! #[macro_use] extern crate parse_macros;
//! #[macro_use] extern crate parse_generics_shim;
//! #[macro_use] extern crate macro_attr;
//!
//! #[macro_use] extern crate rand_derive;
//!
//! macro_attr! {
//!     #[derive(Rand!, Debug)]
//!     enum TestEnum {
//!         A,
//!         B,
//!         C,
//!     }
//! }
//!
//! macro_attr! {
//!     #[derive(Rand!, Debug)]
//!     struct Point<T> {
//!         x: T,
//!         y: T,
//!     }
//! }
//!
//! fn main() {
//!     let t: TestEnum = rand::random();
//!     let p: Point<f32> = rand::random();
//! }
//! ```
//!
//! ## Known Limitations
//!
//! If the struct or enum is too complex, the compiler may run up against
//! the recursion limit when compiling your crate. This can be adjusted
//! with an attribute like `#![recursion_limit="128"]`.
//!
//! * Does not allow explicit discriminants on unitary enum variants
//! * Does not yet allow customizing which type parameters get the `T: Rand`
//!   bound applied. Right now they all get it.
#![cfg_attr(not(test), no_std)]

//#![cfg_attr(test, feature(trace_macros))]
#![recursion_limit="128"]
#[cfg(test)]
#[macro_use] extern crate parse_macros;
#[cfg(test)]
#[macro_use] extern crate parse_generics_shim;
#[cfg(test)]
#[macro_use] extern crate macro_attr;
#[cfg(test)]
extern crate rand;

/// Implementation of `derive(Rand!)` for `macro_attr!{}`.
#[macro_export]
macro_rules! Rand {
    (
        () $($tail:tt)*
    ) => {
        parse_item! {
            then Rand! { @item },
            $($tail)*
        }
    };
    // enum
    (
        @item
        enum {
            attrs: $_attrs:tt,
            vis: $_vis:tt,
            name: $name:ident,
            generics: {
                constr: [$($constr:tt)*],
                params: [$($params:tt)*],
                ltimes: $_ltimes:tt,
                tnames: [$($tnames:ident,)*],
            },
            where: {
                clause: $_clause:tt,
                preds: [$($preds:tt)*],
            },
            variants: [
                $({
                    ord: ($ord:expr, $_ord:tt),
                    attrs: [$($_vattrs:tt)*],
                    kind: $vkind:ident,
                    name: $vname:ident,
                    fields: $vfields:tt,
                    num_fields: $vnum_fields:expr,
                },)+  // + because 0 variants is explicitly unsupported
            ],
            num_variants: $num_variants:expr,
            $($_enum_tail:tt)*
        }
    ) => {
        Rand!{ @as_item
            impl<$($constr)*> ::rand::Rand for $name<$($params)*>
                where (): ::rand::Rand, $($tnames: ::rand::Rand,)* $($preds)*
            {
                fn rand<R: ::rand::Rng>(_rng: &mut R) -> Self {
                    let variant = Rand!(
                        @isone [$($vname)*]
                        0,
                        _rng.gen_range(0, $num_variants));
                    match variant {
                    $(
                        $ord => Rand!(@enum $vkind _rng $name $vname $vfields),
                    )+
                        _ => loop { }
                    }
                }
            }
        }
    };
    // @isone: test if there is exactly one tt in the list, then $e else $f
    (@isone [$_one:tt] $e:expr, $_f:expr) => { $e };
    (@isone [$($_notone:tt)*] $_e:expr, $f:expr) => { $f };
    (@enum unitary $rng:ident $name:ident $vname:ident $vfields:tt) => {
        $name::$vname
    };
    (@enum tuple $rng:ident $name:ident $vname:ident
     [$($vfield:tt,)*]
    ) => {
        $name::$vname($(Rand!(@substitute $vfield $rng.gen())),*)
    };
    (@enum record $rng:ident $name:ident $vname:ident
     [$({
         ord: $_ford:tt,
         attrs: $_fattrs:tt,
         vis: $_fvis:tt,
         ty: $_fty:ty,
         name: $fname:ident,
      },)*]
    ) => {
        $name::$vname {
            $(
                $fname: $rng.gen()
            ),*
        }
    };
    // struct
    (
        @item
        struct {
            attrs: $_attrs:tt,
            vis: $_vis:tt,
            name: $name:ident,
            generics: {
                constr: [$($constr:tt)*],
                params: [$($params:tt)*],
                ltimes: $_ltimes:tt,
                tnames: [$($tnames:ident,)*],
            },
            where: {
                clause: $_clause:tt,
                preds: [$($preds:tt)*],
            },
            kind: $kind:ident,
            fields: $fields:tt,
            $($_struct_tail:tt)*
        }
    ) => {
        Rand!{ @as_item
            impl<$($constr)*> ::rand::Rand for $name<$($params)*>
                where (): ::rand::Rand, $($tnames: ::rand::Rand,)* $($preds)*
            {
                fn rand<R: ::rand::Rng>(_rng: &mut R) -> Self {
                    Rand!{@struct $kind _rng $name $fields }
                }
            }
        }
    };
    (@struct unitary $rng:ident $name:ident $vfields:tt) => {
        $name
    };
    (@struct tuple $rng:ident $name:ident
     [$($vfield:tt,)*]
    ) => {
        $name($(Rand!(@substitute $vfield $rng.gen())),*)
    };
    (@struct record $rng:ident $name:ident
     [$({
         ord: $_ford:tt,
         attrs: $_fattrs:tt,
         vis: $_fvis:tt,
         ty: $_fty:ty,
         name: $fname:ident,
      },)*]
    ) => {
        $name {
            $(
                $fname: $rng.gen()
            ),*
        }
    };
    (@substitute $_input:tt $output:expr) => { $output };
    (@as_item $i:item) => { $i };
}

#[cfg(test)]
mod tests {
    //trace_macros!(true);
    use rand::random;
    macro_attr! {
        #[derive(Rand!, Debug)]
        enum Test {
            A, B, C,
        }
    }
    /*
       // Does not compile with 0 variants
    macro_attr! {
        #[derive(Rand!, Debug)]
        pub enum Test2 {
        }
    }
    */

    macro_attr! {
        #[derive(Rand!, Debug)]
        enum Test1 {
            A,
        }
    }
    macro_attr! {
        #[derive(Rand!, Debug)]
        enum Test2 {
            A,
            B,
        }
    }

    #[test]
    fn it_works() {
        let t: Test = random();
        println!("{:?}", t);
        let t1: Test1 = random();
        println!("{:?}", t1);
        let t2: Test2 = random();
        println!("{:?}", t2);
    }

    macro_attr! {
        #[derive(Rand!, Debug)]
        enum Test3 {
            A(i8),
            B(Test2),
        }
    }
    #[test]
    fn enum_tuplevar() {
        let t: Test3 = random();
        println!("{:?}", t);
    }

    macro_attr! {
        #[derive(Rand!, Debug)]
        enum TestS {
            A { x: u8, y: u8 },
            B { x: u8, y: u8, z: u8 },
            C { },
        }
    }
    macro_attr! {
        #[derive(Rand!, Debug)]
        enum TestS2 {
            A { },
        }
    }
    #[test]
    fn enum_structvar() {
        let t: TestS = random();
        println!("{:?}", t);
        let s: TestS2 = random();
        println!("{:?}", s);
    }

    macro_attr! {
        #[derive(Rand!, Debug)]
        enum TestGeneric1<T> where T: ::rand::Rand {
            A { x: T },
            B { x: u8, y: u8, z: u8 },
        }
    }

    macro_attr! {
        #[derive(Rand!, Debug)]
        enum TestGeneric2<T> {
            A { x: T },
            B { x: u8, y: u8, z: u8 },
        }
    }

    #[test]
    fn enum_generic() {
        let t: TestGeneric1<TestS> = random();
        println!("{:?}", t);
        let s: TestGeneric2<()> = random();
        println!("{:?}", s);
    }

    macro_attr! {
        #[derive(Rand!, Debug)]
        struct TestStruct;
    }

    macro_attr! {
        #[derive(Rand!, Debug)]
        struct TestStruct2 {
            x: u8,
            y: (),
        }
    }

    macro_attr! {
        #[derive(Rand!, Debug)]
        struct TestStruct3(u8, Test1);
    }

    macro_attr! {
        #[derive(Rand!, Debug)]
        struct TestStruct4<T, U> where T: 'static {
            x: T,
            y: U,
        }
    }

    macro_attr! {
        #[derive(Rand!, Debug)]
        struct TestStruct5 {}
    }


    #[test]
    fn struct_simple() {
        let t: TestStruct = random();
        println!("{:?}", t);
        let s: TestStruct2 = random();
        println!("{:?}", s);
        let u: TestStruct3 = random();
        println!("{:?}", u);
        let v: TestStruct4<TestStruct, TestStruct2> = random();
        println!("{:?}", v);
        let w: TestStruct5 = random();
        println!("{:?}", w);
    }

    macro_attr! {
        #[derive(Rand!, Debug)]
        struct BigStruct<T> {
            a: T,
            b: (),
            c: i32,
            d: i32,
            e: i32,
            f: u8,
            g: u8,
            h: u8,
            i: f32,
            j: f32,
            k: f32,
            l: f32,
            m: f32,
            n: f64,
            o: Test,
            p: Test1,
            q: TestStruct,
            r: u8,
            s: u8,
            t: u8,
            u: u8,
            v: u8,
            x: u8,
            y: u8,
            z: u8,
        }
    }

    #[test]
    fn struct_big() {
        let t: BigStruct<i32> = random();
        println!("{:?}", t);
    }
}
