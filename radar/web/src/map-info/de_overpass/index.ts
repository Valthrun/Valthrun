import { LoadedMap } from "..";
import kImageBuyZones from "./overlay_buyzones.png";
import kImageRadar from "./radar.png";

export default {
    mapName: "de_overpass",
    displayName: "Overpass",

    metaInfo: {
        resolution: 5.18,

        offset: {
            x: 4830,
            y: 3540,
        },

        floors: [],
    },

    overlayBuyzones: kImageBuyZones,
    overlayRadar: kImageRadar,
} satisfies LoadedMap;
