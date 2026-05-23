use super::*;
use globals::*;
use rollcall::Roll;
// Addresses, offsets, and inline hooking
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use utils::game_modes::CustomMode;

type GetParamIntCallOriginal = extern "C" fn(u64, u64, u64) -> i32;
type GetParamFloatCallOriginal = extern "C" fn(u64, u64, u64) -> f32;

const COMMON: u64 = utils::hash40!("common");
const DAMAGE_FLY_CORRECTION_MAX: u64 = utils::hash40!("damage_fly_correction_max");
const DAMAGE_FLY_TOP_AIR_ACCEL_Y: u64 = utils::hash40!("damage_fly_top_air_accel_y");
const DAMAGE_FLY_TOP_SPEED_Y_STABLE: u64 = utils::hash40!("damage_fly_top_speed_y_stable");
const DAMAGE_LEVEL3: u64 = utils::hash40!("damage_level3");
const DIVE_SPEED_Y: u64 = utils::hash40!("dive_speed_y");
const HIT_STOP_DELAY_FLICK_MAX_COUNT: u64 = utils::hash40!("hit_stop_delay_flick_max_count");
const HOP_CLEAR_ATTACK_SPEED: u64 = utils::hash40!("hop_clear_attack_speed");
const HOP_LIFE: u64 = utils::hash40!("hop_life");
const HOP_SPEED_X: u64 = utils::hash40!("hop_speed_x");
const HOP_SPEED_Y: u64 = utils::hash40!("hop_speed_y");
const JUMP_SQUAT_FRAME: u64 = utils::hash40!("jump_squat_frame");
const JUST_SHIELD_PRECEDE_EXTENSION: u64 = utils::hash40!("just_shield_precede_extension");
const LANDING_ATTACK_AIR_FRAME_B: u64 = utils::hash40!("landing_attack_air_frame_b");
const LANDING_ATTACK_AIR_FRAME_F: u64 = utils::hash40!("landing_attack_air_frame_f");
const LANDING_ATTACK_AIR_FRAME_HI: u64 = utils::hash40!("landing_attack_air_frame_hi");
const LANDING_ATTACK_AIR_FRAME_LW: u64 = utils::hash40!("landing_attack_air_frame_lw");
const LANDING_ATTACK_AIR_FRAME_N: u64 = utils::hash40!("landing_attack_air_frame_n");
const LANDING_FRAME: u64 = utils::hash40!("landing_frame");
const LANDING_FRAME_ESCAPE_AIR_SLIDE_MAX: u64 = utils::hash40!("landing_frame_escape_air_slide_max");
const LANDING_HEAVY_FRAME: u64 = utils::hash40!("landing_heavy_frame");
const LIFE: u64 = utils::hash40!("life");
const MAX_SPEED: u64 = utils::hash40!("max_speed");
const MIN_SPEED: u64 = utils::hash40!("min_speed");
const N1_START_SPEED_X: u64 = utils::hash40!("n1_start_speed_x");
const N1_THROW_ANGLE: u64 = utils::hash40!("n1_throw_angle");
const PARAM_AURABALL: u64 = utils::hash40!("param_auraball");
const PARAM_FISHINGROD: u64 = utils::hash40!("param_fishingrod");
const PARAM_GORDO: u64 = utils::hash40!("param_gordo");
const PARAM_MOTION: u64 = utils::hash40!("param_motion");
const PARAM_SPECIAL_HI: u64 = utils::hash40!("param_special_hi");
const PARAM_SPECIAL_N: u64 = utils::hash40!("param_special_n");
const PARAM_SPECIAL_S: u64 = utils::hash40!("param_special_s");
const PARAM_SPIKEBALL: u64 = utils::hash40!("param_spikeball");
const PARAM_TRENCHMORTARBULLET: u64 = utils::hash40!("param_trenchmortarbullet");
const RUSH_SPEED: u64 = utils::hash40!("rush_speed");
const SHIELD_SETOFF_ADD: u64 = utils::hash40!("shield_setoff_add");
const SHIELD_SETOFF_MUL: u64 = utils::hash40!("shield_setoff_mul");
const SHOOT_ANGLE: u64 = utils::hash40!("shoot_angle");
const SHOOT_SPEED_X_MAX: u64 = utils::hash40!("shoot_speed_x_max");
const SHOOT_SPEED_X_MIN: u64 = utils::hash40!("shoot_speed_x_min");
const SHOOT_SPEED_Y_MAX: u64 = utils::hash40!("shoot_speed_y_max");
const SHOOT_SPEED_Y_MIN: u64 = utils::hash40!("shoot_speed_y_min");
const SHOOT_X: u64 = utils::hash40!("shoot_x");
const SHOOT_Y: u64 = utils::hash40!("shoot_y");
const SHOT_START_ANGLE: u64 = utils::hash40!("shot_start_angle");
const SPECIAL_HI_JET_ANG_F_MAX: u64 = utils::hash40!("special_hi_jet_ang_f_max");
const SPEED_X: u64 = utils::hash40!("speed_x");
const AIR_ACCEL_Y: u64 = utils::hash40!("air_accel_y");
const AIR_SPEED_Y_STABLE: u64 = utils::hash40!("air_speed_y_stable");
const EXPLODE: u64 = utils::hash40!("explode");
const HI1_FIRST_JUMP_Y_SPEED: u64 = utils::hash40!("hi1_first_jump_y_speed");
const HI2_RUSH_SPEED: u64 = utils::hash40!("hi2_rush_speed");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParamObjectKind {
    Fighter,
    Weapon,
    Other,
}

