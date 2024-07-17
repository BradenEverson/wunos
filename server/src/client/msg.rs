use fastwebsockets::{Frame, Payload};

pub struct DynMessage {

}

impl Into<Frame> for DynMessage {
    fn into(self) -> Frame {
        Frame::text(Payload::Owned("Hello".as_bytes().to_vec()))
    }
}
