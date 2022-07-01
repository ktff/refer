/*


Ima se struktura te (kod,data?) koji znaju pristupiti polju u strukturi.

Field su okosnica, lijepilo koje spaja korisnički struct s store struct.

Definitivno macro za definiciju svega toga.
*/

// pub trait Memory {
//     fn iter<'a, S: Structure<'a>>(&'a self) -> Box<dyn Iterator<Item = S>>;
// }

// // ! Owner je jedini koji može imati uni directional reference, drugi moraju imati bi directional reference.
// // ! Owner može imati i bi directional reference.
// // ! Samo jedna struktura može biti vlasnik druge. Memory je vlasnik root struktura.

use std::marker::PhantomData;

pub trait Structure {
    //<'a>: Container + Clone + 'a {
    // type Iter: Iterator<Item = Self::R> + 'a;
    // type R: Reference<'a>;

    // /// Edges from this vertice.
    // fn edges(&self) -> Self::Iter;

    fn get<F: Field>(&self, field: &F) -> F::Item;

    // fn iter<R: Reference<'a>>(&self) -> dyn Iterator<Item = R>;

    // fn field<R: Reference<'a>>(&self, name: &str) -> Option<R>;
}

// Node: List<&Node> { ... }

macro_rules! structure {
    (pub struct $name:ident {$($($attr:meta)* $($vis:vis)? $name:ident : $ty:$ty),*} in $store:ty) => {
        pub struct $name{
            store: $store,
            $(
                $($attr)*
                $($vis)* $name: field!($ty, $store)
            ),*
        }
    };
}

macro_rules! field {
    (... , $store:ty) => {
        
    };
}

macro_rules! inner_structure{
    ({$($attr:meta* $vis:vis? $name:ident : $ty:$ty),*})={

    }
}


structure!{
    pub struct ExampleBinaryNode {
        data: u32,
        parent: Option<&Self>,
        left: Option<Self>,
        right: Option<Self>,
    } in PlainObject
};

///
/// T {
///     ...
///     NAME: Self
///     ...
/// }
///
/// stores Item in S.
pub trait Field<T, S> {
    const NAME: &'static str;

    type Item;
}

type Raw<T> = [u8];
pub enum RawObject<'a,T>{
    Inlined(Box<Raw<T>>),
    Ref(&'a Raw<T>),
}

impl<'a,T> RawObject<'a,T>{
    pub fn get(&self) -> &Raw<T>{
        match self{
            RawObject::Inlined(b) => b.as_ref(),
            RawObject::Ref(r) => r,
        }
    }

    pub fn as_ref<'b>(&'b self) -> RawObject<'b,T>{
        match self{
            RawObject::Inlined(b) => RawObject::Ref(b),
            RawObject::Ref(r) => RawObject::Ref(r),
        }
    }
}

pub type ExampleBinaryNodeRaw<'a>=RawObject<'a,ExampleBinaryNodeRawStruct>;


// Ponaša se kao fat pointer
pub struct ExampleBinaryNodeRawRef<'a> {
    data: ExampleBinaryNodeRaw<'a>,
    fields: ExampleBinaryNodeRawFields,
}

impl<'a> ExampleBinaryNodeRefRaw<'a>{
    pub fn new(data: ExampleBinaryNodeRaw<'a>) -> Self{
        ExampleBinaryNodeRefRaw{         
            fields: ExampleBinaryNodeRawFields::new(data.get()),
            data,
        }
    }

    pub fn data(&self)->u32{  
        self.fields.data(self.data.get())
    }

    pub fn parent(&self)->Option<ExampleBinaryNodeRawRef<'a>>{       
        let parent= self.fields.parent(self.data.get());

        parent.map(Self::new)
    }

    pub fn left<'b>(&'b self)->Option<ExampleBinaryNodeRawRef<'b>>{
        let left= self.fields.left(self.data.get());

        left.map(Self::new)
    }

    pub fn right<'b>(&'b self)->Option<ExampleBinaryNodeRawRef<'b>>{
        let right= self.fields.right(self.data.get());

        right.map(Self::new)
    }
}

struct ExampleBinaryNodeRawStruct<'a>{
    // data: u32,
    // parent: Option<&'a ExampleBinaryNodeRaw<'a>>,
    // left: Option<ExampleBinaryNodeRaw<'a>>,
    // right: Option<ExampleBinaryNodeRaw<'a>>,
}

pub struct ExampleBinaryNodeRawFields<'a> {
    data: Property<u32>,
    parent: Ref<Option<ExampleBinaryNodeRaw<'a>>>,
    left: Owned<Option<ExampleBinaryNodeRaw<'a>>>,
    right: Owned<Option<ExampleBinaryNodeRaw<'a>>>,
}

impl ExampleBinaryNodeRawFields{
    pub fn new(raw: &Raw<ExampleBinaryNodeRawStruct>)->Self{
        unimplemented!()
    }

    pub fn data(&self,raw: &Raw<ExampleBinaryNodeRawStruct>)->u32{        
        unimplemented!()
    }

    pub fn parent(&self,raw: &'a Raw<ExampleBinaryNodeRawStruct>)->Option<ExampleBinaryNodeRaw<'a>>{       
        unimplemented!()
    }

    pub fn left<'b>(& self,raw: &'b Raw<ExampleBinaryNodeRawStruct>)->Option<ExampleBinaryNodeRaw<'b>>{
        unimplemented!()
    }

    pub fn right<'b>(& self,raw: &'b Raw<ExampleBinaryNodeRawStruct>)->Option<ExampleBinaryNodeRaw<'b>>{
        unimplemented!()
    }
}



type Plain<T>=T;

pub enum PlainObject<'a,T>{
    Inlined(Box<Plain<T>>),
    Ref(&'a Plain<T>),
}

