use crate::{
    components::{action_dropdown, alert_indicator, lock_indicator, resource_links, table, Placement},
    extensions::MergeAttrs,
    extract_id,
    generated::css_classes::C,
    route::RouteId,
    GMsg, Route,
};
use iml_wire_types::{
    db::DeviceRecord,
    warp_drive::{ArcCache, ArcValuesExt, Locks},
    Session, TargetKind, ToCompositeId,
};
use seed::{prelude::*, *};
use std::{collections::HashMap, sync::Arc};

pub struct Row {
    dropdown: action_dropdown::Model,
}

#[derive(Default)]
pub struct Model {
    pub rows: HashMap<u32, Row>,
    pub devices: Vec<Arc<DeviceRecord>>,
}

#[derive(Clone)]
pub enum Msg {
    SetDevices(Vec<Arc<DeviceRecord>>),
}

pub fn init(cache: &ArcCache, orders: &mut impl Orders<Msg, GMsg>) {
    orders.send_msg(Msg::SetDevices(cache.device.values().cloned().collect()));
}

pub fn update(msg: Msg, cache: &ArcCache, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::SetDevices(xs) => {
            model.rows = xs
                .iter()
                .map(|x| {
                    (
                        x.record_id,
                        Row {
                            dropdown: action_dropdown::Model::new(vec![x.composite_id()]),
                        },
                    )
                })
                .collect();

            let mut devices: Vec<_> = xs;

            devices.sort_by(|a, b| natord::compare(&a.device.id, &b.device.id));

            model.devices = devices;
        }
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    div![
        class![C.bg_white],
        div![
            class![C.px_6, C.bg_gray_200],
            h3![class![C.py_4, C.font_normal, C.text_lg], "devices"]
        ],
        if model.devices.is_empty() {
            div![
                class![C.text_3xl, C.text_center],
                h1![class![C.m_2, C.text_gray_600], "No devices found"],
            ]
        } else {
            table::wrapper_view(vec![
                table::thead_view(vec![
                    table::th_view(plain!["Name"]),
                    table::th_view(plain!["Filesystems"]),
                    table::th_view(plain!["Volume"]),
                    table::th_view(plain!["Primary Server"]),
                    table::th_view(plain!["Failover Server"]),
                    table::th_view(plain!["Started on"]),
                    th![],
                ]),
                tbody![model.devices.iter().map(|x| match model.rows.get(&x.record_id) {
                    None => empty![],
                    Some(row) => {
                        let fs = cache.filesystem.arc_values().filter(|y| {
                            extract_id(&y.mgt)
                                .and_then(|y| y.parse::<u32>().ok())
                                .filter(|y| y == &x.record_id)
                                .is_some()
                        });

                        tr![table::td_center(vec![a![
                            class![C.text_blue_500, C.hover__underline],
                            attrs! {At::Href => Route::Device(RouteId::from(x.record_id)).to_href()},
                            &x.device.id
                        ],]),]
                    }
                })],
            ])
        }
    ]
}
