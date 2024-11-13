use cs2_schema_cutl::{
    CUtlVector,
    FixedCStr,
    PtrCStr,
    UtlRBTree,
};
use raw_struct::{
    builtins::Ptr64,
    raw_struct,
    Copy,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AtomicCategory {
    Basic = 0,
    T = 1,
    CollectionOfT = 2,
    TF = 3,
    TT = 4,
    TTF = 5,
    I = 6,
    Unknown = 7,
    None = 8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeCategory {
    Builtin = 0,
    Ptr = 1,
    Bitfield = 2,
    FixedArray = 3,
    Atomic = 4,
    DeclaredClass = 5,
    DeclaredEnum = 6,
    None = 7,
}

#[raw_struct(size = 0x10)]
pub struct IdHashEntr {
    #[field(offset = 0x00)]
    pub id: u64,

    #[field(offset = 0x08)]
    pub value: Ptr64<()>,
}

#[raw_struct(size = 0x200)]
pub struct CSchemaSystem {
    #[field(offset = 0x188)]
    pub scopes: Copy<dyn CUtlVector<Ptr64<dyn CSchemaSystemTypeScope>>>,
}

#[derive(Debug, Clone, Copy)]
pub struct IdHashEntry {
    pub id: u64,
    pub value: Ptr64<()>,
}

#[raw_struct(size = 0x5620)]
pub struct CSchemaSystemTypeScope {
    #[field(offset = 0x08)]
    pub scope_name: FixedCStr<0x100>,
    // pub parent_scope: Ptr64<dyn CSchemaSystemTypeScope> = 0x108,
    // pub buildin_types_initialized: bool = 0x110,
    // pub type_buildin: CSchemaType[14] = 0x118,

    // The UtlRBTree entries are all at offset 0x08. First is a bool (probably indicating if existing)
    // pub type_ptr: ??? = 0x348,
    // pub type_atomic: ??? = 0x370,
    // pub type_atomic_t: ??? = 0x398,
    // pub type_atomic_collection_of_t: ??? = 0x3C0
    // pub type_atomic_tf: ??? = 0x3E8
    // pub type_atomic_tt: ??? = 0x410
    // pub type_atomic_tt: ??? = 0x438
    // pub type_atomic ttf: ??? = 0x460
    // pub type_atomic_i: ??? = 0x488
    // pub type_???: ??? = 0x4B0
    // pub type_atomic_i: ??? = 0x4D8
    // pub type_atomic_i: ??? = 0x488
    #[field(offset = 0x440)]
    pub type_declared_class: Copy<dyn UtlRBTree<IdHashEntry>>,

    #[field(offset = 0x468)]
    pub type_declared_enum: Copy<dyn UtlRBTree<IdHashEntry>>,
    /* 0x500 contains A CUtlMemoryPoolBase */
    /* 0x580 contains 0x100 elements of size 0x28 */

    /* 0x2D90 contains A CUtlMemoryPoolBase */
    /* 0x2E10 contains 0x100 elements of size 0x28 */
    // pub type_???: ??? = 0x528
    // pub type_fixed_array: ??? = 0x558
    // pub type_bit_fields: ??? = 0x588

    // pub class_bindings: CUtlTSHash<u64, Ptr<CSchemaClassBinding>> = 0x5C0,
    // pub enum_bindings: CUtlTSHash<u64, Ptr<CSchemaEnumBinding>> = 0x2E50,
}

#[raw_struct(size = 0x20)]
pub struct CSchemaType {
    #[field(offset = 0x00)]
    pub vtable: u64,

    #[field(offset = 0x08)]
    pub var_type: PtrCStr,

    #[field(offset = 0x10)]
    pub var_type_scope: Ptr64<dyn CSchemaSystemTypeScope>,

    #[field(offset = 0x18)]
    pub type_category: TypeCategory,

    #[field(offset = 0x19)]
    pub atomic_category: AtomicCategory,
}

#[raw_struct(size = 0x28)]
pub struct CSchemaTypeBuildin {
    #[field(offset = 0x20)]
    pub index: u8,
}
impl CSchemaType for dyn CSchemaTypeBuildin {}

#[raw_struct(size = 0x28)]
pub struct CSchemaTypeDeclaredEnum {
    #[field(offset = 0x20)]
    pub declaration: Ptr64<dyn CSchemaEnumBinding>,
}
impl CSchemaType for dyn CSchemaTypeDeclaredEnum {}

#[raw_struct(size = 0x30)]
pub struct CSchemaTypeDeclaredClass {
    #[field(offset = 0x20)]
    pub declaration: Ptr64<dyn CSchemaClassBinding>,
}
impl CSchemaType for dyn CSchemaTypeDeclaredClass {}

#[raw_struct(size = 0x30)]
pub struct CSchemaTypePtr {
    #[field(offset = 0x20)]
    pub base_type: Ptr64<dyn CSchemaType>,
    /* unknown value which is sometimes 1 (maybe "*" count?) */
}
impl CSchemaType for dyn CSchemaTypePtr {}

#[raw_struct(size = 0x30)]
pub struct CSchemaTypeFixedArray {
    #[field(offset = 0x20)]
    pub array_length: u32,

    #[field(offset = 0x28)]
    pub base_type: Ptr64<dyn CSchemaType>,
}
impl CSchemaType for dyn CSchemaTypeFixedArray {}

#[raw_struct(size = 0x20)]
pub struct CSchemaTypeAtomic {/* I'm unaware of "special fields" */}
impl CSchemaType for dyn CSchemaTypeAtomic {}

#[raw_struct(size = 0x40)]
pub struct CSchemaTypeAtomicT {
    /* FIXME: Where is the "handle type" field? */
    #[field(offset = 0x30)]
    pub inner_type: Ptr64<dyn CSchemaType>,
}
impl CSchemaType for dyn CSchemaTypeAtomicT {}
impl CSchemaTypeAtomic for dyn CSchemaTypeAtomicT {}

#[raw_struct(size = 0x40)]
pub struct CSchemaTypeAtomicCollectionOfT {}
impl CSchemaType for dyn CSchemaTypeAtomicCollectionOfT {}
impl CSchemaTypeAtomic for dyn CSchemaTypeAtomicCollectionOfT {}
impl CSchemaTypeAtomicT for dyn CSchemaTypeAtomicCollectionOfT {}

#[raw_struct(size = 0x10)]
pub struct CSchemaMetadataEntry {
    #[field(offset = 0x00)]
    pub name: PtrCStr,
    // ptr to metadata value
    // - const char*
    // - float
    // - int
    // - void*
    #[field(offset = 0x08)]
    pub metadata_value: Ptr64<u64>,
}

#[raw_struct(size = 0x10)]
pub struct CSchemaMetadataVarNames {
    #[field(offset = 0x00)]
    pub var_name: PtrCStr,

    #[field(offset = 0x08)]
    pub var_type: PtrCStr,
}

#[raw_struct(size = 0x20)]
pub struct CSchemaClassField {
    #[field(offset = 0x00)]
    pub name: PtrCStr,

    #[field(offset = 0x08)]
    pub field_type: Ptr64<dyn CSchemaType>,

    #[field(offset = 0x10)]
    pub offset: u32,

    #[field(offset = 0x14)]
    pub metadata_size: u32,

    #[field(offset = 0x18)]
    pub metadata: Ptr64<[Copy<dyn CSchemaMetadataEntry>]>,
}

#[raw_struct(size = 0x68)]
pub struct CSchemaClassBinding {
    #[field(offset = 0x00)]
    pub parent: Ptr64<dyn CSchemaClassBinding>,

    #[field(offset = 0x08)]
    pub name: PtrCStr,

    #[field(offset = 0x10)]
    pub module_name: PtrCStr,

    #[field(offset = 0x18)]
    pub size: u32, // Size of own struct

    #[field(offset = 0x1C)]
    pub field_size: u16,

    #[field(offset = 0x1E)]
    pub static_size: u16,

    #[field(offset = 0x20)]
    pub metadata_size: u16,

    #[field(offset = 0x28)]
    pub fields: Ptr64<[Copy<dyn CSchemaClassField>]>,

    /* pub static_fields: Ptr<[CSchemaStaticField]> = 0x30, */
    #[field(offset = 0x38)]
    pub base_class: Ptr64<dyn CSchemaClassInheritance>,

    #[field(offset = 0x48)]
    pub metadata: Ptr64<[Copy<dyn CSchemaMetadataEntry>]>,

    #[field(offset = 0x50)]
    pub type_scope: Ptr64<dyn CSchemaSystemTypeScope>,

    #[field(offset = 0x58)]
    pub schema_type: Ptr64<dyn CSchemaType>,

    #[field(offset = 0x60)]
    pub flags: u64,
}

#[raw_struct(size = 0x10)]
pub struct CSchemaClassInheritance {
    #[field(offset = 0x08)]
    pub class_binding: Ptr64<dyn CSchemaClassBinding>,
}

#[raw_struct(size = 0x40)]
pub struct CSchemaEnumBinding {
    #[field(offset = 0x08)]
    pub name: PtrCStr,

    #[field(offset = 0x10)]
    pub module_name: PtrCStr,

    #[field(offset = 0x18)]
    pub size: u8, // Size of own struct

    #[field(offset = 0x1C)]
    pub member_count: u16,

    #[field(offset = 0x1E)]
    pub flags: u16,

    #[field(offset = 0x20)]
    pub members: Ptr64<[Copy<dyn CSchemaEnumMember>]>,

    #[field(offset = 0x30)]
    pub type_scope: Ptr64<dyn CSchemaSystemTypeScope>,
}

#[raw_struct(size = 0x20)]
pub struct CSchemaEnumMember {
    #[field(offset = 0x00)]
    pub name: PtrCStr,

    #[field(offset = 0x08)]
    pub value: u64,
}
