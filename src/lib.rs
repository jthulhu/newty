#[cfg(test)]
mod tests {
    use super::{newty, nvec};

    #[test]
    fn vec_newtype() {
	newty! {
	    pub id InstructionPointer
	    impl {
		pub fn incr(&self) -> Self {
		    Self(self.0+1)
		}
	    }
	}
	newty! {
	    #[derive(PartialEq)]
	    pub vec Program(usize)[InstructionPointer]
	    impl {
		pub fn len_ip(&self) -> InstructionPointer {
		    InstructionPointer(self.len())
		}
	    }
	}

	let a = nvec![Program 1, 2, 3];
	let i = InstructionPointer::from(0);
	assert_eq!(a.len_ip(), i.incr().incr().incr());
	assert_eq!(a[i], 1);
	let i = i.incr();
	assert_eq!(a[i], 2);
	let i = i.incr();
	assert_eq!(a[i], 3);
    }

    #[test]
    fn slice_newtype() {
	newty! {
	    pub id InstructionPointer
	    impl {
		pub fn incr(&self) -> Self {
		    Self(self.0+1)
		}
	    }
	}
	newty! {
	    #[derive(PartialEq)]
	    pub vec Program(usize)[InstructionPointer]
	}
	newty! {
	    pub slice ProgramSlice(usize)[InstructionPointer]
		of Program
	}

	let a = nvec![Program 1, 2, 3];
	let b: &ProgramSlice = &a;
	assert_eq!(b.len(), 3);
	let i = InstructionPointer::from(0);
	assert_eq!(b[i], 1);
	let i = i.incr();
	assert_eq!(b[i], 2);
	let i = i.incr();
	assert_eq!(b[i], 3);
    }

    #[cfg(feature="set")]
    #[test]
    fn set_newtype() {
	newty! {
	    pub id InstructionPointer
	    impl {
		fn incr(&self) -> InstructionPointer {
		    Self(self.0+1)
		}
	    }
	}
	newty! {
	    set DoneThreads[InstructionPointer]
	}

	let mut a = DoneThreads::with_capacity(InstructionPointer::from(10));
	let i = InstructionPointer::from(0);
	a.insert(i);
	assert!(a.contains(i));
	assert!(!a.contains(i.incr()));
    }
    
    #[test]
    fn id_newtype() {
	newty! {
	    pub id InstructionPointer
	    impl {
		pub fn incr(&self) -> Self {
		    Self(self.0+1)
		}
	    }
	}

	let i = InstructionPointer::from(0);
	let ip1 = InstructionPointer::from(1);
	assert_eq!(i.incr(), ip1);
    }
}

