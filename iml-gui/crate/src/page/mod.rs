pub mod about;
pub mod activity;
pub mod dashboard;
pub mod filesystem;
pub mod filesystems;
pub mod fs_dashboard;
pub mod jobstats;
pub mod login;
pub mod logs;
pub mod mgts;
pub mod not_found;
pub mod ostpool;
pub mod ostpools;
pub mod partial;
pub mod power_control;
pub mod server;
pub mod server_dashboard;
pub mod servers;
pub mod target;
pub mod target_dashboard;
pub mod targets;
pub mod user;
pub mod users;
pub mod volume;
pub mod volumes;

use crate::{
    components::stratagem,
    route::{Route, RouteId},
    GMsg,
};
use iml_wire_types::warp_drive::{ArcCache, ArcRecord, RecordId};
use seed::prelude::Orders;
use std::sync::Arc;

/// Handles Cache changes on a per-page level.
pub trait RecordChange<Msg: 'static> {
    /// Called when a record is either added or updated
    fn update_record(&mut self, record: ArcRecord, cache: &ArcCache, orders: &mut impl Orders<Msg, GMsg>);
    /// Called when a record is removed
    fn remove_record(&mut self, id: RecordId, cache: &ArcCache, orders: &mut impl Orders<Msg, GMsg>);
    /// Called on initial load of all records, or on page initialization
    fn set_records(&mut self, cache: &ArcCache, orders: &mut impl Orders<Msg, GMsg>);
}

pub(crate) enum Page {
    About,
    AppLoading,
    Filesystems(filesystems::Model),
    Filesystem(Box<filesystem::Model>),
    Dashboard(dashboard::Model),
    FsDashboard(fs_dashboard::Model),
    ServerDashboard(server_dashboard::Model),
    TargetDashboard(target_dashboard::Model),
    Jobstats,
    Login(login::Model),
    Mgts(mgts::Model),
    NotFound,
    OstPools,
    OstPool(ostpool::Model),
    PowerControl,
    Servers(servers::Model),
    Server(Box<server::Model>),
    Targets,
    Target(Box<target::Model>),
    Users,
    User(user::Model),
    Volumes(volumes::Model),
    Volume(volume::Model),
}

