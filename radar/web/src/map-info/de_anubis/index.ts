import { LoadedMap } from "..";
import kImageBuyZones from "./overlay_buyzones.png";
import kImageRadar from "./radar.png";

export default {
    mapName: "de_anubis",
    displayName: "Anubis",

    metaInfo: {
        resolution: 5.25,

        offset: {
            x: 2830,
            y: 2030,
        },

        floors: [],
    },

    overlayBuyzones: kImageBuyZones,
    overlayRadar: kImageRadar,
} satisfies LoadedMap;
