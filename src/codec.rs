use crate::packets::{
    Agreement, Confirmation, ControlPacket, FileInfo, FileTransmission, Initiation, Packet,
    Response,
};

pub trait Codec {
    fn encode(&self) -> Packet;
    fn decode(packet: Packet) -> Self;
}

impl Codec for Initiation {
    fn encode(&self) -> Packet {
        Packet::Control(ControlPacket::Initiation)
    }

    fn decode(packet: Packet) -> Self {
        Self {}
    }
}

impl Codec for Response {
    fn encode(&self) -> Packet {
        Packet::Data(
            bincode::serialize(self).expect("Codec Failure while serializing Response Packet"),
        )
    }

    fn decode(packet: Packet) -> Self {
        //bincode::deserialize(packet::Data(e))
        todo!()
    }
}

impl Codec for Agreement {
    fn encode(&self) -> Packet {
        Packet::Control(ControlPacket::Agreement)
    }

    fn decode(packet: Packet) -> Self {
        Self {}
    }
}

impl Codec for FileInfo {
    fn encode(&self) -> Packet {
        Packet::Data(
            bincode::serialize(self).expect("Codec Failure while serializing FileInfo Packet"),
        )
    }

    fn decode(packet: Packet) -> Self {
        todo!()
    }
}

impl Codec for Confirmation {
    fn encode(&self) -> Packet {
        Packet::Data(
            bincode::serialize(self).expect("Codec Failure while serializing Confirmation Packet"),
        )
    }

    fn decode(packet: Packet) -> Self {
        todo!()
    }
}

impl Codec for FileTransmission {
    fn encode(&self) -> Packet {
        Packet::Data(
            bincode::serialize(self)
                .expect("Codec Failure while serializing FileTransmission Packet"),
        )
    }

    fn decode(packet: Packet) -> Self {
        todo!()
    }
}
