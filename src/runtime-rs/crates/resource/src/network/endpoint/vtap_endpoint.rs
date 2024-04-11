// Copyright (c) 2019-2022 Alibaba Cloud
// Copyright (c) 2019-2022 Ant Group
//
// SPDX-License-Identifier: Apache-2.0
//

// use std::io::{self, Error};
use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use hypervisor::device::device_manager::{do_handle_device, DeviceManager};
use hypervisor::device::driver::NetworkConfig;
use hypervisor::device::{DeviceConfig, DeviceType};
use hypervisor::{Hypervisor, NetworkDevice};
use tokio::sync::RwLock;

use super::endpoint_persist::{EndpointState, VtapEndpointState};
use super::Endpoint;
use crate::network::utils;

#[derive(Debug)]
pub struct VtapEndpoint {
    // Name of virt interface
    name: String,
    // Hardware address of virt interface
    guest_mac: String,
    // Vtap interface on the host
    tap_iface: NetworkInterface,
    // Device manager
    dev_mgr: Arc<RwLock<DeviceManager>>,
    // Virtio queue num
    queue_num: usize,
    // Virtio queue size
    queue_size: usize,
}

impl VtapEndpoint {
    pub async fn new(
        dev_mgr: &Arc<RwLock<DeviceManager>>,
        handle: &rtnetlink::Handle,
        iface_name: &str,
        hardware_addr: &[u8],
        queues: usize,
    ) -> Result<Self> {
        Ok(VtapEndpoint {
            name: iface_name.to_string(),
            guest_mac: utils::get_mac_addr(hardware_addr).context("get mac addr")?,
            tap_iface: NetworkInterface {
                name: name.clone(),
                hard_addr: utils::get_mac_addr(hardware_addr).context("get mac addr")?,
            },
            dev_mgr: dev_mgr.clone(),

        })
    }

    fn get_network_config(&self) -> Result<NetworkConfig> {
        let guest_mac = utils::parse_mac(&self.guest_mac).context("Parse mac address")?;
        Ok(NetworkConfig {
            host_dev_name: self.name.clone(),
            guest_mac: Some(guest_mac),
            ..Default::default()
        })
    }
}

#[async_trait]
impl Endpoint for VtapEndpoint {
    async fn name(&self) -> String {
        self.name.clone()
    }

    async fn hardware_addr(&self) -> String {
        self.guest_mac.clone()
    }

    async fn attach(&self) -> Result<()> {
        let config = self.get_network_config().context("Get network config")?;
        do_handle_device(&self.d, &DeviceConfig::NetworkCfg(config))
            .await
            .context("Handle device")?;
        Ok(())
    }

    async fn detach(&self, h: &dyn Hypervisor) -> Result<()> {
        let config = self.get_network_config().context("Get network config")?;
        h.remove_device(DeviceType::Network(NetworkDevice {
            config,
            ..Default::default()
        }))
        .await
        .context("Remove device")?;
        Ok(())
    }

    async fn save(&self) -> Option<EndpointState> {
        Some(EndpointState {
            vtap_endpoint: Some(VtapEndpointState {
                if_name: self.name.clone(),
            }),
            ..Default::default()
        })
    }
}
