// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::type_complexity)]

use riker::actors::*;

use std::path::PathBuf;

use engine::{
    snapshot,
    vault::{ClientId, DbView, Key, VaultId},
};

use stronghold_utils::GuardDebug;

use crate::{
    actors::{InternalMsg, SHResults},
    line_error,
    state::{
        client::Store,
        snapshot::{Snapshot, SnapshotState},
    },
    utils::StatusMessage,
    Provider,
};

use std::collections::HashMap;

/// Messages used for the Snapshot Actor.
#[derive(Clone, GuardDebug)]
pub enum SMsg {
    /// Write the snapshot to the file.
    WriteSnapshot {
        key: snapshot::Key,
        filename: Option<String>,
        path: Option<PathBuf>,
    },
    /// Fill the snapshot structure with data.
    FillSnapshot {
        data: Box<(HashMap<VaultId, Key<Provider>>, DbView<Provider>, Store)>,
        id: ClientId,
    },
    /// Reead from the snapshot.
    ReadFromSnapshot {
        key: snapshot::Key,
        filename: Option<String>,
        path: Option<PathBuf>,
        id: ClientId,
        fid: Option<ClientId>,
    },
    SynchronizeSnapshot {
        id: ClientId,
        key: snapshot::Key,
        f_other: Option<String>,
        p_other: Option<PathBuf>,
        p_target: PathBuf,
        k_target: snapshot::Key,
    },
}

/// Actor Factory for the Snapshot.
impl ActorFactory for Snapshot {
    fn create() -> Self {
        Snapshot::new(SnapshotState::default())
    }
}

impl Actor for Snapshot {
    type Msg = SMsg;

    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
        self.receive(ctx, msg, sender);
    }
}

impl Receive<SMsg> for Snapshot {
    type Msg = SMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
        match msg {
            SMsg::FillSnapshot { data, id } => {
                self.state.add_data(id, *data);

                sender
                    .as_ref()
                    .expect(line_error!())
                    .try_tell(SHResults::ReturnFillSnap(StatusMessage::OK), None)
                    .expect(line_error!());
            }
            SMsg::ReadFromSnapshot {
                key,
                filename,
                path,
                id,
                fid,
            } => {
                let id_str: String = id.into();
                let internal = ctx.select(&format!("/user/internal-{}/", id_str)).expect(line_error!());
                let cid = if let Some(fid) = fid { fid } else { id };

                if self.has_data(cid) {
                    let data = self.get_state(cid);

                    internal.try_tell(
                        InternalMsg::ReloadData {
                            id: cid,
                            data: Box::new(data),
                            status: StatusMessage::OK,
                        },
                        sender,
                    );
                } else {
                    match Snapshot::read_from_snapshot(filename.as_deref(), path.as_deref(), key) {
                        Ok(mut snapshot) => {
                            let data = snapshot.get_state(cid);

                            *self = snapshot;

                            internal.try_tell(
                                InternalMsg::ReloadData {
                                    id: cid,
                                    data: Box::new(data),
                                    status: StatusMessage::OK,
                                },
                                sender,
                            );
                        }
                        Err(e) => {
                            sender
                                .as_ref()
                                .expect(line_error!())
                                .try_tell(
                                    SHResults::ReturnReadSnap(StatusMessage::Error(format!(
                                        "{}, Unable to read snapshot. Please try another password.",
                                        e
                                    ))),
                                    None,
                                )
                                .expect(line_error!());
                        }
                    }
                };
            }
            SMsg::WriteSnapshot { key, filename, path } => {
                self.write_to_snapshot(filename.as_deref(), path.as_deref(), key)
                    .expect(line_error!());

                self.state = SnapshotState::default();

                sender
                    .as_ref()
                    .expect(line_error!())
                    .try_tell(SHResults::ReturnWriteSnap(StatusMessage::OK), None)
                    .expect(line_error!());
            }
            SMsg::SynchronizeSnapshot {
                id,
                key,
                f_other,
                p_other,
                p_target,
                k_target,
            } => {
                if !self.has_data(id) {
                    println!("No data present");
                }

                self.synchronize(p_other.as_deref(), f_other.as_deref(), key, p_target, k_target)
                    .expect(line_error!());

                sender
                    .as_ref()
                    .expect("Could not get sender")
                    .try_tell(SHResults::ReturnSynchronizeSnapshot(StatusMessage::Ok(())), None)
                    .expect("Could not tell sender result");
            }
        }
    }
}
