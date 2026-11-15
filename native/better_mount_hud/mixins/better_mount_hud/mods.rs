// The mod's row in Options > Mods.
fn better_mount_hud_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::better_mount_hud::better_mount_hud as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::MOD_ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
