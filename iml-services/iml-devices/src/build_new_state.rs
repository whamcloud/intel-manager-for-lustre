// Copyright (c) 2020 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    breadth_first_parent_iterator::BreadthFirstParentIterator,
    db::{DeviceHosts, Devices},
    merge_state::merge_state,
};
use iml_wire_types::{db::DeviceId, Fqdn};
use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};

pub fn build_new_state<'a>(
    fqdn: &Fqdn,
    incoming_devices: &Devices,
    incoming_device_hosts: &DeviceHosts,
    db_devices: &Devices,
    db_device_hosts: &DeviceHosts,
) -> (Devices, DeviceHosts) {
    let (devices, device_hosts) = (BTreeMap::new(), BTreeMap::new());

    let (temporary_devices, temporary_device_hosts) = merge_state(
        fqdn,
        incoming_devices,
        incoming_device_hosts,
        db_devices,
        db_device_hosts,
    );
    let local_virtual_device_hosts = temporary_device_hosts.iter().filter(|(_, dh)| dh.local);
    let non_local_virtual_device_hosts = temporary_device_hosts.iter().filter(|(_, dh)| !dh.local);

    let sorted_device_ids: Vec<_> = temporary_devices
        .iter()
        .sorted_by_key(|(_, d)| d.max_depth)
        .map(|(id, _)| id.clone())
        .collect();

    let new_device_hosts = temporary_device_hosts.clone();
    let new_devices = temporary_devices.clone();

    let other_fqdns: BTreeSet<_> = temporary_device_hosts
        .iter()
        .filter_map(|((_, f), _)| if f != fqdn { Some(f) } else { None })
        .collect();

    // FIXME: Sort local_virtual_device_hosts by max_depth!
    for (_, dh) in local_virtual_device_hosts {
        for f in other_fqdns.iter() {
            let all_available = are_all_parents_available(
                &temporary_devices,
                &temporary_device_hosts,
                Fqdn(f.0.clone()),
                &dh.device_id,
            );
        }
    }

    (devices, device_hosts)
}

fn are_all_parents_available(
    devices: &Devices,
    device_hosts: &DeviceHosts,
    host: Fqdn,
    child_id: &DeviceId,
) -> bool {
    let mut i = BreadthFirstParentIterator::new(devices, child_id);
    let all_available = i.all(|p| {
        let result = device_hosts.get(&(p.clone(), host.clone())).is_some();
        tracing::trace!("Checking device {:?} on host {:?}: {:?}", p, host, result);
        result
    });
    tracing::info!(
        "are_all_parents_available: host: {:?}, device: {:?}, all_available: {:?}",
        host,
        child_id,
        all_available
    );
    all_available
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::test::deser_fixture;
    use ::test_case::test_case;
    use insta::assert_debug_snapshot;

    #[test_case("vd_with_shared_parents_added_to_oss2")]
    #[test_case("vd_with_no_shared_parents_not_added_to_oss2")]
    #[test_case("vd_with_shared_parents_updated_on_oss2")]
    #[test_case("vd_with_shared_parents_replaced_on_oss2")]
    #[test_case("vd_with_shared_parents_removed_from_oss2_when_parent_disappears")]
    #[test_case("vd_with_two_levels_of_shared_parents_added_to_oss2")]
    #[test_case("vd_with_two_levels_of_shared_parents_removed_from_oss2_when_parent_disappears")]
    #[test_case("vd_with_two_levels_of_shared_parents_replaced_on_oss2")]
    #[test_case("vd_with_two_levels_of_shared_parents_updated_on_oss2")]
    #[test_case("vd_with_two_levels_of_shared_parents_in_reverse_order_added_to_oss2")]
    #[test_case("vd_with_three_levels_of_shared_parents_in_reverse_order_added_to_oss2")]
    #[test_case("vd_with_shared_parents_added_after_oss1_to_oss2")]
    fn build_new_state(test_name: &str) {
        crate::db::test::_init_subscriber();
        let (fqdn, incoming_devices, incoming_device_hosts, db_devices, db_device_hosts) =
            deser_fixture(test_name);

        for ((_, f), dh) in incoming_device_hosts.iter() {
            assert_eq!(&fqdn, f);
            assert_eq!(fqdn, dh.fqdn);
        }

        let changes = super::build_new_state(
            &fqdn,
            &incoming_devices,
            &incoming_device_hosts,
            &db_devices,
            &db_device_hosts,
        );

        assert_debug_snapshot!(test_name, changes);
    }
}
