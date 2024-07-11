import { LoadedMap } from "..";
import kImageRadar from "./radar.png";
import kImageBuyZones from "./overlay_buyzones.png";

export default {
    mapName: "de_cache",
    displayName: "Cache",

    metaInfo: {
        "resolution": 5.54,

        "offset": {
            "x": 2020,
            "y": 2390
        },

        "floors": []
    },

    overlayBuyzones: kImageBuyZones,
    overlayRadar: kImageRadar,
} satisfies LoadedMap;