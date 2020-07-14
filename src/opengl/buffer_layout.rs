use gl::{
    self,
    types::{GLboolean, GLfloat, GLuint},
};
use std::mem::size_of;

#[derive(Clone)]
pub enum BufferComponentType {
    Float,
    Float2,
    Float3,
    Float4,
    Int,
    Int2,
    Int3,
    Int4,
    Mat3,
    Mat4,
    Bool,
}

fn size_for_component_type(kind: &BufferComponentType) -> usize {
    match kind {
        BufferComponentType::Float => size_of::<GLfloat>(),
        BufferComponentType::Float2 => 2 * size_of::<GLfloat>(),
        BufferComponentType::Float3 => 3 * size_of::<GLfloat>(),
        BufferComponentType::Float4 => 4 * size_of::<GLfloat>(),
        BufferComponentType::Int => size_of::<GLuint>(),
        BufferComponentType::Int2 => 2 * size_of::<GLuint>(),
        BufferComponentType::Int3 => 3 * size_of::<GLuint>(),
        BufferComponentType::Int4 => 4 * size_of::<GLuint>(),
        BufferComponentType::Mat3 => 3 * 3 * size_of::<GLfloat>(),
        BufferComponentType::Mat4 => 4 * 4 * size_of::<GLfloat>(),
        BufferComponentType::Bool => size_of::<GLboolean>(),
    }
}

fn count_for_component_type(kind: &BufferComponentType) -> usize {
    match kind {
        BufferComponentType::Float => 1,
        BufferComponentType::Float2 => 2,
        BufferComponentType::Float3 => 3,
        BufferComponentType::Float4 => 4,
        BufferComponentType::Int => 1,
        BufferComponentType::Int2 => 2,
        BufferComponentType::Int3 => 3,
        BufferComponentType::Int4 => 4,
        BufferComponentType::Mat3 => 3, // 3 * Float3
        BufferComponentType::Mat4 => 4, // 4 * Float4
        BufferComponentType::Bool => 1,
    }
}

#[derive(Clone)]
pub struct BufferComponent {
    name: String,
    kind: BufferComponentType,
    size: usize,
    count: usize,
    offset: usize,
    normalized: bool,
}

impl BufferComponent {
    pub fn new(name: String, kind: BufferComponentType, normalized: bool) -> Self {
        let size = size_for_component_type(&kind);
        let count = count_for_component_type(&kind);
        BufferComponent {
            name: name,
            kind: kind,
            size: size,
            count: count,
            offset: 0,
            normalized: normalized,
        }
    }

    pub fn size(&self) -> &usize {
        &self.size
    }

    pub fn kind(&self) -> &BufferComponentType {
        &self.kind
    }

    pub fn count(&self) -> &usize {
        &self.count
    }

    pub fn normalized(&self) -> &bool {
        &self.normalized
    }

    pub fn offset(&self) -> &usize {
        &self.offset
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset
    }
}

pub struct BufferLayout {
    components: Vec<BufferComponent>,
    stride: usize,
}

impl BufferLayout {
    pub fn new(components: &mut [BufferComponent]) -> Self {
        let mut stride = 0;
        let mut offset = 0;
        for component in components.iter_mut() {
            component.set_offset(offset);
            offset += component.size();
            stride += component.size();
        }
        BufferLayout {
            components: components.to_vec(),
            stride: stride,
        }
    }

    pub fn stride(&self) -> &usize {
        &self.stride
    }

    pub fn components(&self) -> &Vec<BufferComponent> {
        &self.components
    }
}
