// Reference implementation, thanks to the author:
// https://github.com/greshake/i3status-rust/blob/master/src/blocks/focused_window/wlr_toplevel_management.rs

use std::collections::HashMap;

use miette::{Context, IntoDiagnostic, Result};
use wayrs_client::{Connection, EventCtx};
use wayrs_protocols::wlr_foreign_toplevel_management_unstable_v1::{ZwlrForeignToplevelHandleV1, ZwlrForeignToplevelManagerV1, zwlr_foreign_toplevel_handle_v1, zwlr_foreign_toplevel_manager_v1};

#[derive(Default)]
struct ToplevelState {
    toplevels: HashMap<ZwlrForeignToplevelHandleV1, ToplevelInfo>,
    active_toplevel: Option<ZwlrForeignToplevelHandleV1>,
}

#[derive(Default, Clone)]
pub struct ToplevelInfo {
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub is_focused: bool,
}

fn handle_manager_events(ctx: EventCtx<ToplevelState, ZwlrForeignToplevelManagerV1>) {
    use zwlr_foreign_toplevel_manager_v1::Event::*;
    match ctx.event {
        Toplevel(handle) => {
            ctx.state.toplevels.insert(handle, ToplevelInfo::default());
            ctx.conn.set_callback_for(handle, handle_toplevel_events);
        }
        Finished => {
            tracing::error!("Unexpected 'finished' event");
            ctx.conn.break_dispatch_loop();
        }
        _ => {}
    }
}

fn handle_toplevel_events(ctx: EventCtx<ToplevelState, ZwlrForeignToplevelHandleV1>) {
    use zwlr_foreign_toplevel_handle_v1::Event::*;

    let Some(toplevel) = ctx.state.toplevels.get_mut(&ctx.proxy) else {
        return;
    };

    match ctx.event {
        Title(title) => {
            toplevel.title = Some(String::from_utf8_lossy(title.as_bytes()).into_owned());
        }
        AppId(app_id) => {
            toplevel.app_id = Some(String::from_utf8_lossy(app_id.as_bytes()).into_owned())
        }
        // State flags are given as an array of packed u32 values. We loop over these
        // values (by grouping 4 u8 values) and try to find the Activated state (2).
        State(state_bytes) => {
            toplevel.is_focused = state_bytes
                .chunks_exact(4)
                .map(|b| u32::from_ne_bytes(b.try_into().expect("slice somehow not 4 bytes long")))
                .any(|s| s == zwlr_foreign_toplevel_handle_v1::State::Activated as u32);
        }
        Closed => {
            if ctx.state.active_toplevel == Some(ctx.proxy) {
                ctx.state.active_toplevel = None;
            }

            ctx.proxy.destroy(ctx.conn);
            ctx.state.toplevels.remove(&ctx.proxy);
        }
        Done => {
            if toplevel.is_focused {
                ctx.state.active_toplevel = Some(ctx.proxy);
            } else if ctx.state.active_toplevel == Some(ctx.proxy) {
                ctx.state.active_toplevel = None;
            }
        }
        _ => {}
    }
}

fn get_toplevels() -> Result<ToplevelState> {
    let mut conn = Connection::<ToplevelState>::connect().unwrap();
    // Setup globals
    conn.blocking_roundtrip()
        .into_diagnostic()
        .context("first round trip failed")?;

    let _manager: ZwlrForeignToplevelManagerV1 = conn
        .bind_singleton_with_cb(1..=3, handle_manager_events)
        .into_diagnostic()
        .context("compositor does not support wlr-foreign-toplevel-management-unstable-v1")
        .unwrap();

    let mut state = ToplevelState::default();

    conn.blocking_roundtrip()
        .into_diagnostic()
        .context("second round trip failed")?;
    conn.dispatch_events(&mut state);

    Ok(state)
}

pub fn get_active_toplevel() -> Result<Option<ToplevelInfo>> {
    let mut state = get_toplevels()?;
    let active_toplevel = state
        .active_toplevel
        .and_then(|k| state.toplevels.remove_entry(&k).map(|(_, v)| v));
    Ok(active_toplevel)
}
