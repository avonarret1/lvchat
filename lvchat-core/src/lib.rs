#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum Source {
    Server,
    Client,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Message<'a> {
    pub source: Source,
    pub kind: MessageKind,
    pub time: std::time::Instant,
    pub data: &'a [u8],
}

#[non_exhaustive]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum MessageKind {
    Join,
    Leave,

    Text,
    Voice,
}


unsafe impl bytemuck::Zeroable for Message<'static> {}
unsafe impl bytemuck::Pod for Message<'static> {}