impl<'a,T> PlainObject<'a,T>{
    pub fn get(&self) -> &Plain<T>{
        match self{
            PlainObject::Inlined(b) => b.as_ref(),
            PlainObject::Ref(r) => r,
        }
    }

    pub fn as_ref<'b>(&'b self) -> PlainObject<'b,T>{
        match self{
            PlainObject::Inlined(b) => PlainObject::Ref(b),
            PlainObject::Ref(r) => PlainObject::Ref(r),
        }
    }
}

pub type ExampleBinaryNode<'a>=PlainObject<'a,ExampleBinaryNodeStruct>;


// Ponaša se kao fat pointer
pub struct ExampleBinaryNodeRef<'a,B> {
    backend: &'a B,
    data: ExampleBinaryNode<'a>,
    fields: ExampleBinaryNodeFields,
}

impl<'a,B> ExampleBinaryNodeRef<'a,B>{
    pub fn new( backend: &'a B,data: ExampleBinaryNode<'a>) -> Self{
        ExampleBinaryNodeRef{         
            fields: ExampleBinaryNodeFields::new(data.get()),
            data,
        }
    }

    pub fn data(&self)->u32{
        self.fields.data(self.data.get())
    }

    pub fn parent(& self)->Option<ExampleBinaryNodeRef<'a>>{
        let parent= self.fields.parent(self.data.get());

        parent.map(Self::new)
    }

    pub fn left<'b>(&'b self)->Option<ExampleBinaryNodeRef<'b>>{
        let left= self.fields.left(self.data.get());

        left.map(Self::new)
    }

    pub fn right<'b>(&'b self)->Option<ExampleBinaryNodeRef<'b>>{
        let right= self.fields.right(self.data.get());

        right.map(Self::new)
    }
    
}

//! NOTE: Sa #[repr(packed, C)] ili bez njega
struct ExampleBinaryNodeStruct<'a>{
    data: u32,
    parent: Option<&'a ExampleBinaryNode<'a>>,
    left: Option<ExampleBinaryNode<'a>>,
    right: Option<ExampleBinaryNode<'a>>,
}

pub struct ExampleBinaryNodeFields {
    // data: Property<u32>,
    // parent: Ref<Option<Self>>,
    // left: Owned<Option<Self>>,
    // right: Owned<Option<Self>>,
}


impl ExampleBinaryNodeFields{
    pub fn new(raw: &Plain<ExampleBinaryNodeStruct>)->Self{
        Self{}
    }

    pub fn data(&self,data: &Plain<ExampleBinaryNodeStruct>)->u32{        
       data.data
    }

    pub fn parent(&self,data: &Plain<ExampleBinaryNodeStruct>)->Option<ExampleBinaryNode<'a>>{       
        data.parent.map(|data|data.as_ref())
    }

    pub fn left<'b>(& self,data: &'b Plain<ExampleBinaryNodeStruct>)->Option<ExampleBinaryNode<'b>>{
        data.left.as_ref().map(|data|data.as_ref())
    }

    pub fn right<'b>(& self,data: &'b Plain<ExampleBinaryNodeStruct>)->Option<ExampleBinaryNode<'b>>{
        data.right.as_ref().map(|data|data.as_ref())
    }
}

/*
! Process:

For each field:
 0. Select field -- function or a struct field, specificno za strukturu
 1. Find field -- ovisi o ostalim poljima, specificno za strukturu
 2. Read field -- genericno po field atributima
 3. Map field -- genericno po targetu fielda

*/



pub struct ExampleRichEdge{
    data: u32,

    next: Option<To<Self,bool>>,
}

pub struct To<T,With>{}


pub trait Access{

}

impl Access for Option<>

// // EDGE
// pub trait Reference<'a>: Container + Clone + 'a {
//     type T: Structure<'a>;

//     fn to(&self) -> Self::T;
// }

pub struct ExampleBinaryNode {
    data: Property<u32>,
    parent: Ref<Option<Self>>,
    left: Owned<Option<Self>>,
    right: Owned<Option<Self>>,
}

pub struct ExampleRootCollection {
    roots: List<Owned<ExampleBinaryNode>>,
    active: Set<Ref<UnsignedInteger>>,
    string_map: Map<DynProperty<str>, Ref<Str>>,
}

pub struct Common {
    import_id: i64,
    position: [f64; 2],
}

pub struct WayNode {
    common: Property<Common>,
    other: List<Ref<WayNode>>,
}

pub struct ExampleWay {
    nodes: Tree<WayStateMachine>,
}

pub enum WayStateMachine {
    Start,
    Line,
    End,
}

pub trait TreeStateMachine: Copy {
    /// Vertices of tree
    type V;

    // NOTE: Nešto ovakvo, ali je to pre detaljno za sada
    fn start(root: Self::V) -> Option<Self>;

    fn next(self, to: Self::V) -> Option<Self>;
}

pub struct SignedInteger {
    value: Property<i128>,
}

pub struct UnsignedInteger {
    value: Property<u128>,
}

pub struct Str {
    value: DynProperty<str>,
}

pub struct Offset(usize);

// ************************ Field ************************

pub enum DynProperty<T: ?Sized> {
    Tmp(Offset),
    Ref(Offset),
}

pub enum Property<T> {
    Tmp(T),
    Cached(T, Offset),
    Ref(Offset),
}

pub enum Owned<T> {
    Tmp(Offset),
    Ref(Offset),
}

pub enum Ref<T> {
    Ref(Offset),
}

// // Tell the garbage collector how to explore a graph of this object
// impl Trace<Self> for Object {
//     fn trace(&self, tracer: &mut Tracer<Self>) {
//         match self {
//             Object::Num(_) => {},
//             Object::List(objects) => objects.trace(tracer),
//         }
//     }
// }
