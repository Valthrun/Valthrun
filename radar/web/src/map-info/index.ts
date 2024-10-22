export type MapMetaFloor = {
    offset: {
        x: number;
        y: number;
    };
    zRange: {
        min: number;
        max: number;
    };
};

export type MapMetaData = {
    resolution: number;
    offset: {
        x: number;
        y: number;
    };
    floors: MapMetaFloor[];
};

export const kRegisteredMaps: Record<string, () => Promise<LoadedMap>> = {
    de_ancient: () => import("./de_ancient").then((value) => value.default),
    de_anubis: () => import("./de_anubis").then((value) => value.default),
    de_cache: () => import("./de_cache").then((value) => value.default),
    de_dust2: () => import("./de_dust2").then((value) => value.default),
    de_inferno: () => import("./de_inferno").then((value) => value.default),
    de_mirage: () => import("./de_mirage").then((value) => value.default),
    de_nuke: () => import("./de_nuke").then((value) => value.default),
    de_overpass: () => import("./de_overpass").then((value) => value.default),
    de_train: () => import("./de_train").then((value) => value.default),
    de_vertigo: () => import("./de_vertigo").then((value) => value.default),
    cs_office: () => import("./cs_office").then((value) => value.default),
};

export type LoadedMap = {
    mapName: string;
    displayName: string;

    metaInfo: MapMetaData;

    overlayBuyzones: string;
    overlayRadar: string;
};

export const loadMap = async (name: string): Promise<LoadedMap | null> => {
    const mapInfo = kRegisteredMaps[name];
    if (!mapInfo) {
        return null;
    }

    return await mapInfo();
};
