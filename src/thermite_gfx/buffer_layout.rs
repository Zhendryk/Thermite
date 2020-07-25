// use gl::{
//     self,
//     types::{GLboolean, GLfloat, GLuint},
// };
// use std::mem::size_of;

// /// The type/amount of data a `BufferComponent` contains
// #[derive(Clone)]
// pub enum BufferComponentType {
//     Float,
//     Float2,
//     Float3,
//     Float4,
//     Int,
//     Int2,
//     Int3,
//     Int4,
//     Mat3,
//     Mat4,
//     Bool,
// }

// /// Returns the total size in bytes of a given `BufferComponentType`
// fn size_for_component_type(kind: &BufferComponentType) -> usize {
//     match kind {
//         BufferComponentType::Float => size_of::<GLfloat>(),
//         BufferComponentType::Float2 => 2 * size_of::<GLfloat>(),
//         BufferComponentType::Float3 => 3 * size_of::<GLfloat>(),
//         BufferComponentType::Float4 => 4 * size_of::<GLfloat>(),
//         BufferComponentType::Int => size_of::<GLuint>(),
//         BufferComponentType::Int2 => 2 * size_of::<GLuint>(),
//         BufferComponentType::Int3 => 3 * size_of::<GLuint>(),
//         BufferComponentType::Int4 => 4 * size_of::<GLuint>(),
//         BufferComponentType::Mat3 => 3 * 3 * size_of::<GLfloat>(),
//         BufferComponentType::Mat4 => 4 * 4 * size_of::<GLfloat>(),
//         BufferComponentType::Bool => size_of::<GLboolean>(),
//     }
// }

// /// Returns the number of data points for a given `BufferComponentType`
// fn count_for_component_type(kind: &BufferComponentType) -> usize {
//     match kind {
//         BufferComponentType::Float => 1,
//         BufferComponentType::Float2 => 2,
//         BufferComponentType::Float3 => 3,
//         BufferComponentType::Float4 => 4,
//         BufferComponentType::Int => 1,
//         BufferComponentType::Int2 => 2,
//         BufferComponentType::Int3 => 3,
//         BufferComponentType::Int4 => 4,
//         BufferComponentType::Mat3 => 3, // 3 * Float3
//         BufferComponentType::Mat4 => 4, // 4 * Float4
//         BufferComponentType::Bool => 1,
//     }
// }

// /// A single component of a `BufferLayout`, usually containing data pertaining to a single Vertex Attribute
// #[derive(Clone)]
// pub struct BufferComponent {
//     name: String,
//     kind: BufferComponentType,
//     size: usize,
//     count: usize,
//     offset: usize,
//     normalized: bool,
// }

// impl BufferComponent {
//     /// Creates a new `BufferComponent` to be used in a `BufferLayout` for a `VertexBuffer`
//     ///
//     /// ### Parameters
//     ///
//     /// - `name`: The textual name of this `BufferComponent`
//     /// - `kind`: The type/amount of data this `BufferComponent` contains, represented as a `BufferComponentType`
//     /// - `normalized`: Specifies whether fixed-point data values should be normalized (true) or converted directly as fixed-point values (false) when they are accessed
//     ///
//     /// ### Returns
//     ///
//     /// A new `BufferComponent`, ready for insertion into a `BufferLayout`
//     pub fn new(name: String, kind: BufferComponentType, normalized: bool) -> BufferComponent {
//         let size = size_for_component_type(&kind);
//         let count = count_for_component_type(&kind);
//         BufferComponent {
//             name: name,
//             kind: kind,
//             size: size,
//             count: count,
//             offset: 0,
//             normalized: normalized,
//         }
//     }

//     /// Returns a reference to this `BufferComponent`'s total size in bytes
//     pub fn size(&self) -> &usize {
//         &self.size
//     }

//     /// Returns a reference to the type/amount of data within this `BufferComponent`, represented as a `BufferComponentType`
//     pub fn kind(&self) -> &BufferComponentType {
//         &self.kind
//     }

//     /// Returns a reference to the number of data points in this `BufferComponent`
//     pub fn count(&self) -> &usize {
//         &self.count
//     }

//     /// Returns a reference to whether or not the data within this `BufferComponent` is normalized to a 0.0 - 1.0 numerical range
//     pub fn normalized(&self) -> &bool {
//         &self.normalized
//     }

//     /// Returns a reference to the offset index within this `BufferComponent`'s owner `BufferLayout` (0 if this is not yet a part of a `BufferLayout`)
//     pub fn offset(&self) -> &usize {
//         &self.offset
//     }

//     /// Sets the offset index of this `BufferComponent` in the context of an owning `BufferLayout`
//     pub fn set_offset(&mut self, offset: usize) {
//         self.offset = offset
//     }
// }

// /// The layout of data within a `VertexBuffer`, as it relates to Vertex Attributes in OpenGL
// pub struct BufferLayout {
//     components: Vec<BufferComponent>,
//     stride: usize,
// }

// impl BufferLayout {
//     /// Creates a new `BufferLayout` to be used to construct a `VertexBuffer`
//     ///
//     /// ### Parameters
//     ///
//     /// - `components`: The constituent `BufferComponent`s which make up this `BufferLayout`
//     ///
//     /// ### Returns
//     ///
//     /// A new `BufferLayout`, ready for insertion into a `VertexBuffer`
//     pub fn new(components: &mut [BufferComponent]) -> BufferLayout {
//         let mut stride = 0;
//         let mut offset = 0;
//         for component in components.iter_mut() {
//             component.set_offset(offset);
//             offset += component.size();
//             stride += component.size();
//         }
//         BufferLayout {
//             components: components.to_vec(),
//             stride: stride,
//         }
//     }

//     /// Returns a reference to this `BufferLayout`'s stride (distance in bytes from one internal `BufferComponent` to the next)
//     pub fn stride(&self) -> &usize {
//         &self.stride
//     }

//     /// Returns a reference to the internal `BufferComponents` of this `BufferLayout`
//     pub fn components(&self) -> &Vec<BufferComponent> {
//         &self.components
//     }
// }
