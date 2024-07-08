pub const WEAPON_FLAG_TYPE_KNIFE: u32 = 0x01;
pub const WEAPON_FLAG_TYPE_PISTOL: u32 = 0x02;
pub const WEAPON_FLAG_TYPE_SHOTGUN: u32 = 0x04;
pub const WEAPON_FLAG_TYPE_SMG: u32 = 0x08;
pub const WEAPON_FLAG_TYPE_RIFLE: u32 = 0x10;
pub const WEAPON_FLAG_TYPE_SNIPER_RIFLE: u32 = 0x20;
pub const WEAPON_FLAG_TYPE_MACHINE_GUN: u32 = 0x40;
pub const WEAPON_FLAG_TYPE_GRENADE: u32 = 0x80;

macro_rules! define_weapons {
    (
        $(#[$struct_meta:meta])*
        pub enum $struct_name:ident {
            $(
                    $member_name:ident {
                    id: $id:literal,
                    name: $name:literal,
                    flags: $flags:tt
                },
            )*
        }
    ) => {
        $(#[$struct_meta])*
        pub enum $struct_name {
            $($member_name = $id,)*
        }

        impl $struct_name {
            pub fn all_weapons() -> Vec<Self> {
                vec![
                    $(Self::$member_name, )*
                ]
            }

            pub fn from_id(id: u16) -> Option<Self> {
                match id {
                    $($id => Some(Self::$member_name),)*
                    _ => None,
                }
            }

            pub fn id(&self) -> u16 {
                match self {
                    $(Self::$member_name => $id,)*
                }
            }

            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$member_name => stringify!($member_name),)*
                }
            }

            pub fn flags(&self) -> u32 {
                match self {
                    $(Self::$member_name => $flags,)*
                }
            }

            pub fn display_name(&self) -> &'static str {
                match self {
                    $(Self::$member_name => $name,)*
                }
            }
        }
    };
}