impl RecordChange<Msg> for Page {
    fn update_record(&mut self, record: ArcRecord, cache: &ArcCache, orders: &mut impl Orders<Msg, GMsg>) {
        if let Self::Volumes(x) = self {
            x.update_record(record, cache, &mut orders.proxy(Msg::Volumes));
        }
    }
    fn remove_record(&mut self, id: RecordId, cache: &ArcCache, orders: &mut impl Orders<Msg, GMsg>) {
        if let Self::Volumes(x) = self {
            x.remove_record(id, cache, &mut orders.proxy(Msg::Volumes));
        }
    }
    fn set_records(&mut self, cache: &ArcCache, orders: &mut impl Orders<Msg, GMsg>) {
        if let Self::Volumes(x) = self {
            x.set_records(cache, &mut orders.proxy(Msg::Volumes));
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::NotFound
    }
}

impl<'a> From<(&ArcCache, &Route<'a>)> for Page {
    fn from((cache, route): (&ArcCache, &Route<'a>)) -> Self {
        match route {
            Route::About => Self::About,
            Route::Filesystems => Self::Filesystems(filesystems::Model::default()),
            Route::Filesystem(id) => id
                .parse()
                .ok()
                .and_then(|x| cache.filesystem.get(&x))
                .map(|x| {
                    Self::Filesystem(Box::new(filesystem::Model {
                        fs: Arc::clone(x),
                        mdts: Default::default(),
                        mdt_paging: Default::default(),
                        mgt: Default::default(),
                        osts: Default::default(),
                        ost_paging: Default::default(),
                        rows: Default::default(),
                        stratagem: stratagem::Model::new(Arc::clone(x)),
                    }))
                })
                .unwrap_or_default(),
            Route::Dashboard => Self::Dashboard(dashboard::Model {
                ..dashboard::Model::default()
            }),
            Route::FsDashboard(id) => Self::FsDashboard(fs_dashboard::Model {
                fs_name: id.to_string(),
                ..fs_dashboard::Model::default()
            }),
            Route::ServerDashboard(id) => Self::ServerDashboard(server_dashboard::Model {
                host_name: id.to_string(),
            }),
            Route::TargetDashboard(id) => Self::TargetDashboard(target_dashboard::Model {
                target_name: id.to_string(),
            }),
            Route::Jobstats => Self::Jobstats,
            Route::Login => Self::Login(login::Model::default()),
            Route::Mgt => Self::Mgts(mgts::Model::default()),
            Route::NotFound => Self::NotFound,
            Route::OstPools => Self::OstPools,
            Route::OstPool(id) => id
                .parse()
                .map(|id| Self::OstPool(ostpool::Model { id }))
                .unwrap_or_default(),
            Route::PowerControl => Self::PowerControl,
            Route::Servers => Self::Servers(servers::Model::default()),
            Route::Server(id) => id
                .parse()
                .ok()
                .and_then(|x| cache.host.get(&x))
                .map(|x| {
                    Self::Server(Box::new(server::Model::new(
                        Arc::clone(x),
                        x.lnet_id().and_then(|x| cache.lnet_configuration.get(&x).cloned()),
                        x.pacemaker_id()
                            .and_then(|x| cache.pacemaker_configuration.get(&x).cloned()),
                        x.corosync_id()
                            .and_then(|x| cache.corosync_configuration.get(&x).cloned()),
                    )))
                })
                .unwrap_or_default(),
            Route::Targets => Self::Targets,
            Route::Target(id) => id
                .parse()
                .ok()
                .and_then(|x| cache.target.get(&x))
                .map(|x| Self::Target(Box::new(target::Model::new(Arc::clone(x)))))
                .unwrap_or_default(),
            Route::Users => Self::Users,
            Route::User(id) => id
                .parse()
                .ok()
                .and_then(|x| cache.user.get(&x))
                .map(|x| Self::User(user::Model::new(Arc::clone(x))))
                .unwrap_or_default(),
            Route::Volumes => Self::Volumes(volumes::Model::default()),
            Route::Volume(id) => id
                .parse()
                .map(|id| Self::Volume(volume::Model { id }))
                .unwrap_or_default(),
        }
    }
}

impl Page {
    /// Is the given `Route` equivalent to the current page?
    pub fn is_active<'a>(&self, route: &Route<'a>) -> bool {
        match (route, self) {
            (Route::About, Self::About)
            | (Route::Filesystems, Self::Filesystems(_))
            | (Route::Dashboard, Self::Dashboard(dashboard::Model { .. }))
            | (Route::Jobstats, Self::Jobstats)
            | (Route::Login, Self::Login(_))
            | (Route::Mgt, Self::Mgts(_))
            | (Route::NotFound, Self::NotFound)
            | (Route::OstPools, Self::OstPools)
            | (Route::PowerControl, Self::PowerControl)
            | (Route::Servers, Self::Servers(_))
            | (Route::Targets, Self::Targets)
            | (Route::Users, Self::Users)
            | (Route::Volumes, Self::Volumes(_)) => true,
            (Route::OstPool(route_id), Self::OstPool(ostpool::Model { id }))
            | (Route::Volume(route_id), Self::Volume(volume::Model { id })) => route_id == &RouteId::from(id),
            (Route::Server(route_id), Self::Server(x)) => route_id == &RouteId::from(x.server.id),
            (Route::User(route_id), Self::User(x)) => route_id == &RouteId::from(x.user.id),
            (Route::Filesystem(route_id), Self::Filesystem(x)) => route_id == &RouteId::from(x.fs.id),
            (Route::Target(route_id), Self::Target(x)) => route_id == &RouteId::from(x.target.id),
            (Route::FsDashboard(route_id), Self::FsDashboard(fs_dashboard::Model { fs_name, .. })) => {
                &route_id.to_string() == fs_name
            }
            (Route::ServerDashboard(route_id), Self::ServerDashboard(server_dashboard::Model { host_name })) => {
                &route_id.to_string() == host_name
            }
            (Route::TargetDashboard(route_id), Self::TargetDashboard(target_dashboard::Model { target_name })) => {
                &route_id.to_string() == target_name
            }
            _ => false,
        }
    }
    /// Initialize the page. This gives a chance to initialize data when a page is switched to.
    pub fn init(&mut self, cache: &ArcCache, orders: &mut impl Orders<Msg, GMsg>) {
        match self {
            Self::Servers(_) => {
                servers::init(cache, &mut orders.proxy(Msg::Servers));
            }
            Self::Filesystems(_) => {
                filesystems::init(cache, &mut orders.proxy(Msg::Filesystems));
            }
            Self::Filesystem(_) => {
                filesystem::init(cache, &mut orders.proxy(Msg::Filesystem));
            }
            Self::Mgts(_) => {
                mgts::init(cache, &mut orders.proxy(Msg::Mgts));
            }
            Self::FsDashboard(_) => {
                fs_dashboard::init(&mut orders.proxy(Msg::FsDashboard));
            }
            Self::Dashboard(_) => {
                dashboard::init(&mut orders.proxy(Msg::Dashboard));
            }
            Self::Volumes(x) => {
                volumes::init(cache, x, &mut orders.proxy(Msg::Volumes));
            }
            _ => {}
        };
    }
}

