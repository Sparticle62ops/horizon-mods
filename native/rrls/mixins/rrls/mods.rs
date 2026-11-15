// The mod's row in Options > Mods.
fn rrls_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::rrls as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::MOD_ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
