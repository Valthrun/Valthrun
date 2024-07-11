import { LoadedMap } from "..";
import kImageRadar from "./radar.png";
import kImageBuyZones from "./overlay_buyzones.png";

export default {
    mapName: "de_ancient",
    displayName: "Ancient",

    metaInfo: {
        "resolution": 4.26,

        "offset": {
            "x": 2590,
            "y": 2520
        },

        "floors": []
    },

    overlayBuyzones: kImageBuyZones,
    overlayRadar: kImageRadar,
} satisfies LoadedMap;