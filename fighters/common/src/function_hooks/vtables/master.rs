use super::*;
use crate::function_hooks::attack::{OnHitContext, OnHit};
use utils::ext::*;

pub(crate) struct MasterOnHit;

impl OnHit for MasterOnHit {
    #[inline(always)]
    fn on_hit(ctx: &mut OnHitContext<'_>) {
        unsafe {
            if ctx.hitbox_id() != 0 {
                return;
            }

            let attacker_boma = ctx.attacker_boma();
            if !VarModule::is_flag(
                attacker_boma.object(),
                vars::master::status::SPECIAL_LW_GROUND_HITBOX,
            ) {
                return;
            }

            let receiver_boma = ctx.receiver_boma();
            if receiver_boma.is_status_one_of(&[
                *FIGHTER_STATUS_KIND_GUARD_ON,
                *FIGHTER_STATUS_KIND_GUARD,
                *FIGHTER_STATUS_KIND_GUARD_DAMAGE,
            ]) {
                ctx.set_hitbox_id(2);
            } else {
                ctx.set_hitbox_id(1);
            }
        }
    }
}

extern "C" {
    #[link_name = "master_link_event_inner"]
    fn master_link_event_inner(
        vtable: u64,
        fighter: &mut Fighter,
        event: &mut smash_rs::app::LinkEvent,
        original: extern "C" fn(u64, &mut Fighter, &mut smash_rs::app::LinkEvent) -> bool
    ) -> bool;
}

#[skyline::hook(offset = 0xceb020)]
pub unsafe extern "C" fn master_link_event(vtable: u64, fighter: &mut Fighter, event: &mut smash_rs::app::LinkEvent) -> bool {
    master_link_event_inner(vtable, fighter, event, original!())
}

pub fn install() {
    skyline::install_hooks!(
        master_link_event
    );
}