struct IntParamCtx {
    x0: u64,
    x1: u64,
    x2: u64,
    boma: *mut BattleObjectModuleAccessor,
    kind: i32,
    object_kind: ParamObjectKind,
    call_original: GetParamIntCallOriginal,
}

impl IntParamCtx {
    #[inline(always)]
    unsafe fn new(x0: u64, x1: u64, x2: u64, call_original: GetParamIntCallOriginal) -> Self {
        let boma = *((x0 as *mut u64).offset(1)) as *mut BattleObjectModuleAccessor;
        let boma_reference = &mut *boma;
        let object_kind = if boma_reference.is_fighter() {
            ParamObjectKind::Fighter
        } else if boma_reference.is_weapon() {
            ParamObjectKind::Weapon
        } else {
            ParamObjectKind::Other
        };

        Self {
            x0,
            x1,
            x2,
            boma,
            kind: boma_reference.kind(),
            object_kind,
            call_original,
        }
    }

    #[inline(always)]
    unsafe fn boma(&mut self) -> &mut BattleObjectModuleAccessor {
        &mut *self.boma
    }

    #[inline(always)]
    unsafe fn call_original(&self) -> i32 {
        (self.call_original)(self.x0, self.x1, self.x2)
    }
}

struct FloatParamCtx {
    x0: u64,
    x1: u64,
    x2: u64,
    boma: *mut BattleObjectModuleAccessor,
    kind: i32,
    object_kind: ParamObjectKind,
    owner_boma: Option<*mut BattleObjectModuleAccessor>,
    call_original: GetParamFloatCallOriginal,
}

impl FloatParamCtx {
    #[inline(always)]
    unsafe fn new(x0: u64, x1: u64, x2: u64, call_original: GetParamFloatCallOriginal) -> Self {
        let boma = *((x0 as *mut u64).offset(1)) as *mut BattleObjectModuleAccessor;
        let boma_reference = &mut *boma;
        let object_kind = if boma_reference.is_fighter() {
            ParamObjectKind::Fighter
        } else if boma_reference.is_weapon() {
            ParamObjectKind::Weapon
        } else {
            ParamObjectKind::Other
        };

        Self {
            x0,
            x1,
            x2,
            boma,
            kind: boma_reference.kind(),
            object_kind,
            owner_boma: None,
            call_original,
        }
    }

    #[inline(always)]
    unsafe fn boma(&mut self) -> &mut BattleObjectModuleAccessor {
        &mut *self.boma
    }

