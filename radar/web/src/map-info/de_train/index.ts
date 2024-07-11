import { LoadedMap } from "..";
import kImageRadar from "./radar.png";
import kImageBuyZones from "./overlay_buyzones.png";

export default {
    mapName: "de_train",
    displayName: "Train",

    metaInfo: {
        "resolution": 4.74,

        "offset": {
            "x": 2510,
            "y": 2440
        },

        "floors": []
    },

    overlayBuyzones: kImageBuyZones,
    overlayRadar: kImageRadar,
} satisfies LoadedMap;