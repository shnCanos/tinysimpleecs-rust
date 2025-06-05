use std::{
    alloc::{alloc, Allocator, Global, Layout}, any::{Any, TypeId}, cell::RefCell, collections::HashMap, num::NonZero, ops::{Index, IndexMut}, ptr::NonNull, rc::Rc
};

use tinysimpleecs_rust_macros::implement_bundle;

pub(crate) struct ComponentCollumn {
    /// The id used for the bitmask
    id: usize,
    data: NonNull<[u8]>,
    item_layout: Layout,
    current_layout: Layout,
    capacity: usize,
    len: usize
}

impl ComponentCollumn {
    // I will need to check this later
    pub(crate) fn new<C: Component>(id: usize) -> Self {
        let layout = Layout::new::<C>();
        assert_ne!(layout.align(), 0); // Alignment mustn't equal 0
        Self {
            data: unsafe {NonNull::new_unchecked(layout.align() as *mut u8)},
            item_layout: layout,
            current_layout: layout,
            capacity: 0,
            len: 0,
            id,
        }
    }

    pub(crate) fn id(&self) -> usize {
        return self.id;
    }

    // -- Copied straight from bevy's BlobVec -- //

    /// Returns the number of elements in the vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the [`Layout`] of the element type stored in the vector.
    #[inline]
    pub fn layout(&self) -> Layout {
        self.item_layout
    }

    // -- // -- //

    pub(crate) fn ensure_minimum(&mut self, new_capacity: usize) {
        if new_capacity <= self.capacity {
            return;
        }
        let allocator = Global;

        let new_capacity = self.capacity + self.capacity.max(new_capacity);
        let new_layout = self.layout().repeat_packed(new_capacity).unwrap();
        let ptr = unsafe {
            if self.capacity == 0 {
                self.data
            } else {
                allocator.grow(self.data, self.current_layout, new_layout).unwrap()
            }
        };
    }
    
    /// Pushes a raw component into the ComponentCollumn
    ///
    /// # Safety
    ///
    /// `raw_component`'s `TypeId` must match `ComponentCollumn`'s key
    pub(crate) unsafe fn push_raw(&mut self, raw_component: *const u8) {
        self.data.extend_from_slice(bytes);
    }

    /// Pushes a component into the ComponentCollumn
    ///
    /// # Safety
    ///
    /// `component`'s `TypeId` must match `ComponentCollumn`'s key
    pub(crate) unsafe fn push<C: Component>(&mut self, component: C) {
        debug_assert_eq!(Layout::new::<C>(), self.layout(), "Attempted to push value with different layout from expected");
        let bytes =
            std::slice::from_raw_parts(&component as *const C as *const u8, self.item_layout.size());
        self.data.extend_from_slice(bytes);
    }

    /// Gets a component from its index in ComponentCollumn
    ///
    /// # Safety
    ///
    /// `component`'s `TypeId` must match `ComponentCollumn`'s key
    pub(crate) fn get_index_mut<C: Component>(&mut self) -> &mut C {
        let ptr = self.data.as_ptr().add()
    }
}

#[derive(Default)]
pub(crate) struct ComponentManager {
    components: HashMap<TypeId, ComponentCollumn>,
    last_used_id: usize,
}

impl ComponentManager {
    pub(crate) fn new() -> Self {
        return Self::default();
    }

    pub(crate) fn register_component_unchecked<C: Component>(&mut self) {
        let type_id = TypeId::of::<C>();
        let result = self.components.insert(type_id, AnyVec::new::<C>());
        debug_assert!(result.is_none());
    }

    pub(crate) fn uncheked_component_vec<C: Component>(&mut self) -> &mut AnyVec {
        let result = self.components.get_mut(&TypeId::of::<C>());
        debug_assert!(result.is_some());
        return result.unwrap();
    }

    pub(crate) fn register_component_if_not_exists<C: Component>(&mut self) {
        let type_id = TypeId::of::<C>();
        if self.components.contains_key(&type_id) {
            return;
        }
        self.register_component_unchecked::<C>();
    }

    pub(crate) fn add_component_as<C: Component>(&mut self, component: C) {
        self.register_component_if_not_exists::<C>();
        let type_id = TypeId::of::<C>();
        let anyvec = self.components.get_mut(&type_id).unwrap();
        anyvec.push(component);
    }
}

pub trait Component: std::fmt::Debug {}

pub trait Bundle {
    fn add(self, manager: &mut ComponentManager);
}

variadics_please::all_tuples!(implement_bundle, 1, 15, B);