    #[inline(always)]
    unsafe fn owner_boma(&mut self) -> *mut BattleObjectModuleAccessor {
        if let Some(owner_boma) = self.owner_boma {
            owner_boma
        } else {
            let owner_boma = sv_battle_object::module_accessor(WorkModule::get_int(self.boma, *WEAPON_INSTANCE_WORK_ID_INT_LINK_OWNER) as u32);
            self.owner_boma = Some(owner_boma);
            owner_boma
        }
    }

    #[inline(always)]
    unsafe fn call_original(&self) -> f32 {
        (self.call_original)(self.x0, self.x1, self.x2)
    }

    #[inline(always)]
    unsafe fn call_original_with_param(&self, x1: u64, x2: u64) -> f32 {
        (self.call_original)(self.x0, x1, x2)
    }
}

pub fn install() {
    skyline::install_hooks!(
        //get_offset,
        //get_inline_offset,
        get_param_int_hook,
        get_param_float_hook,
        //get_item_backtrace_inline,
    );
    //skyline::nro::add_hook(item_nro_hook);
}

// #[skyline::hook(offset=0x720540)]
// unsafe fn get_offset(arg0: u64, arg1: u64) {
//     static mut ONCE: bool = true;
//     if ONCE {
//         ONCE = false;
//         //debug::dump_trace();
//     }
//     original!()(arg0, arg1);
// }

// #[skyline::hook(offset=0x1f8810c, inline)]
// unsafe fn get_inline_offset(ctx: &InlineCtx) {
//     static mut ONCE: bool = true;
//     if ONCE {
//         ONCE = false;
//         println!("{:#x}", ctx.registers[3].x.as_ref() - getRegionAddress(Region::Text) as u64);
//     }
// }

#[skyline::hook(offset = 0x4E53A0)]
pub unsafe fn get_param_int_hook(x0: u64, x1: u64, x2: u64) -> i32 {
    // Skyline defines original!/call_original! inside this hook body; store the
    // function pointer so Rollcall handlers can call through the context.
    let mut ctx = IntParamCtx::new(x0, x1, x2, original!());

    match rollcall::dispatch_until!(&mut ctx, ctx.object_kind, {
        ParamObjectKind::Fighter => [maybe_override_custom_mode_ints, override_just_shield_precede_extension],
        ParamObjectKind::Weapon => [dispatch_weapon_int_overrides],
        _ => [],
    }) {
        Roll::Return(value) => value,
        Roll::Continue | Roll::Break => ctx.call_original(),
    }
}

#[skyline::hook(offset = 0x4E53E0)]
pub unsafe fn get_param_float_hook(x0 /*boma*/: u64, x1 /*param_type*/: u64, x2 /*param_hash*/: u64) -> f32 {
    // Skyline defines original!/call_original! inside this hook body; store the
    // function pointer so Rollcall handlers can call through the context.
    let mut ctx = FloatParamCtx::new(x0, x1, x2, original!());

    match rollcall::dispatch_until!(&mut ctx, ctx.object_kind, {
        ParamObjectKind::Fighter => [
            maybe_override_custom_mode_floats,
            maybe_override_frame_data_debug_damage_fly_correction,
            offset_heavy_landing_faf_for_motion_start,
            dispatch_fighter_kind_float_overrides,
        ],
        ParamObjectKind::Weapon => [dispatch_weapon_float_overrides],
        _ => [],
    }) {
        Roll::Return(value) => value,
        Roll::Continue | Roll::Break => ctx.call_original(),
    }
}

