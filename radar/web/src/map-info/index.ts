export type MapMetaFloor = {
    "offset": {
        "x": number,
        "y": number
    },
    "zRange": {
        "min": number,
        "max": number
    }
}

export type MapMetaData = {
    resolution: number,
    offset: {
        "x": number,
        "y": number,
    },
    floors: MapMetaFloor[]
}

type RegisteredMap = {
    [K in keyof LoadedMap]: () => Promise<LoadedMap[K]>
};

export const kRegisteredMaps: Record<string, RegisteredMap> = {
    "de_ancient": {
        displayName: async () => "Ancient",
        metaInfo: () => import("./de_ancient/meta.json").then(value => value.default),
        overlayBuyzones: () => import("./de_ancient/overlay_buyzones.png").then(value => value.default),
        overlayRadar: () => import("./de_ancient/radar.png").then(value => value.default),
    },
    "de_anubis": {
        displayName: async () => "Anubis",
        metaInfo: () => import("./de_anubis/meta.json").then(value => value.default),
        overlayBuyzones: () => import("./de_anubis/overlay_buyzones.png").then(value => value.default),
        overlayRadar: () => import("./de_anubis/radar.png").then(value => value.default),
    },
    "de_cache": {
        displayName: async () => "Cache",
        metaInfo: () => import("./de_cache/meta.json").then(value => value.default),
        overlayBuyzones: () => import("./de_cache/overlay_buyzones.png").then(value => value.default),
        overlayRadar: () => import("./de_cache/radar.png").then(value => value.default),
    },
    "de_dust2": {
        displayName: async () => "Dust 2",
        metaInfo: () => import("./de_dust2/meta.json").then(value => value.default),
        overlayBuyzones: () => import("./de_dust2/overlay_buyzones.png").then(value => value.default),
        overlayRadar: () => import("./de_dust2/radar.png").then(value => value.default),
    },
    "de_inferno": {
        displayName: async () => "Inferno",
        metaInfo: () => import("./de_inferno/meta.json").then(value => value.default),
        overlayBuyzones: () => import("./de_inferno/overlay_buyzones.png").then(value => value.default),
        overlayRadar: () => import("./de_inferno/radar.png").then(value => value.default),
    },
    "de_mirage": {
        displayName: async () => "Mirage",
        metaInfo: () => import("./de_mirage/meta.json").then(value => value.default),
        overlayBuyzones: () => import("./de_mirage/overlay_buyzones.png").then(value => value.default),
        overlayRadar: () => import("./de_mirage/radar.png").then(value => value.default),
    },
    "de_nuke": {
        displayName: async () => "Nuke",
        metaInfo: () => import("./de_nuke/meta.json").then(value => value.default),
        overlayBuyzones: () => import("./de_nuke/overlay_buyzones.png").then(value => value.default),
        overlayRadar: () => import("./de_nuke/radar.png").then(value => value.default),
    },
    "de_overpass": {
        displayName: async () => "Overpass",
        metaInfo: () => import("./de_overpass/meta.json").then(value => value.default),
        overlayBuyzones: () => import("./de_overpass/overlay_buyzones.png").then(value => value.default),
        overlayRadar: () => import("./de_overpass/radar.png").then(value => value.default),
    },
    "de_train": {
        displayName: async () => "Train",
        metaInfo: () => import("./de_train/meta.json").then(value => value.default),
        overlayBuyzones: () => import("./de_train/overlay_buyzones.png").then(value => value.default),
        overlayRadar: () => import("./de_train/radar.png").then(value => value.default),
    },
    "de_vertigo": {
        displayName: async () => "Vertigo",
        metaInfo: () => import("./de_vertigo/meta.json").then(value => value.default),
        overlayBuyzones: () => import("./de_vertigo/overlay_buyzones.png").then(value => value.default),
        overlayRadar: () => import("./de_vertigo/radar.png").then(value => value.default),
    },
}

export type LoadedMap = {
    displayName: string,
    metaInfo: MapMetaData,
    overlayBuyzones: string,
    overlayRadar: string
};

export const loadMap = async (name: string): Promise<LoadedMap | null> => {
    const mapInfo = kRegisteredMaps[name];
    if (!mapInfo) {
        return null;
    }

    return {
        displayName: await mapInfo.displayName(),
        metaInfo: await mapInfo.metaInfo(),
        overlayRadar: await mapInfo.overlayRadar(),
        overlayBuyzones: await mapInfo.overlayBuyzones()
    }
}