#[derive(Clone)]
pub enum Msg {
    About(about::Msg),
    Dashboard(dashboard::Msg),
    Filesystem(filesystem::Msg),
    Filesystems(filesystems::Msg),
    FsDashboard(fs_dashboard::Msg),
    Mgts(mgts::Msg),
    Server(server::Msg),
    ServerDashboard(server_dashboard::Msg),
    Servers(servers::Msg),
    Target(target::Msg),
    Targets(targets::Msg),
    TargetDashboard(target_dashboard::Msg),
    Jobstats(jobstats::Msg),
    OstPools(ostpools::Msg),
    OstPool(ostpool::Msg),
    PowerControl(power_control::Msg),
    Login(Box<login::Msg>),
    User(user::Msg),
    Users(users::Msg),
    Volume(volume::Msg),
    Volumes(volumes::Msg),
}

pub(crate) fn update(msg: Msg, page: &mut Page, cache: &ArcCache, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Servers(msg) => {
            if let Page::Servers(page) = page {
                servers::update(msg, cache, page, &mut orders.proxy(Msg::Servers))
            }
        }
        Msg::Server(msg) => {
            if let Page::Server(page) = page {
                server::update(msg, cache, page, &mut orders.proxy(Msg::Server))
            }
        }
        Msg::Filesystem(msg) => {
            if let Page::Filesystem(page) = page {
                filesystem::update(msg, cache, page, &mut orders.proxy(Msg::Filesystem))
            }
        }
        Msg::Filesystems(msg) => {
            if let Page::Filesystems(page) = page {
                filesystems::update(msg, cache, page, &mut orders.proxy(Msg::Filesystems))
            }
        }
        Msg::Mgts(msg) => {
            if let Page::Mgts(page) = page {
                mgts::update(msg, cache, page, &mut orders.proxy(Msg::Mgts))
            }
        }
        Msg::Target(msg) => {
            if let Page::Target(page) = page {
                target::update(msg, cache, page, &mut orders.proxy(Msg::Target))
            }
        }
        Msg::Dashboard(msg) => {
            if let Page::Dashboard(page) = page {
                dashboard::update(msg, page, &mut orders.proxy(Msg::Dashboard))
            }
        }
        Msg::FsDashboard(msg) => {
            if let Page::FsDashboard(page) = page {
                fs_dashboard::update(msg, page, &mut orders.proxy(Msg::FsDashboard))
            }
        }
        Msg::User(msg) => {
            if let Page::User(page) = page {
                user::update(msg, page, &mut orders.proxy(Msg::User));
            }
        }
        Msg::Volumes(msg) => {
            if let Page::Volumes(page) = page {
                volumes::update(msg, page, &mut orders.proxy(Msg::Volumes))
            }
        }
        Msg::Login(msg) => {
            if let Page::Login(page) = page {
                login::update(*msg, page, &mut orders.proxy(|x| Msg::Login(Box::new(x))));
            }
        }
        Msg::About(_)
        | Msg::Jobstats(_)
        | Msg::OstPool(_)
        | Msg::OstPools(_)
        | Msg::PowerControl(_)
        | Msg::ServerDashboard(_)
        | Msg::TargetDashboard(_)
        | Msg::Targets(_)
        | Msg::Users(_)
        | Msg::Volume(_) => {}
    }
}
