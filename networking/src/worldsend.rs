use crate::authent::{Client, ClientGameState};
use crate::packets::{ClientReliablePacket, ServerReliablePacket, WorldDataFragment};
use crate::{decode, encode, AuthentID, Frame, MAX_WORLDSEND_PACKET_SIZE};
use common::FastMap;
use message_io::network::{Endpoint, Network};
use serde::de::DeserializeOwned;

#[derive(Eq, PartialEq)]
enum WorldSendStatus {
    ReadyToSend,
    WaitingForFinalAck,
    Over,
}

struct WorldSendState {
    data: Vec<u8>,
    sent: usize,
    status: WorldSendStatus,
    frame: Frame,
}

#[derive(Default)]
pub(crate) struct WorldSend {
    send_state: FastMap<AuthentID, WorldSendState>,
}

impl WorldSend {
    pub fn begin_send(&mut self, c: &Client, data: Vec<u8>, frame: Frame) {
        self.send_state.insert(
            c.id,
            WorldSendState {
                data,
                sent: 0,
                status: WorldSendStatus::ReadyToSend,
                frame,
            },
        );
    }

    pub fn ack(&mut self, c: &Client) {
        if let Some(state) = self.send_state.get_mut(&c.id) {
            if state.status == WorldSendStatus::WaitingForFinalAck {
                state.status = WorldSendStatus::Over
            }
        } else {
            log::error!("ack ing a non existing world send");
        }
    }

    pub fn update(&mut self, c: &mut Client, net: &mut Network) {
        if let Some(state) = self.send_state.get_mut(&c.id) {
            if state.status == WorldSendStatus::Over {
                self.send_state.remove(&c.id);
                c.state = ClientGameState::CatchingUp;
                return;
            }
            if state.status != WorldSendStatus::ReadyToSend {
                return;
            }

            let to_send = MAX_WORLDSEND_PACKET_SIZE.min(state.data.len() - state.sent);
            let is_over = (to_send < MAX_WORLDSEND_PACKET_SIZE).then(|| state.frame);

            net.send(
                c.reliable,
                &*encode(&ServerReliablePacket::WorldSend(WorldDataFragment {
                    is_over,
                    data: Vec::from(&state.data[state.sent..state.sent + to_send]),
                })),
            );

            if is_over.is_some() {
                log::info!("{}: sending final world fragment", c.name);
                state.status = WorldSendStatus::WaitingForFinalAck;
            } else {
                log::info!("{}: sending world fragment", c.name);
            }

            state.sent += to_send;
        } else {
            log::error!("updating a non existing world send");
        }
    }

    pub fn disconnected(&mut self, id: AuthentID) {
        self.send_state.remove(&id);
    }
}

#[derive(Debug)]
pub(crate) enum WorldReceive<W> {
    Downloading { data_so_far: Vec<u8> },
    Finished { frame: Frame, world: W },
    Errored,
}

impl<W> Default for WorldReceive<W> {
    fn default() -> Self {
        Self::Downloading {
            data_so_far: vec![],
        }
    }
}

impl<W: DeserializeOwned> WorldReceive<W> {
    pub fn handle(&mut self, fragment: WorldDataFragment, net: &mut Network, tcp: Endpoint) {
        if let WorldReceive::Downloading {
            ref mut data_so_far,
        } = self
        {
            data_so_far.extend(fragment.data);
            if let Some(frame) = fragment.is_over {
                log::info!("received last fragment at {:?}", frame);
                net.send(tcp, &*encode(&ClientReliablePacket::WorldAck));

                let d = decode(data_so_far);

                if let Some(w) = d {
                    *self = WorldReceive::Finished { frame, world: w }
                } else {
                    *self = WorldReceive::Errored;
                }
            }
        } else {
            log::warn!(
                "received fragment but was not downloading (errored: {:?})",
                matches!(self, WorldReceive::Errored)
            );
        }
    }
}
