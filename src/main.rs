/*
 * Copyright (c) 2023 42dot All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use futures_util::stream::StreamExt;
use log::LevelFilter;
use zbus::{dbus_proxy, zvariant::ObjectPath, zvariant::OwnedObjectPath, Connection, Result};

#[dbus_proxy(
    default_service = "org.containers.hirte",
    interface = "org.containers.hirte.Manager",
    default_path = "/org/containers/hirte"
)]
trait HirteManager {
    fn get_node(&self, name: &str) -> Result<OwnedObjectPath>;
    fn list_nodes(&self) -> Result<Vec<(String, OwnedObjectPath, String)>>;
}

#[dbus_proxy(
    default_service = "org.containers.hirte",
    interface = "org.containers.hirte.Node"
)]
trait HirteNode {
    #[dbus_proxy(object = "HirteJob")]
    fn start_unit(&self, name: &str, mode: &str);

    #[dbus_proxy(object = "HirteJob")]
    fn stop_unit(&self, name: &str, mode: &str);

    #[dbus_proxy(property)]
    fn name(&self) -> Result<String>;

    #[dbus_proxy(property)]
    fn status(&self) -> Result<String>;
}

#[dbus_proxy(
    default_service = "org.containers.hirte",
    interface = "org.containers.hirte.Job"
)]
trait HirteJob {
    fn cancel(&self) -> Result<()>;

    #[dbus_proxy(property)]
    fn id(&self) -> Result<u32>;

    #[dbus_proxy(property)]
    fn node(&self) -> Result<String>;

    #[dbus_proxy(property)]
    fn unit(&self) -> Result<String>;

    #[dbus_proxy(property)]
    fn job_type(&self) -> Result<String>;

    #[dbus_proxy(property)]
    fn state(&self) -> Result<String>;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::new()
        .target(env_logger::Target::Stdout)
        .format_timestamp_millis()
        .filter_level(LevelFilter::Debug)
        .init();

    let conn = Connection::system().await?;

    let hirte_manager = HirteManagerProxy::new(&conn).await?;
    let hnode = hirte_manager.get_node("ak7_master_sub").await?;
    let hnodes = hirte_manager.list_nodes().await?;
    log::debug!("HirteManager: GetNode:[{}]", hnode.as_str());
    for (name, path, stat) in hnodes.iter() {
        log::debug!(
            "HirteManager: ListNodes:[{}|{}|{}]",
            name.as_str(),
            path.as_str(),
            stat.as_str()
        );
    }

    let hirte_node = HirteNodeProxy::builder(&conn)
        .path(hnode.into_inner())?
        .build()
        .await?;

    let node_name = hirte_node.name().await?;
    let node_stat = hirte_node.status().await?;
    log::debug!("HirteNode:[Name-{}|Status-{}]", node_name, node_stat);

    let j1 = hirte_node.start_unit("v2x.service", "replace").await;
    if j1.is_err() {
        log::debug!("HirteNode: StartUnit Failed:[{:?}]", j1.err().unwrap());
    } else {
        let job = j1.ok().unwrap();
        log::debug!("HirteNode: StartUnit Ok:[{:?}]", job);

        // cancel is not supported currently
        let ret = job.cancel().await;
        if ret.is_err() {
            log::debug!("HirteJob: Cancel Failed:[{:?}]", ret.err().unwrap());
        } else {
            log::debug!("HirteJob: Cancel Ok");
        }
    }

    let j2 = hirte_node.stop_unit("v2x.service", "replace").await;
    if j2.is_err() {
        log::debug!("HirteNode: StopUnit Failed:[{:?}]", j2.err().unwrap());
    } else {
        let job = j2.ok().unwrap();
        log::debug!("HirteNode: StopUnit Ok:[{:?}]", job);
    }

    Ok(())
}