#[inline(always)]
unsafe fn maybe_override_custom_mode_ints(ctx: &mut IntParamCtx) -> Roll<i32> {
    if let Some(modes) = utils::game_modes::get_custom_mode() {
        if modes.contains(&CustomMode::Smash64Mode) {
            if ctx.x1 == LANDING_HEAVY_FRAME {
                Roll::Return(4)
            } else {
                Roll::Continue
            }
        } else if modes.contains(&CustomMode::RivalsOfAetherMode) {
            if ctx.x1 == COMMON && ctx.x2 == HIT_STOP_DELAY_FLICK_MAX_COUNT {
                Roll::Return(0)
            } else if ctx.x1 == JUMP_SQUAT_FRAME {
                Roll::Return(5)
            } else {
                Roll::Continue
            }
        } else {
            Roll::Continue
        }
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn override_just_shield_precede_extension(ctx: &mut IntParamCtx) -> Roll<i32> {
    if ctx.x2 == JUST_SHIELD_PRECEDE_EXTENSION {
        Roll::Return(1000)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn dispatch_weapon_int_overrides(ctx: &mut IntParamCtx) -> Roll<i32> {
    rollcall::dispatch_until!(ctx, ctx.kind, {
        _ if ctx.kind == *WEAPON_KIND_PACKUN_SPIKEBALL => [override_exploding_spikeball_life],
        _ if ctx.kind == *WEAPON_KIND_LUCARIO_AURABALL => [override_powered_auraball_life],
        _ => [],
    })
}

#[inline(always)]
unsafe fn override_exploding_spikeball_life(ctx: &mut IntParamCtx) -> Roll<i32> {
    if ctx.x1 == PARAM_SPIKEBALL && ctx.x2 == HOP_LIFE && VarModule::is_flag(ctx.boma().object(), vars::packun_spikeball::instance::ENABLE_EXPLODE) {
        Roll::Return(105)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn override_powered_auraball_life(ctx: &mut IntParamCtx) -> Roll<i32> {
    if ctx.x1 == PARAM_AURABALL && ctx.x2 == LIFE && VarModule::is_flag(ctx.boma().object(), vars::lucario::instance::IS_POWERED_UP) {
        Roll::Return(180)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn maybe_override_custom_mode_floats(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if let Some(modes) = utils::game_modes::get_custom_mode() {
        if modes.contains(&CustomMode::Smash64Mode) {
            override_smash64_mode_floats(ctx)
        } else if modes.contains(&CustomMode::RivalsOfAetherMode) {
            override_rivals_mode_landings(ctx)
        } else {
            Roll::Continue
        }
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn override_smash64_mode_floats(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x2 == SHIELD_SETOFF_ADD {
        Roll::Return(4.0)
    } else if ctx.x2 == SHIELD_SETOFF_MUL {
        Roll::Return(1.62)
    } else if is_smash64_gravity_scaled_hash(ctx.x1) {
        Roll::Return(ctx.call_original() * 0.8)
    } else if is_any_landing_lag_hash(ctx.x1) {
        Roll::Return(4.0)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn override_rivals_mode_landings(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_MOTION && ctx.x2 == LANDING_FRAME_ESCAPE_AIR_SLIDE_MAX {
        Roll::Return(13.0)
    } else if ctx.x1 == LANDING_FRAME {
        VarModule::set_float(ctx.boma().object(), vars::common::instance::LANDING_LAG_FOR_RIVALS_MODE, 4.0);
        Roll::Return(4.0)
    } else if is_aerial_landing_lag_hash(ctx.x1) {
        let landing_lag = 2.0 + ctx.call_original();
        let prev_inflict_status = VarModule::get_int(ctx.boma().object(), vars::common::instance::PREV_STATUS_INFLICT_STATUS);

        if prev_inflict_status & *COLLISION_KIND_MASK_HIT != 0 {
            let reduced_landing_lag = (landing_lag * 0.6667).floor().max(4.0);
            VarModule::set_float(ctx.boma().object(), vars::common::instance::LANDING_LAG_FOR_RIVALS_MODE, reduced_landing_lag);
            Roll::Return(reduced_landing_lag)
        } else {
            VarModule::set_float(ctx.boma().object(), vars::common::instance::LANDING_LAG_FOR_RIVALS_MODE, landing_lag);
            Roll::Return(landing_lag)
        }
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn maybe_override_frame_data_debug_damage_fly_correction(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x2 == DAMAGE_FLY_CORRECTION_MAX && VarModule::is_flag(ctx.boma().object(), vars::common::instance::ENABLE_FRAME_DATA_DEBUG) {
        Roll::Return(15.0)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn offset_heavy_landing_faf_for_motion_start(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == LANDING_FRAME {
        Roll::Return(ctx.call_original_with_param(LANDING_FRAME, 0) + 2.0)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn dispatch_fighter_kind_float_overrides(ctx: &mut FloatParamCtx) -> Roll<f32> {
    rollcall::dispatch_until!(ctx, ctx.kind, {
        _ if ctx.kind == *FIGHTER_KIND_DIDDY => [maybe_override_diddy_ground_rocketbarrel_angle_limit],
        _ if ctx.kind == *FIGHTER_KIND_DONKEY => [reduce_donkey_barrel_carry_tumble_threshold],
        _ if ctx.kind == *FIGHTER_KIND_KEN => [override_ken_aerial_hadouken_offsets],
        _ if ctx.kind == *FIGHTER_KIND_LUCARIO => [scale_lucario_extreme_speed_rush_speed],
        _ if maybe_is_miifighter_or_kirby_copying_ironball(ctx) => [override_miifighter_ironball],
        _ if ctx.kind == *FIGHTER_KIND_MIISWORDSMAN => [maybe_override_miiswordsman_heavy_special_hi_rush_speed],
        _ if ctx.kind == *FIGHTER_KIND_MIIGUNNER => [maybe_override_miigunner_charged_special_hi_jump_speed],
        _ if ctx.kind == *FIGHTER_KIND_PFUSHIGISOU => [steer_ivysaur_razor_leaf_angle],
        _ => [],
    })
}

#[inline(always)]
unsafe fn maybe_override_diddy_ground_rocketbarrel_angle_limit(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_SPECIAL_HI && ctx.x2 == SPECIAL_HI_JET_ANG_F_MAX && WorkModule::get_int(ctx.boma, *FIGHTER_DIDDY_STATUS_SPECIAL_HI_WORK_INT_SITUATION) == *SITUATION_KIND_GROUND {
        Roll::Return(5.0)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn reduce_donkey_barrel_carry_tumble_threshold(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x2 == DAMAGE_LEVEL3 && (481..=489).contains(&ctx.boma().status()) {
        Roll::Return(ctx.call_original() * 0.5)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn override_ken_aerial_hadouken_offsets(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_SPECIAL_N && VarModule::is_flag(ctx.boma().object(), vars::shotos::instance::SPECIAL_N_HADOKEN_AIR) {
        rollcall::dispatch_first!(ctx, ctx.x2, {
            SHOOT_X => |_| Roll::Return(11.0),
            SHOOT_Y => |_| Roll::Return(6.0),
            _ => |_| Roll::Continue,
        })
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn scale_lucario_extreme_speed_rush_speed(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_SPECIAL_HI && ctx.x2 == RUSH_SPEED {
        let rate = VarModule::get_float(ctx.boma().object(), vars::lucario::instance::SPECIAL_HI_MOTION_RATE);
        if rate > 0.0 {
            Roll::Return(ctx.call_original() * rate)
        } else {
            Roll::Continue
        }
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn override_miifighter_ironball(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_SPECIAL_N {
        rollcall::dispatch_first!(ctx, ctx.x2, {
            N1_THROW_ANGLE => get_miifighter_ironball_angle,
            N1_START_SPEED_X => get_miifighter_ironball_speed,
            _ => |_| Roll::Continue,
        })
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn get_miifighter_ironball_angle(ctx: &mut FloatParamCtx) -> Roll<f32> {
    Roll::Return(VarModule::get_float(ctx.boma().object(), vars::miifighter::status::SPECIAL_N1_ANGLE))
}

#[inline(always)]
unsafe fn get_miifighter_ironball_speed(ctx: &mut FloatParamCtx) -> Roll<f32> {
    Roll::Return(VarModule::get_float(ctx.boma().object(), vars::miifighter::status::SPECIAL_N1_SPEED))
}

#[inline(always)]
unsafe fn maybe_override_miiswordsman_heavy_special_hi_rush_speed(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_SPECIAL_HI && ctx.x2 == HI2_RUSH_SPEED && VarModule::is_flag(ctx.boma().object(), vars::common::instance::IS_HEAVY_ATTACK) {
        Roll::Return(3.0)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn maybe_override_miigunner_charged_special_hi_jump_speed(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_SPECIAL_HI && ctx.x2 == HI1_FIRST_JUMP_Y_SPEED {
        let object = ctx.boma().object();
        let charge = VarModule::get_float(object, vars::miigunner::status::ATTACK_CHARGE);
        let base_y_speed = ParamModule::get_float(object, ParamType::Agent, "param_special_hi1.base_y_speed");
        let charge_y_speed_mul = ParamModule::get_float(object, ParamType::Agent, "param_special_hi1.charge_y_speed_mul");
        let charge_y_speed_div = ParamModule::get_float(object, ParamType::Agent, "param_special_hi1.charge_y_speed_div");
        Roll::Return(base_y_speed + (charge_y_speed_mul * charge) / charge_y_speed_div)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn steer_ivysaur_razor_leaf_angle(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_SPECIAL_S && ctx.x2 == SHOOT_ANGLE {
        Roll::Return(ControlModule::get_stick_y(ctx.boma) * 25.0)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn dispatch_weapon_float_overrides(ctx: &mut FloatParamCtx) -> Roll<f32> {
    rollcall::dispatch_until!(ctx, ctx.kind, {
        _ if ctx.kind == *WEAPON_KIND_SNAKE_TRENCHMORTAR_BULLET => [maybe_steer_snake_trenchmortar_bullet_speed],
        _ if ctx.kind == *WEAPON_KIND_DEDEDE_GORDO => [steer_dedede_gordo_angle],
        _ if ctx.kind == *WEAPON_KIND_PICKEL_FISHINGROD => [steer_pickel_fishingrod_shoot_angle],
        _ if ctx.kind == *WEAPON_KIND_PACKUN_SPIKEBALL => [override_packun_spikeball_speeds],
        _ if ctx.kind == *WEAPON_KIND_LUCARIO_AURABALL => [override_powered_auraball_speed],
        _ => [],
    })
}

#[inline(always)]
unsafe fn maybe_steer_snake_trenchmortar_bullet_speed(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_TRENCHMORTARBULLET && ctx.x2 == SPEED_X {
        Roll::Return(ControlModule::get_stick_x(ctx.boma) / 1.5 * PostureModule::lr(ctx.boma))
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn steer_dedede_gordo_angle(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_GORDO && ctx.x2 == SHOT_START_ANGLE {
        Roll::Return(20.0 * ControlModule::get_stick_y(ctx.owner_boma()))
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn steer_pickel_fishingrod_shoot_angle(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_FISHINGROD && ctx.x2 == SHOOT_ANGLE {
        let stick_y = ControlModule::get_stick_y(ctx.owner_boma());
        if stick_y < 0.0 {
            Roll::Return(48.0 + (stick_y * 30.0))
        } else {
            Roll::Return(48.0)
        }
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn override_packun_spikeball_speeds(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_SPIKEBALL {
        if VarModule::is_flag(ctx.boma().object(), vars::packun_spikeball::instance::ENABLE_EXPLODE) {
            let is_explode_motion = MotionModule::motion_kind(ctx.boma()) == EXPLODE;
            rollcall::dispatch_first!(ctx, ctx.x2, {
                HOP_SPEED_X => |_| Roll::Return(0.0),
                HOP_SPEED_Y => |_| Roll::Return(0.0),
                HOP_CLEAR_ATTACK_SPEED if is_explode_motion => |_| Roll::Return(-0.1),
                _ => |_| Roll::Continue,
            })
        } else if VarModule::get_int((*ctx.owner_boma()).object(), vars::packun::instance::CURRENT_STANCE) == 2 {
            rollcall::dispatch_first!(ctx, ctx.x2, {
                SHOOT_SPEED_X_MAX => |_| Roll::Return(1.5),
                SHOOT_SPEED_Y_MAX => |_| Roll::Return(1.3),
                SHOOT_SPEED_X_MIN => |_| Roll::Return(0.5),
                SHOOT_SPEED_Y_MIN => |_| Roll::Return(0.4),
                _ => |_| Roll::Continue,
            })
        } else {
            Roll::Continue
        }
    } else {
        Roll::Continue
    }
}

#[inline(always)]
unsafe fn override_powered_auraball_speed(ctx: &mut FloatParamCtx) -> Roll<f32> {
    if ctx.x1 == PARAM_AURABALL && VarModule::is_flag(ctx.boma().object(), vars::lucario::instance::IS_POWERED_UP) && (ctx.x2 == MIN_SPEED || ctx.x2 == MAX_SPEED) {
        Roll::Return(0.7)
    } else {
        Roll::Continue
    }
}

#[inline(always)]
fn is_smash64_gravity_scaled_hash(param_type: u64) -> bool {
    matches!(param_type, AIR_SPEED_Y_STABLE | AIR_ACCEL_Y | DAMAGE_FLY_TOP_AIR_ACCEL_Y | DAMAGE_FLY_TOP_SPEED_Y_STABLE | DIVE_SPEED_Y)
}

#[inline(always)]
fn is_any_landing_lag_hash(param_type: u64) -> bool {
    param_type == LANDING_FRAME || is_aerial_landing_lag_hash(param_type)
}

#[inline(always)]
fn is_aerial_landing_lag_hash(param_type: u64) -> bool {
    matches!(param_type, LANDING_ATTACK_AIR_FRAME_N | LANDING_ATTACK_AIR_FRAME_F | LANDING_ATTACK_AIR_FRAME_B | LANDING_ATTACK_AIR_FRAME_HI | LANDING_ATTACK_AIR_FRAME_LW)
}

#[inline(always)]
unsafe fn maybe_is_miifighter_or_kirby_copying_ironball(ctx: &mut FloatParamCtx) -> bool {
    ctx.kind == *FIGHTER_KIND_MIIFIGHTER || (ctx.kind == *FIGHTER_KIND_KIRBY && WorkModule::get_int(ctx.boma, *FIGHTER_KIRBY_INSTANCE_WORK_ID_INT_COPY_CHARA) == 0x48)
}

// #[skyline::hook(offset=0x165d0b0, inline)]
// unsafe fn get_item_backtrace_inline(ctx: &InlineCtx) {
//     ::utils::dump_trace!(0x0);
// }

// fn item_nro_hook(info: &skyline::nro::NroInfo) {
//     if info.name == "item" {
//         unsafe {
//             //println!("Module Base: {:#x}", (*info.module.ModuleObject).module_base);
//             EXPLOSIONBOMB_ADDRESS += (*info.module.ModuleObject).module_base;
//             skyline::install_hook!(get_explosionbomb_hook);
//         }
//     }
// }

// static mut EXPLOSIONBOMB_ADDRESS: u64 = 0x8c4940;//0x8c493c;

// #[skyline::hook(replace=EXPLOSIONBOMB_ADDRESS, inline)]
// pub unsafe fn get_explosionbomb_hook(ctx: &InlineCtx) {
//     let agent = (ctx.registers[21].x()) as *mut L2CAgentBase;
//     //println!("Agent: {}", (*agent).kind());
//     let lua_state = (*agent).lua_state_agent;
//     let item_boma = sv_system::battle_object_module_accessor(lua_state);
//     let owner_id = WorkModule::get_int(item_boma, *WEAPON_INSTANCE_WORK_ID_INT_LINK_OWNER) as u32;
//     if sv_battle_object::kind(owner_id) == *FIGHTER_KIND_SHEIK {
// 		let sheik = utils::util::get_battle_object_from_id(owner_id);
//         let item_id = item_boma.battle_object_id;
//         VarModule::set_int(sheik, vars::sheik::status::GRENADE_OBJECT_ID, item_id as i32);
//         //let sheik_boma = &mut *(*zelda).module_accessor;
//         //let value = VarModule::get_float(sheik, vars::sheik::status::GRENADE_GRAVITY);
//         //ctx.registers_f[0].set_s(value);
//     }
// }
