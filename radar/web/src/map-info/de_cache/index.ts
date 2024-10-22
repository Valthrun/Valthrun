import { LoadedMap } from "..";
import kImageBuyZones from "./overlay_buyzones.png";
import kImageRadar from "./radar.png";

export default {
    mapName: "de_cache",
    displayName: "Cache",

    metaInfo: {
        resolution: 5.54,

        offset: {
            x: 2020,
            y: 2390,
        },

        floors: [],
    },

    overlayBuyzones: kImageBuyZones,
    overlayRadar: kImageRadar,
} satisfies LoadedMap;
