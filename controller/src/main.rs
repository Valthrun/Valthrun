#![feature(iterator_try_collect)]
#![allow(dead_code)]

use std::fmt::Debug;
use anyhow::Context;
use cs2_schema::offsets;
use imgui::Condition;
use imgui_winit_support::{WinitPlatform, HiDpiMode, winit::{event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, event::{Event, WindowEvent}}};
use schema::dump_schema;
use valthrun_kinterface::ByteSequencePattern;

use crate::handle::{CS2Handle, Module};

mod handle;
mod schema;
mod overlay;

#[repr(C)]
#[derive(Default)]
pub struct EntityHandle {
    pub value: u32,
}

impl Debug for EntityHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityHandle")
            .field("value", &self.value)
            .field("entity_index", &format_args!("0x{:X}", &self.get_entity_index()))
            .field("serial_number", &format_args!("0x{:X}", &self.get_serial_number()))
            .finish()
    }
}

impl EntityHandle {
    pub fn get_entity_index(&self) -> u32 {
        (self.value & 0x7FFF) as u32
    }

    pub fn is_valid(&self) -> bool {
        self.get_entity_index() < 0x7FF0
    }

    pub fn get_serial_number(&self) -> u32 {
        (self.value >> 15) as u32
    }

    pub fn entity_array_offsets(&self) -> (u64, u64) {
        let entity_index = self.get_entity_index();
        ((entity_index >> 9) as u64, (entity_index & 0x1FF) as u64)
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct EntityIdentity {
    entity_ptr: u64,
    ptr_2: u64,

    handle: EntityHandle,
    name_stringable_index: u32,
    name: u64,

    designer_name: u64,
    pad_0: u64,

    flags: u64,
    world_group_id: u32,
    data_object_types: u32,

    path_index: u64,
    pad_1: u64,
    
    pad_2: u64,
    p_prev: u64,

    p_next: u64,
    p_prev_by_class: u64,
    
    p_next_by_class: u64,
}

fn find_local_player_controller_ptr(cs2: &CS2Handle) -> anyhow::Result<u64> {
    // 48 83 3D ? ? ? ? ? 0F 95 -> IsLocalPlayerControllerValid
    let pattern = ByteSequencePattern::parse("48 83 3D ? ? ? ? ? 0F 95").unwrap();
    let inst_address = cs2.find_pattern(Module::Client, &pattern)?
        .context("failed to find local player controller ptr")?;

    let address = inst_address + cs2.read::<i32>(Module::Client, &[ inst_address + 0x03 ])? as u64 + 0x08;
    log::debug!("Local player controller ptr at {:X}", address);
    Ok(address)
}

fn find_entity_list(cs2: &CS2Handle) -> anyhow::Result<u64> {
    // 4C 8B 0D ? ? ? ? 48 89 5C 24 ? 8B -> Global entity list
    let pattern_entity_list = ByteSequencePattern::parse("4C 8B 0D ? ? ? ? 48 89 5C 24 ? 8B").unwrap();
    let inst_address = cs2.find_pattern(Module::Client, &pattern_entity_list)
        .context("missing entity list")?
        .context("failed to find global entity list pattern")?;
    let entity_list_address = inst_address + cs2.read::<i32>(Module::Client, &[ inst_address + 0x03 ])? as u64 + 0x07;
    log::debug!("Entity list at {:X}", entity_list_address);
    Ok(entity_list_address)
}

fn dump_entities(cs2: &CS2Handle) -> anyhow::Result<()> {
    let entity_list_address = find_entity_list(cs2)?;
    let mut entity_identity = cs2.read::<u64>(Module::Client, &[
        entity_list_address,
        0, /* read first entity identity of the first bucket */
    ])?;

    loop {
        let prev_identity = cs2.read::<u64>(Module::Absolute, &[ entity_identity + offsets::client::CEntityIdentity::m_pPrev ])   
            .context("failed to read prev entity")?;
        if prev_identity == 0 {
            break;
        }

        entity_identity = prev_identity;
    }
    
    while entity_identity > 0 {
        let designer_name = cs2.read_string(Module::Absolute, &[ 
            entity_identity + offsets::client::CEntityIdentity::m_designerName, 
            0
        ], Some(32))
            .unwrap_or_else(|_| "<< ERROR >>".to_string());

        if designer_name == "cs_player_controller" {
            let player_controller = cs2.read::<u64>(Module::Absolute, &[ entity_identity ])?;
            let player_health = cs2.read::<i32>(Module::Absolute, &[ 
                player_controller + offsets::client::CCSPlayerController::m_iPawnHealth
            ])?;
            let player_name = cs2.read_string(Module::Absolute, &[ 
                player_controller + offsets::client::CBasePlayerController::m_iszPlayerName 
            ], Some(128))?;

            let player_pawn_handle = cs2.read::<EntityHandle>(Module::Absolute, &[
                player_controller + offsets::client::CCSPlayerController::m_hPlayerPawn
            ])?;

            let (bulk, offset) = player_pawn_handle.entity_array_offsets();
            let player_pawn = cs2.read::<u64>(Module::Client, &[
                entity_list_address,
                bulk * 0x08,
                offset * 120,
            ])?;

            let player_collision = cs2.read::<u64>(Module::Absolute, &[
                player_pawn + offsets::client::C_BaseEntity::m_pCollision
            ])?;

            let abb = cs2.read::<[f32; 6]>(Module::Absolute, &[
                player_collision + offsets::client::CCollisionProperty::m_vecMins
            ])?;

            log::info!(" - {} {} at 0x{:X}. Health: {: >3}, {}.", entity_identity,
                designer_name,
                player_controller,
                player_health,
                player_name,
            );
            log::info!("    Pawn: {:?} -> {:X}", player_pawn_handle, player_pawn);
            log::info!("    Collision: {:X}", player_collision);
            log::info!("    ABB: {:?}", abb);
        }

        entity_identity = cs2.read(Module::Absolute, &[ entity_identity + offsets::client::CEntityIdentity::m_pNext ])?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    {
        // let cs2 = CS2Handle::create()?;
        // // dump_schema(&cs2)?;
        // // dump_entities(&cs2)?;

        // let controller_ptr = find_local_player_controller_ptr(&cs2)?;
        // let entity_identity = cs2.read::<u64>(Module::Client, &[
        //     controller_ptr,
        //     offsets::client::CEntityInstance::m_pEntity
        // ])?;

        // log::info!("Identity name: {}",
        //     cs2.read_string(Module::Absolute, &[ entity_identity + offsets::client::CEntityIdentity::m_designerName, 0 ], Some(32))?
        // );
    }

    
    overlay::init("Test")
        .main_loop(|_run, ui| {
            ui.window("Hello world")
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("Hello world!");
                    ui.text("こんにちは世界！");
                    ui.text("This...is...imgui-rs!");
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });
        });

    Ok(())
}