#[macro_export]
macro_rules! newty {
    (@slice $(#[$($meta:meta),*])* $visibility:vis $name:ident
     ($interior_type:ty) [$indexer:ty] of $vec_type:ty) => {
	$crate::newty!{
	    @slice
	    $(#[$($meta),*])*
	    $visibility $name($interior_type)
	    [$indexer :getter |x: $indexer| $crate::Wrapper::dewrap_into(x)]
	    of $vec_type
	}
    };
    (@slice $(#[$($meta:meta),*])* $visibility:vis $name:ident
     ($interior_type:ty) [$indexer:ty :getter $getter:expr] of $vec_type:ty) => {
	$(#[$($meta),*])*
	#[derive(Debug)]
	#[repr(transparent)]
	$visibility struct $name([$interior_type]);

	impl $name {
	    #[inline]
	    $visibility fn len(&self) -> usize {
		self.0.len()
	    }
	}

	impl std::convert::AsRef<$name> for $vec_type {
	    fn as_ref(&self) -> &$name {
		unsafe { &*(self.0.as_ref() as *const [_] as *const $name) }
	    }
	}

	impl ::std::ops::Deref for $vec_type {
	    type Target = $name;

	    fn deref(&self) -> &Self::Target {
		self.as_ref()
	    }
	}

	impl ::std::ops::Index<$indexer> for $name {
	    type Output = $interior_type;

	    fn index(&self, index: $indexer) -> &Self::Output {
		&self.0[$getter(index)]
	    }
	}
    };

    (@map $(#[$($meta:meta),*])* $visibility:vis $name:ident($value:ty)[$key:ty]
     $(impl { $($method:item)* })?) => {
	$(#[$($meta),*])*
	#[derive(Debug, Default, PartialEq, Eq)]
	$visibility struct $name(::hashbrown::HashMap<$key, $value>);

	impl $name {
	    #![allow(dead_code)]
	    
	    #[inline]
	    $visibility fn new() -> Self {
		Self(::hashbrown::HashMap::new())
	    }

	    #[inline]
	    $visibility fn len(&self) -> usize {
		self.0.len()
	    }

	    #[inline]
	    $visibility fn len_as(&self) -> $key {
		self.0.len().into()
	    }

	    #[inline]
	    $visibility fn iter(&self) -> ::hashbrown::hash_map::Iter<'_, $key, $value> {
		self.0.iter()
	    }

	    #[inline]
	    $visibility fn iter_mut(&mut self) -> ::hashbrown::hash_map::IterMut<'_, $key, $value> {
		self.0.iter_mut()
	    }

	    #[inline]
	    $visibility fn is_empty(&self) -> bool {
		self.0.is_empty()
	    }
	    
	    #[inline]
	    $visibility fn insert(&mut self, key: $key, value: $value) {
		self.0.insert(key, value);
	    }

	    #[inline]
	    $visibility fn get(&self, key: $key) -> Option<$value> {
		self.0.get(key)
	    }
	}

	$(impl $name {
	    $($method)*
	})?

	impl From<::hashbrown::HashMap<$key, $value>> for $name {
	    fn from(hm: HashMap<$key, $value>) -> Self {
		Self(hm)
	    }
	}

	impl ::std::ops::Index<$key> for $name {
	    type Output = $value;

	    fn index(&self, index: $key) -> &Self::Output {
		&self.0[&index]
	    }
	}
    };

    (@set $(#[$($meta:meta),*])* $visibility:vis $name:ident [$indexer:ty]) => {
	$crate::newty!{
	    @set
	    $(#[$($meta),*])*
	    $visibility $name
	    [$indexer :getter |x: $indexer| $crate::Wrapper::dewrap_into(x)]
	}
    };

    (@set $(#[$($meta:meta),*])* $visibility:vis $name:ident
     [$indexer:ty :getter $getter:expr]) => {	
	#[cfg(feature="serde")]
	$(#[$($meta),*])*
	#[derive(Debug)]
	#[derive(::serde::Serialize, ::serde::Deserialize)]
	#[allow(missing_docs)]
	$visibility struct $name(fixedbitset::FixedBitSet);

	#[cfg(not(feature="serde"))]
	$(#[$($meta),*])*
	#[derive(Debug)]
	#[allow(missing_docs)]
	$visibility struct $name(fixedbitset::FixedBitSet);

	impl std::fmt::Display for $name {
	    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	    }
	}

	impl $name {
	    #![allow(dead_code)]
	    
	    $visibility const fn new() -> Self {
		Self(::fixedbitset::FixedBitSet::new())
	    }

	    $visibility fn from_vec(size: $indexer, vec: Vec<$indexer>) -> Self {
		let mut set = ::fixedbitset::FixedBitSet::with_capacity($getter(size));
		for i in vec {
		    set.insert($getter(i));
		}
		Self(set)
	    }

	    #[inline]
	    $visibility fn with_capacity(size: $indexer) -> Self {
		Self(::fixedbitset::FixedBitSet::with_capacity($getter(size)))
	    }

	    #[inline]
	    $visibility fn with_raw_capacity(size: usize) -> Self {
		Self(::fixedbitset::FixedBitSet::with_capacity(size))
	    }

	    #[inline]
	    $visibility fn len(&self) -> usize {
		self.0.len()
	    }

	    #[inline]
	    $visibility fn is_empty(&self) -> bool {
		self.0.len() == 0
	    }

	    #[inline]
	    $visibility fn len_as(&self) -> $indexer {
		self.0.len().into()
	    }

	    #[inline]
	    $visibility fn contains(&self, bit: $indexer) -> bool {
		self.0.contains($getter(bit))
	    }

	    #[inline]
	    $visibility fn set_range(
		&mut self,
		::std::ops::Range { start, end }: ::std::ops::Range<$indexer>,
		enabled: bool
	    ) {
		self.0.set_range($getter(start)..$getter(end), enabled)
	    }

	    #[inline]
	    $visibility fn insert_range(&mut self, range: ::std::ops::Range<$indexer>) {
		self.set_range(range, true)
	    }

	    #[inline]
	    $visibility fn toggle_range(
		&mut self,
		::std::ops::Range { start, end }: ::std::ops::Range<$indexer>
	    ) {
		self.0.toggle_range($getter(start)..$getter(end))
	    }

	    #[inline]
	    $visibility fn as_slice(&self) -> &[u32] {
		self.0.as_slice()
	    }

	    #[inline]
	    $visibility fn as_mut_slice(&mut self) -> &mut [u32] {
		self.0.as_mut_slice()
	    }

	    #[inline]
	    $visibility fn insert(&mut self, bit: $indexer) {
		self.0.insert($getter(bit))
	    }

	    #[inline]
	    $visibility fn put(&mut self, bit: $indexer) -> bool {
		self.0.put($getter(bit))
	    }

	    #[inline]
	    $visibility fn toggle(&mut self, bit: $indexer) {
		self.0.toggle($getter(bit))
	    }

	    #[inline]
	    $visibility fn set(&mut self, bit: $indexer, enabled: bool) {
		self.0.set($getter(bit), enabled)
	    }
	}
    };

    (@id $(#[$($meta:meta),*])* $visibility:vis $name:ident
     $(impl { $($method:item)* })?) => {
	$crate::newty!{
	    @id
	    $(#[$($meta),*])*
	    #[derive(Copy, PartialEq, Eq, Hash)]
	    $visibility $name(usize)
	    $(impl { $($method)* })?
	}
    };
    
    (@id $(#[$($meta:meta),*])* $visibility:vis $name:ident
     ($interior_type:ty) $(impl { $($method:item)* })?) => {
	#[cfg(feature="serde")]
	$(#[$($meta),*])*
	#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
	$visibility struct $name(pub $interior_type);

	#[cfg(not(feature="serde"))]
	$(#[$($meta),*])*
	#[derive(Debug, Clone)]
	$visibility struct $name(pub $interior_type);

	$(impl $name {
	    $($method)*
	})?

	impl From<$interior_type> for $name {
	    fn from(x: $interior_type) -> Self {
		Self(x)
	    }
	}

	impl $crate::Wrapper for $name {
	    type Wrapped = $interior_type;
	    
	    fn dewrap(&self) -> &Self::Wrapped {
		&self.0
	    }
	    
	    fn dewrap_into(self) -> Self::Wrapped {
		self.0
	    }
	}

	impl ::std::fmt::Display for $name {
	    fn fmt(
		&self,
		f: &mut ::std::fmt::Formatter<'_>
	    ) -> ::std::result::Result<(), ::std::fmt::Error>
	    {
		write!(f, "{}", self.0)
	    }
	}
    };

    (@vec $(#[$($meta:meta),*])* $visibility:vis $name:ident($interior_type:ty)
     [$indexer:ty] $($rest:tt)*) => {
	$crate::newty!{
	    @vec
	    $(#[$($meta),*])*
	    $visibility $name($interior_type)
	    [$indexer :getter |x: $indexer| $crate::Wrapper::dewrap_into(x)]
	    $($rest)*
	}
    };

    (@vec $(#[$($meta:meta),*])* $visibility:vis $name:ident($interior_type:ty)
     [$indexer:ty :getter $getter:expr] $(impl { $($method:item)* })?) => {
	$(#[$($meta),*])?
	#[derive(Debug, Default)]
	$visibility struct $name(Vec<$interior_type>);

	impl $name {
	    #![allow(dead_code)]
	    
	    #[inline]
	    $visibility fn new() -> Self {
		Self(Vec::new())
	    }

	    #[inline]
	    $visibility fn len(&self) -> usize {
		self.0.len()
	    }

	    #[inline]
	    $visibility fn len_as(&self) -> $indexer {
		self.0.len().into()
	    }

	    #[inline]
	    $visibility fn iter(&self) -> std::slice::Iter<'_, $interior_type> {
		self.0.iter()
	    }

	    #[inline]
	    $visibility fn iter_mut(&mut self) -> std::slice::IterMut<'_, $interior_type> {
		self.0.iter_mut()
	    }

	    #[inline]
	    $visibility fn is_empty(&self) -> bool {
		self.0.is_empty()
	    }

	    #[inline]
	    $visibility fn push(&mut self, element: $interior_type) {
		self.0.push(element)
	    }
	}

	$(impl $name {
	    $($method)*
	})?

	impl From<Vec<$interior_type>> for $name {
	    fn from(v: Vec<$interior_type>) -> Self {
		Self(v)
	    }
	}

	impl Extend<$interior_type> for $name {
	    fn extend<I: IntoIterator<Item=$interior_type>>(&mut self, iter: I) {
		self.0.extend(iter)
	    }
	}

	impl ::std::ops::Index<$indexer> for $name {
	    type Output = $interior_type;

	    fn index(&self, index: $indexer) -> &Self::Output {
		&self.0[$getter(index)]
	    }
	}
    };

    ($(#[$($meta:meta),*])* $visibility:vis $newtype:ident $name:ident
     $($rest:tt)*) => {
	$crate::newty!{@$newtype $(#[$($meta),*])* $visibility $name $($rest)*}
    };
}

#[macro_export]
macro_rules! nvec {
    ($nvec:tt $c:expr ; $nb:expr) => {
	$nvec::from(vec![$c ; $crate::Wrapper::dewrap_into($nb)])
    };
    ($nvec:tt $($rest:tt)*) => {
	$nvec::from(vec![$($rest)*])
    };
}

pub trait Wrapper: From<Self::Wrapped> {
    type Wrapped;

    fn dewrap(&self) -> &Self::Wrapped;
    fn dewrap_into(self) -> Self::Wrapped
    where
	Self: Sized;
}