define_weapons! {
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub enum WeaponId {
        Unknown { id: 0, name: "Unknown", flags: WEAPON_FLAG_TYPE_KNIFE },
        Deagle { id: 1, name: "Desert Eagle", flags: WEAPON_FLAG_TYPE_PISTOL },
        Elite { id: 2, name: "Elite", flags: 0 },
        FiveSeven { id: 3, name: "Five-SeveN", flags: WEAPON_FLAG_TYPE_PISTOL },
        Glock { id: 4, name: "Glock-18", flags: WEAPON_FLAG_TYPE_PISTOL },
        Ak47 { id: 7, name: "AK-47", flags: WEAPON_FLAG_TYPE_RIFLE },
        Aug { id: 8, name: "AUG", flags: WEAPON_FLAG_TYPE_RIFLE },
        AWP { id: 9, name: "AWP", flags: WEAPON_FLAG_TYPE_SNIPER_RIFLE },
        Famas { id: 10, name: "FAMAS", flags: WEAPON_FLAG_TYPE_RIFLE },
        G3SG1 { id: 11, name: "G3SG1", flags: WEAPON_FLAG_TYPE_SNIPER_RIFLE },
        Galilar { id: 13, name: "Galil AR", flags: WEAPON_FLAG_TYPE_RIFLE },
        M249 { id: 14, name: "M249", flags: WEAPON_FLAG_TYPE_MACHINE_GUN },
        M4A4 { id: 16, name: "M4A4", flags: WEAPON_FLAG_TYPE_RIFLE },
        Mac10 { id: 17, name: "MAC-10", flags: WEAPON_FLAG_TYPE_SMG },
        P90 { id: 19, name: "P90", flags: WEAPON_FLAG_TYPE_SMG },
        MP5SD { id: 23, name: "MP5-SD", flags: WEAPON_FLAG_TYPE_SMG },
        Ump45 { id: 24, name: "UMP-45", flags: WEAPON_FLAG_TYPE_SMG },
        XM1014 { id: 25, name: "XM1014", flags: WEAPON_FLAG_TYPE_SHOTGUN },
        Bizon { id: 26, name: "PP-Bizon", flags: WEAPON_FLAG_TYPE_SMG },
        Mag7 { id: 27, name: "MAG-7", flags: WEAPON_FLAG_TYPE_SHOTGUN },
        Negev { id: 28, name: "Negev", flags: WEAPON_FLAG_TYPE_MACHINE_GUN },
        SawedOff { id: 29, name: "Sawed-Off", flags: WEAPON_FLAG_TYPE_SHOTGUN },
        Tec9 { id: 30, name: "Tec-9", flags: WEAPON_FLAG_TYPE_PISTOL },
        Taser { id: 31, name: "Zeus x27", flags: 0 },
        HKP200 { id: 32, name: "P2000", flags: WEAPON_FLAG_TYPE_PISTOL },
        MP7 { id: 33, name: "MP7", flags: WEAPON_FLAG_TYPE_SMG },
        MP9 { id: 34, name: "MP9", flags: WEAPON_FLAG_TYPE_SMG },
        Nova { id: 35, name: "Nova", flags: WEAPON_FLAG_TYPE_SHOTGUN },
        P250 { id: 36, name: "P250", flags: WEAPON_FLAG_TYPE_PISTOL },
        Scar20 { id: 38, name: "SCAR-20", flags: WEAPON_FLAG_TYPE_SNIPER_RIFLE },
        Sg553 { id: 39, name: "SG 553", flags: WEAPON_FLAG_TYPE_RIFLE },
        Ssg08 { id: 40, name: "SSG 08", flags: WEAPON_FLAG_TYPE_SNIPER_RIFLE },
        Knife { id: 42, name: "Knife", flags: WEAPON_FLAG_TYPE_KNIFE },
        Flashbang { id: 43, name: "Flashbang", flags: WEAPON_FLAG_TYPE_GRENADE },
        HZgrenade { id: 44, name: "HE grenade", flags: WEAPON_FLAG_TYPE_GRENADE },
        Smokegrenade { id: 45, name: "Smoke Grenade", flags: WEAPON_FLAG_TYPE_GRENADE },
        Molotov { id: 46, name: "Molotov", flags: WEAPON_FLAG_TYPE_GRENADE },
        Decoy { id: 47, name: "Decoy Grenade", flags: WEAPON_FLAG_TYPE_GRENADE },
        Incendiary { id: 48, name: "Incendiary", flags: WEAPON_FLAG_TYPE_GRENADE },
        C4 { id: 49, name: "C4", flags: 0 },
        Healthshot { id: 57, name: "Healthshot", flags: 0 },
        KnifeT { id: 59, name: "Knife (T)", flags: WEAPON_FLAG_TYPE_KNIFE },
        M4A1Silencer { id: 60, name: "M4A1-S", flags: WEAPON_FLAG_TYPE_RIFLE },
        USPS { id: 61, name: "USP-S", flags: WEAPON_FLAG_TYPE_RIFLE },
        CZ75a { id: 63, name: "CZ75-Auto", flags: WEAPON_FLAG_TYPE_RIFLE },
        Revolver { id: 64, name: "Revolver", flags: WEAPON_FLAG_TYPE_RIFLE },

        KnifeBayonet { id: 500, name: "Knife (Bayonet)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifesClassic { id: 503, name: "Knife (Classic)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifeFlip { id: 505, name: "Knife (Flip)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifeGut { id: 506, name: "Knife (Gut)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifeKarambit { id: 507, name: "Knife (Karambit)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifeM9Bayonet { id: 508, name: "Knife (M9-Bayonet)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifeTactical { id: 509, name: "Knife (Tactical)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifeFalchion { id: 512, name: "Knife (Falchion)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifeSurvivalBowie { id: 514, name: "Knife (Survival Bowie)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifeButterfly { id: 515, name: "Knife (Butterfly)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifePush { id: 516, name: "Knife (Push)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifeCord { id: 517, name: "Knife (Cord)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifeSurvival { id: 518, name: "Knife (Survival)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifeUrsus { id: 519, name: "Knife (Ursus)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifesNavaja { id: 520, name: "Knife (Navaja)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifesNomad { id: 521, name: "Knife (Nomad)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifesStiletto { id: 522, name: "Knife (Stiletto)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifesTalon { id: 523, name: "Knife (Talon)", flags: WEAPON_FLAG_TYPE_KNIFE },
        KnifesSkeleton { id: 525, name: "Knife (Skeleton)", flags: WEAPON_FLAG_TYPE_KNIFE },
    }
}
